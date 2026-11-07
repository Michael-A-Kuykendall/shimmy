#!/usr/bin/env python3
"""Compare reference and experimental text inference without promoting either build."""
import argparse
import hashlib
import json
import os
import pathlib
import platform
import secrets
import socket
import statistics
import subprocess
import tempfile
import time
import urllib.error
import urllib.request

PROMPT = ("Answer using only the context below.\n\nContext:\n"
          "- Fastembed runs small ONNX embedding models in-process.\n"
          "- Shimmy is a local inference server.\n\n"
          "Question: Which tool embeds text without a server? (run {run})")

def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(8 * 1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()

def request(base, key, payload=None, timeout=2):
    body = json.dumps(payload).encode() if payload is not None else None
    path = "/v1/chat/completions" if body is not None else "/health"
    req = urllib.request.Request(base + path, data=body,
        headers={"Authorization": "Bearer " + key, "Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as response:
        return json.load(response)

def validate_profiles(rows, expected_count, require_metal=False):
    summaries = [r for r in rows if r.get("event") == "generation_summary"]
    if len(summaries) != expected_count:
        raise ValueError("missing or extra generation summaries")
    for report in summaries:
        if report["status"] != "complete" or report["contaminated"]:
            raise ValueError("failed or contaminated profile")
        if report["counter_overflow"] or report["dropped_measurements"]:
            raise ValueError("incomplete measurement counters")
        if require_metal and report.get("backend") != "Metal":
            raise ValueError("Metal required; CPU or another backend cannot certify this gate")
        measured = sum(s["totals"]["compute_passes"] for s in report["stages"])
        if measured != report["compute_pass_count"]:
            raise ValueError("stage pass coverage incomplete")
        if report["gpu_timestamp_enabled"] and any(
            s["totals"]["gpu_measured_calls"] != s["totals"]["calls"] for s in report["stages"]
        ):
            raise ValueError("GPU timestamps incomplete")
    return summaries

def compare_outputs(baseline, candidate):
    for before, after in zip(baseline, candidate, strict=True):
        for field in ("text", "usage", "finish_reason"):
            if before[field] != after[field]:
                raise ValueError(f"candidate changed greedy {field}")

def stage_totals(reports):
    totals = {}
    for report in reports:
        for stage in report["stages"]:
            key = stage["phase"] + "." + stage["stage"]
            item = totals.setdefault(key, {"gpu_ns": 0, "missing_calls": 0, "calls": 0})
            values = stage["totals"]
            item["calls"] += values["calls"]
            item["missing_calls"] += values["calls"] - values["gpu_measured_calls"]
            item["gpu_ns"] += values["gpu_ns_sum"] or 0
    for item in totals.values():
        if item["missing_calls"]:
            item["gpu_ns"] = None
    return totals

def experiment_flags(enabled, args):
    baseline_projections = not enabled and args.baseline_projections
    return {
        "AIRFRAME_EXPERIMENTAL_STAGED_FFN": "1" if enabled or args.baseline_ffn or baseline_projections else "0",
        "AIRFRAME_EXPERIMENTAL_COOPERATIVE_FFN_DOWN": "1" if (enabled and args.cooperative_down) or (not enabled and args.baseline_ffn) or baseline_projections else "0",
        "AIRFRAME_EXPERIMENTAL_COOPERATIVE_QKV": "1" if (enabled and args.cooperative_qkv) or baseline_projections else "0",
        "AIRFRAME_EXPERIMENTAL_COOPERATIVE_ATTN_PROJ": "1" if (enabled and args.cooperative_attn_proj) or baseline_projections else "0",
        "AIRFRAME_EXPERIMENTAL_VECTORIZED_FFN": "1" if enabled and args.vectorized_ffn else "0",
    }

def run_case(binary, model, directory, name, enabled, args):
    profile = directory / (name + ".profile.jsonl")
    log_path = directory / (name + ".server.log")
    if profile.exists() or log_path.exists():
        raise ValueError("use a fresh output directory; append files are not reusable evidence")
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        port = sock.getsockname()[1]
    env = {k: v for k, v in os.environ.items() if not k.startswith(("SHIMMY_", "AIRFRAME_"))}
    key = secrets.token_hex(24)
    env.update(SHIMMY_API_KEY=key, SHIMMY_WARMUP="0", NO_COLOR="1",
        AIRFRAME_TRACE_TIMING="1", AIRFRAME_TRACE_TIMING_GPU="0" if args.host_only else "1",
        AIRFRAME_TRACE_TIMING_PATH=str(profile),
        **experiment_flags(enabled, args))
    cmd = [str(binary), "--ctx", str(args.ctx), "--prefill-chunk", str(args.chunk),
           "serve", "--model-path", str(model), "--bind", f"127.0.0.1:{port}"]
    outputs = []
    base = f"http://127.0.0.1:{port}"
    started = time.perf_counter()
    with tempfile.TemporaryDirectory(prefix="shimmy-perf-") as cwd, log_path.open("w") as log:
        process = subprocess.Popen(cmd, cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + args.timeout
            while True:
                if process.poll() is not None:
                    raise RuntimeError(f"{name} server exited; inspect local server log")
                try:
                    request(base, key)
                    break
                except (urllib.error.URLError, TimeoutError):
                    if time.monotonic() >= deadline:
                        raise TimeoutError("readiness timeout")
                    time.sleep(0.1)
            startup_seconds = time.perf_counter() - started
            for index in range(args.warm_requests + 1):
                prompt = args.prompt.replace("{run}", str(index + 1))
                started = time.perf_counter()
                result = request(base, key, {"model": model.stem,
                    "messages": [{"role": "user", "content": prompt}],
                    "max_tokens": args.max_tokens, "temperature": 0, "stream": False}, args.timeout)
                elapsed = time.perf_counter() - started
                choice, usage = result["choices"][0], result["usage"]
                if not (usage["prompt_tokens"] > 0 and 0 < usage["completion_tokens"] <= args.max_tokens
                        and usage["total_tokens"] == usage["prompt_tokens"] + usage["completion_tokens"]):
                    raise ValueError("invalid generation accounting")
                text = choice["message"]["content"]
                if args.expected and args.expected.casefold() not in text.casefold():
                    raise ValueError("semantic answer check failed")
                if args.require_metal and index == 0:
                    first_rows = [json.loads(line) for line in profile.read_text().splitlines()]
                    validate_profiles(first_rows, 1, True)
                outputs.append({"text": text, "usage": usage, "finish_reason": choice["finish_reason"],
                                "wall_seconds": elapsed})
        finally:
            process.terminate()
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)
    rows = [json.loads(line) for line in profile.read_text().splitlines()]
    reports = validate_profiles(rows, len(outputs), args.require_metal)
    log = log_path.read_text()
    if log.count("[GpuRuntime] Selected adapter:") != 1:
        raise ValueError("one physical runtime load per process required")
    warm = [r["wall_seconds"] for r in outputs[1:]]
    return {"binary_sha256": sha256(binary), "startup_seconds": startup_seconds,
            "cold_request_seconds": outputs[0]["wall_seconds"],
            "warm_median_seconds": statistics.median(warm), "requests": outputs,
            "adapter": reports[0]["adapter"], "backend": reports[0]["backend"],
            "gpu_timestamps": all(r["gpu_timestamp_enabled"] for r in reports),
            "warm_stage_totals": stage_totals(reports[1:])}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline-binary", required=True)
    parser.add_argument("--candidate-binary", required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--model-sha256", required=True)
    parser.add_argument("--baseline-ref", required=True)
    parser.add_argument("--candidate-ref", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--prompt", default=PROMPT)
    parser.add_argument("--expected")
    parser.add_argument("--ctx", type=int, default=512)
    parser.add_argument("--chunk", type=int, default=8)
    parser.add_argument("--max-tokens", type=int, default=16)
    parser.add_argument("--warm-requests", type=int, default=3)
    parser.add_argument("--timeout", type=int, default=240)
    parser.add_argument("--require-metal", action="store_true")
    parser.add_argument("--host-only", action="store_true")
    parser.add_argument("--cooperative-down", action="store_true", help="also enable cooperative FFN-down in the candidate")
    parser.add_argument("--cooperative-qkv", action="store_true")
    parser.add_argument("--cooperative-attn-proj", action="store_true")
    parser.add_argument("--baseline-ffn", action="store_true", help="enable prior staged FFN and cooperative down in the baseline binary")
    parser.add_argument("--vectorized-ffn", action="store_true", help="enable packed four-value FFN dequant in candidate only")
    parser.add_argument("--baseline-projections", action="store_true", help="enable all four prior projection flags in baseline")
    args = parser.parse_args()
    if min(args.ctx, args.chunk, args.max_tokens, args.timeout) <= 0 or args.warm_requests < 3:
        parser.error("positive bounds and at least three warm requests required")
    if "{run}" not in args.prompt:
        parser.error("prompt must contain {run} to avoid repeated-response cache hits")
    model = pathlib.Path(args.model).expanduser().resolve()
    binaries = [pathlib.Path(p).expanduser().resolve() for p in
                [args.baseline_binary, args.candidate_binary]]
    if not all(p.is_file() for p in [model, *binaries]):
        parser.error("model and both binaries must exist")
    model_hash = sha256(model)
    if model_hash != args.model_sha256.lower():
        parser.error("model hash mismatch")
    output = pathlib.Path(args.output).expanduser().resolve()
    output.mkdir(parents=True, exist_ok=True)
    baseline = run_case(binaries[0], model, output, "baseline", False, args)
    candidate = run_case(binaries[1], model, output, "candidate", True, args)
    compare_outputs(baseline["requests"], candidate["requests"])
    if (baseline["backend"], baseline["adapter"]) != (candidate["backend"], candidate["adapter"]):
        raise ValueError("baseline and candidate used different adapters")
    report = {"schema": "shimmy.perf-comparison.v1", "platform": platform.platform(),
              "model_sha256": model_hash, "baseline_ref": args.baseline_ref,
              "candidate_ref": args.candidate_ref, "settings": {
                  "context": args.ctx, "prefill_chunk": args.chunk, "max_tokens": args.max_tokens,
                  "temperature": 0, "warm_requests": args.warm_requests,
                  "prompt_sha256": hashlib.sha256(args.prompt.encode()).hexdigest(),
                  "require_metal": args.require_metal, "host_only": args.host_only,
                  "candidate_staged_ffn": True, "candidate_cooperative_down": args.cooperative_down,
                  "candidate_cooperative_qkv": args.cooperative_qkv,
                  "candidate_cooperative_attn_proj": args.cooperative_attn_proj,
                  "baseline_ffn": args.baseline_ffn,
                  "baseline_projections": args.baseline_projections,
                  "candidate_vectorized_ffn": args.vectorized_ffn},
              "baseline": baseline, "candidate": candidate,
              "warm_wall_speedup": baseline["warm_median_seconds"] / candidate["warm_median_seconds"],
              "parity": True, "metal_executed": candidate["backend"] == "Metal"}
    (output / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"parity": True, "backend": candidate["backend"],
                      "warm_wall_speedup": round(report["warm_wall_speedup"], 3),
                      "comparison": str(output / "comparison.json")}))
if __name__ == "__main__":
    main()
