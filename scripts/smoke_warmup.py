#!/usr/bin/env python3
"""Verify startup warm-up and repeated inference in one process; optionally validate timing JSONL."""
import argparse
import json
import os
import pathlib
import secrets
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request

def request(base, path, key, payload=None):
    data=None if payload is None else json.dumps(payload).encode()
    req=urllib.request.Request(base+path,data=data,headers={"Content-Type":"application/json","Authorization":"Bearer "+key})
    with urllib.request.urlopen(req,timeout=180) as response:
        return json.load(response)

def run_case(binary, model, logs, name, *, warmup=False, legacy=False, timing="off"):
    with socket.socket() as sock:
        sock.bind(("127.0.0.1",0))
        port=sock.getsockname()[1]
    key=secrets.token_hex(24)
    env=dict(os.environ,SHIMMY_API_KEY=key,SHIMMY_WARMUP="1" if warmup else "0",
             NO_COLOR="1",RUST_LOG="info",SHIMMY_LOG_LEVEL="info")
    for variable in ["SHIMMY_BASE_GGUF","SHIMMY_LORA_GGUF","SHIMMY_WARMUP_MODEL",
                     "AIRFRAME_TRACE_TIMING","AIRFRAME_TRACE_TIMING_GPU","AIRFRAME_TRACE_TIMING_PATH"]:
        env.pop(variable,None)
    profile=logs/(name+".jsonl")
    if timing!="off":
        profile.unlink(missing_ok=True)
        env.update(AIRFRAME_TRACE_TIMING="1",AIRFRAME_TRACE_TIMING_GPU="0" if timing=="host" else "1",AIRFRAME_TRACE_TIMING_PATH=str(profile))
    cmd=[str(binary),"--ctx","512","--prefill-chunk","8"]
    if legacy:
        cmd.append("--legacy")
    cmd+=["serve","--model-path",str(model),"--bind",f"127.0.0.1:{port}"]
    base=f"http://127.0.0.1:{port}"
    log_path=logs/(name+".log")
    with tempfile.TemporaryDirectory(prefix="shimmy-warmup-") as cwd, log_path.open("w") as log:
        process=subprocess.Popen(cmd,cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT)
        try:
            deadline=time.monotonic()+240
            while True:
                if process.poll() is not None:
                    raise RuntimeError(f"{name}: server failed; see {log_path}")
                try:
                    request(base,"/health",key)
                    break
                except (urllib.error.URLError,TimeoutError):
                    if time.monotonic()>=deadline:
                        raise RuntimeError(f"{name}: readiness timeout")
                    time.sleep(.1)
            startup=log_path.read_text()
            ready=startup.find("Ready to serve requests")
            assert ready>=0,f"{name}: missing Ready log"
            marker="[GpuRuntime] Selected adapter:"
            if warmup:
                assert 0<=startup.find(marker)<ready,f"{name}: model was not loaded before readiness"
                assert 0<=startup.find("warmup_forward_complete")<ready,f"{name}: forward missing before readiness"
                assert 0<=startup.find("warmup_complete")<ready,f"{name}: completion missing before readiness"
            else:
                assert marker not in startup,f"{name}: disabled mode loaded eagerly"
            outputs=[]
            for prompt in [
                "What is the capital of France? Answer briefly. (run 1) /no_think",
                "What is 2+2? Answer briefly. (run 2) /no_think",
                "What is the capital of France? Answer briefly. (run 3) /no_think",
            ]:
                result=request(base,"/v1/chat/completions",key,{
                    "model":model.stem,"messages":[{"role":"user","content":prompt}],
                    "max_tokens":16,"temperature":0,"stream":False,
                })
                usage=result["usage"]
                choice=result["choices"][0]
                assert usage["prompt_tokens"]>0
                assert 0<usage["completion_tokens"]<=16
                assert usage["total_tokens"]==usage["prompt_tokens"]+usage["completion_tokens"]
                outputs.append({"text":choice["message"]["content"],"finish_reason":choice["finish_reason"],"usage":usage})
            text=log_path.read_text()
            assert text.count(marker)==1,f"{name}: runtime loaded more than once"
            assert text.count("warmup_forward_complete")==int(warmup)
            if timing!="off":
                records=[json.loads(line) for line in profile.read_text().splitlines()]
                summaries=[r for r in records if r["event"]=="generation_summary"]
                assert len(summaries)==3+int(warmup),(name,len(summaries))
                assert all(r["schema"]=="airframe.profile.v1" and r["status"]=="complete" for r in summaries)
                stages={s["stage"] for r in summaries for s in r["stages"]}
                assert {"attn_norm","qkv","attn_out","attn_proj","ffn_norm","ffn_proj","ffn_down","lm_head"}<=stages
                assert all(r["compute_pass_count"]==sum(s["totals"]["compute_passes"] for s in r["stages"]) for r in summaries)
                assert all(r["dispatch_count"]>=r["compute_pass_count"] for r in summaries)
                assert all(r["submissions"]["queue_submission_count"]>0 for r in summaries)
                assert all(r["dropped_measurements"]==0 and not r["counter_overflow"] for r in summaries)
                if timing=="host":
                    assert all(not r["gpu_timestamp_enabled"] for r in summaries)
                    assert all(s["totals"]["gpu_ns_sum"] is None for r in summaries for s in r["stages"])
                else:
                    for r in summaries:
                        if r["gpu_timestamp_enabled"]:
                            for stage in r["stages"]:
                                total=stage["totals"]
                                assert total["gpu_measured_calls"]==total["calls"],(name,stage)
                assert all(not ({"prompt","text","weights","model_path"} & r.keys()) for r in records)
            return outputs
        finally:
            process.terminate()
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()

def negative_cases(binary,model,logs):
    for name,values in [("invalid-flag",{"SHIMMY_WARMUP":"invalid"}),
                        ("unknown-model",{"SHIMMY_WARMUP":"1","SHIMMY_WARMUP_MODEL":"not-a-model"}),
                        ("invalid-key",{"SHIMMY_WARMUP":"1","SHIMMY_API_KEY":"short"})]:
        env=dict(os.environ,NO_COLOR="1",SHIMMY_WARMUP="1")
        for variable in ["SHIMMY_BASE_GGUF","SHIMMY_LORA_GGUF","SHIMMY_WARMUP_MODEL"]:
            env.pop(variable,None)
        env.update(values)
        cmd=[str(binary),"serve","--model-path",str(model),"--bind","127.0.0.1:0"]
        with (logs/(name+".log")).open("w") as log:
            child=subprocess.Popen(cmd,env=env,stdout=log,stderr=subprocess.STDOUT)
            try:
                code=child.wait(timeout=15)
            except BaseException:
                child.kill()
                child.wait()
                raise
        output=(logs/(name+".log")).read_text()
        assert code!=0 and "Ready to serve requests" not in output and "[GpuRuntime] Selected adapter:" not in output,name

def run(args):
    binary=pathlib.Path(args.binary).expanduser().resolve()
    model=pathlib.Path(args.model).expanduser().resolve()
    logs=pathlib.Path(args.log_dir).expanduser().resolve()
    logs.mkdir(parents=True,exist_ok=True)
    assert binary.is_file() and model.is_file()
    baseline=run_case(binary,model,logs,"off")
    warmed=run_case(binary,model,logs,"on",warmup=True,timing=args.timing)
    assert baseline==warmed,"Warm-up changed greedy output, usage, or terminal reason"
    if not args.skip_legacy:
        legacy=run_case(binary,model,logs,"legacy",warmup=True,legacy=True,timing=args.timing)
        assert baseline==legacy,"Enhanced serving path changed output or accounting"
    negative_cases(binary,model,logs)
    print(f"PASS startup ordering, physical load reuse, A/B/A isolation, greedy parity and failure-before-readiness: {model.name} timing={args.timing}")

if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary",required=True)
    parser.add_argument("--model",required=True)
    parser.add_argument("--log-dir",required=True)
    parser.add_argument("--timing",choices=["off","gpu","host"],default="off")
    parser.add_argument("--skip-legacy",action="store_true")
    run(parser.parse_args())
