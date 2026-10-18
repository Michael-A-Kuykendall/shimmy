#!/usr/bin/env python3
"""Exercise a built Shimmy against local GGUF weights; failures exit nonzero.

python3 scripts/smoke_text.py --binary target/release/shimmy --model ~/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf
Use --expected Paris --prompt 'What is the capital of France? Answer briefly.'
GPU driver/environment configuration is inherited from the caller.
"""
import argparse
import json
import pathlib
import os
import secrets
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request


def request(base, path, payload=None, timeout=180, authenticated=True):
    data = None if payload is None else json.dumps(payload).encode()
    headers = {"Content-Type": "application/json"}
    if authenticated and os.environ.get("SHIMMY_API_KEY"):
        headers["Authorization"] = "Bearer " + os.environ["SHIMMY_API_KEY"]
    req = urllib.request.Request(base + path, data=data, headers=headers)
    with urllib.request.urlopen(req, timeout=timeout) as response:
        return response.headers.get_content_type(), response.read().decode()


def run(args):
    os.environ.setdefault("SHIMMY_API_KEY", secrets.token_hex(32))
    binary = pathlib.Path(args.binary).expanduser().resolve()
    model = pathlib.Path(args.model).expanduser().resolve()
    if not binary.is_file() or not model.is_file():
        raise RuntimeError("binary and model must both exist")
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        port = sock.getsockname()[1]
    base = f"http://127.0.0.1:{port}"
    with tempfile.TemporaryDirectory(prefix="shimmy-smoke-") as temp:
        log_path = pathlib.Path(args.log).expanduser().resolve()
        log_path.parent.mkdir(parents=True, exist_ok=True)
        with log_path.open("w") as log:
            server = subprocess.Popen(
                [str(binary), "--ctx", "512", "--prefill-chunk", "8", "serve",
                 "--model-path", str(model), "--bind", f"127.0.0.1:{port}"],
                cwd=temp, stdout=log, stderr=subprocess.STDOUT,
            )
            try:
                deadline = time.monotonic() + 20
                while True:
                    if server.poll() is not None:
                        raise RuntimeError(f"server exited {server.returncode}; see {log_path}")
                    try:
                        _, body = request(base, "/health", timeout=1)
                        assert json.loads(body)["status"] == "ok"
                        break
                    except (urllib.error.URLError, TimeoutError):
                        if time.monotonic() >= deadline:
                            raise RuntimeError("server readiness timed out")
                        time.sleep(0.2)
                try:
                    request(base, "/v1/models", authenticated=False)
                    raise AssertionError("unauthenticated model listing was accepted")
                except urllib.error.HTTPError as error:
                    assert error.code == 401, error.code
                print("PASS authentication rejects missing credentials", flush=True)
                _, body = request(base, "/v1/models")
                names = [entry["id"] for entry in json.loads(body)["data"]]
                assert model.stem in names, names
                print(f"PASS health + model discovery: {model.stem}", flush=True)
                messages = [{"role": "user", "content": args.prompt}]
                for label, path, payload in [
                    ("chat default JSON", "/v1/chat/completions", {"messages": messages}),
                    ("chat explicit JSON", "/v1/chat/completions", {"messages": messages, "stream": False}),
                    ("text completion", "/v1/completions", {"prompt": args.prompt}),
                    ("native generation", "/api/generate", {"prompt": args.prompt, "stream": False}),
                    ("chat SSE", "/v1/chat/completions", {"messages": messages, "stream": True, "stream_options": {"include_usage": True}}),
                ]:
                    payload.update(model=model.stem, max_tokens=args.max_tokens, temperature=0)
                    kind, body = request(base, path, payload)
                    if label == "chat SSE":
                        assert kind == "text/event-stream", kind
                        events = [line[6:] for line in body.splitlines() if line.startswith("data: ")]
                        assert events[-1] == "[DONE]", body
                        chunks = [json.loads(event) for event in events[:-1]]
                        assert not any("error" in chunk for chunk in chunks), chunks
                        text = "".join(chunk["choices"][0]["delta"].get("content") or "" for chunk in chunks if chunk["choices"])
                        assert chunks[-1]["choices"] == []
                        usage = chunks[-1]["usage"]
                        reason = chunks[-2]["choices"][0]["finish_reason"]
                        assert reason in ("stop", "length"), reason
                    else:
                        assert kind == "application/json", f"{label}: {kind}"
                        result = json.loads(body)
                        usage = result["usage"]
                        reason = result["finish_reason"] if path == "/api/generate" else result["choices"][0]["finish_reason"]
                        assert reason in ("stop", "length"), reason
                        if path == "/api/generate":
                            text = result["response"]
                        elif path == "/v1/completions":
                            text = result["choices"][0]["text"]
                        else:
                            text = result["choices"][0]["message"]["content"]
                    assert usage["prompt_tokens"] > 0, usage
                    assert 0 < usage["completion_tokens"] <= args.max_tokens, usage
                    assert usage["total_tokens"] == usage["prompt_tokens"] + usage["completion_tokens"], usage
                    assert text.strip(), f"{label}: empty output"
                    if args.expected:
                        assert args.expected.casefold() in text.casefold(), f"{label}: {text!r}"
                    print(f"PASS {label}: {text!r}", flush=True)
                _, limited = request(base, "/v1/chat/completions", {
                    "model": model.stem, "messages": messages, "max_tokens": 1, "temperature": 0
                })
                limited = json.loads(limited)
                assert limited["usage"]["completion_tokens"] == 1, limited
                assert limited["choices"][0]["finish_reason"] == "length", limited
                print("PASS one-token cap reports exact usage and length", flush=True)
                try:
                    request(base, "/v1/chat/completions", {
                        "model": "missing-model", "messages": messages
                    })
                    raise AssertionError("unknown model was accepted")
                except urllib.error.HTTPError as error:
                    assert error.code == 404, error.code
                    assert json.load(error)["error"]["code"] == "model_not_found"
                print("PASS unknown model returns structured 404", flush=True)
            finally:
                server.terminate()
                try:
                    server.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    server.kill()
                    server.wait()
        print(f"Server log: {log_path}", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--prompt", default="What is the capital of France? Answer briefly.")
    parser.add_argument("--expected", default="Paris")
    parser.add_argument("--max-tokens", type=int, default=32)
    parser.add_argument("--log", default="smoke-text-server.log")
    run(parser.parse_args())
