#!/usr/bin/env python3
"""One native Apple capability/model diagnostic; never claims numerical certification."""
import argparse,hashlib,json,os,platform,socket,subprocess,time,urllib.request
from pathlib import Path
BINARY="4c4ddfae22876e041b34f2abcf77301187dc4ff6f835ea10bba818cb1a8f8a1a"
MODEL="ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a"
MODEL_URL="https://huggingface.co/unsloth/Qwen3-0.6B-GGUF/resolve/50968a4468ef4233ed78cd7c3de230dd1d61a56b/Qwen3-0.6B-Q4_K_M.gguf"
SOURCE="21c7d28381352e1cafc55e84ba941b48d2b12d3e"
def sha(path):
    h=hashlib.sha256()
    with Path(path).open("rb") as f:
        for chunk in iter(lambda:f.read(1024*1024),b""):h.update(chunk)
    return h.hexdigest()
def check(value,answer=False):
    assert value["object"]=="chat.completion","Expected JSON completion, not SSE"
    u=value["usage"];assert u["prompt_tokens"]>0 and u["completion_tokens"]>0,"Zero token usage"
    assert u["total_tokens"]==u["prompt_tokens"]+u["completion_tokens"],"Invalid usage"
    content=value["choices"][0]["message"]["content"]
    assert isinstance(content,str) and content.strip() and "<|im" not in content and "\ufffd" not in content,"Invalid generated content"
    if answer:assert content.strip().strip(".")=="4","Controlled generated arithmetic answer is not4"
def selftest():
    import copy
    good={"object":"chat.completion","usage":{"prompt_tokens":10,"completion_tokens":1,"total_tokens":11},"choices":[{"message":{"content":"4"}}]}
    check(good,True)
    for label,change in (("zero prompt",lambda v:v["usage"].update(prompt_tokens=0)),("wrong answer",lambda v:v["choices"][0]["message"].update(content="unrelated")),("SSE",lambda v:v.update(object="chat.completion.chunk"))):
        bad=copy.deepcopy(good);change(bad)
        try:check(bad,True)
        except AssertionError:pass
        else:raise AssertionError("Negative escaped: "+label)
    print(json.dumps({"result":"PASS","controls":["valid answer","zero prompt rejected","wrong answer rejected","SSE rejected"],"metal":"NOT_RUN"}))
def request(base,payload):
    q=urllib.request.Request(base+"/v1/chat/completions",data=json.dumps(payload).encode(),headers={"Content-Type":"application/json"})
    with urllib.request.urlopen(q,timeout=300) as r:
        assert "application/json" in r.headers.get("Content-Type",""),"Unexpected response format"
        return json.loads(r.read())
def main():
    p=argparse.ArgumentParser();p.add_argument("--self-test",action="store_true")
    for name in ("binary","probe","output"):p.add_argument("--"+name,type=Path)
    a=p.parse_args()
    if a.self_test:selftest();return
    assert all((a.binary,a.probe,a.output))
    assert platform.system()=="Darwin" and platform.machine().lower() in {"arm64","aarch64"} and os.environ["RUNNER_ARCH"]=="ARM64","A real native Apple runner is required"
    binary=a.binary.resolve();binary.chmod(0o755);assert sha(binary)==BINARY,"Wrong verified binary"
    assert subprocess.check_output([str(binary),"--version"],text=True).strip()=="shimmy 2.6.6"
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=True)
    with (out/"system-profiler.json").open("w") as f:subprocess.run(["system_profiler","SPDisplaysDataType","-json"],stdout=f,check=True)
    with (out/"probe.stderr.log").open("w") as f:
        probe=subprocess.run([str(a.probe.resolve())],stdout=subprocess.PIPE,stderr=f,text=True,timeout=180,check=True)
    (out/"probe.json").write_text(probe.stdout)
    facts=json.loads(probe.stdout)
    result={"run_id":int(os.environ["GITHUB_RUN_ID"]),"source_sha":SOURCE,"binary_sha256":BINARY,
            "native_build_run":42404833070,"host":{"system":platform.system(),"machine":platform.machine().lower(),"runner_arch":os.environ["RUNNER_ARCH"]},
            "probe":facts,"model":{"expected_sha256":MODEL,"status":"NOT_RUN"},"requests":[],
            "numerical":{"status":"NOT_RUN","reason":"Build/capability/HTTP checks are not full-vector comparison against a frozen independent reference."},
            "m4_max_certification":"NOT_RUN","m2_performance":"NOT_RUN","adapter_log":""}
    process=None;error=None
    try:
        if facts["status"] in {"METAL_UNAVAILABLE","REQUIRED_DEVICE_UNAVAILABLE"}:
            result["model"]["reason"]="Runner has no usable Metal device for the release engine's required limits; physical-Mac follow-up required."
        else:
            assert facts["status"]=="COMPUTE_PASS" and facts["compute"]=="PASS","Actual GPU compute failed"
            assert facts["selected"]["backend"]=="Metal"
            models=out.parent/"diagnostic-model";models.mkdir(exist_ok=True)
            model=models/"Qwen3-0.6B-Q4_K_M.gguf"
            if not model.exists():
                with urllib.request.urlopen(MODEL_URL,timeout=120) as r,model.open("wb") as f:
                    while chunk:=r.read(1024*1024):f.write(chunk)
            assert sha(model)==MODEL,"Wrong GGUF fixture"
            result["model"].update(status="HASH_VERIFIED",actual_sha256=MODEL,url=MODEL_URL,size=model.stat().st_size)
            with socket.socket() as s:s.bind(("127.0.0.1",0));port=s.getsockname()[1]
            env={k:v for k,v in os.environ.items() if not k.startswith("AIRFRAME_EXPERIMENTAL") and k not in {"SHIMMY_DISABLE_OUTPUT_NORM","SHIMMY_ROUTE_V2_LAYER_PARAMS","AIRFRAME_INT4_KV","AIRFRAME_KV_INT4"}}
            env.update(RUST_LOG="info",WGPU_BACKEND="metal")
            log=out/"server.log"
            with log.open("w") as server_log:
                process=subprocess.Popen([str(binary),"--ctx","2048","serve","--model-path",str(model),"--bind","127.0.0.1:"+str(port)],stdout=server_log,stderr=subprocess.STDOUT,env=env)
                base="http://127.0.0.1:"+str(port);deadline=time.monotonic()+90
                while True:
                    if process.poll() is not None:raise RuntimeError("Server exited: "+str(process.returncode))
                    try:
                        with urllib.request.urlopen(base+"/health",timeout=2) as health:
                            if health.status==200:break
                    except OSError:
                        if time.monotonic()>deadline:raise TimeoutError("Server did not become healthy")
                        time.sleep(0.25)
                payload={"model":"Qwen3-0.6B-Q4_K_M","messages":[{"role":"user","content":"What is 2+2? Answer with one number."}],"max_tokens":30,"temperature":0,"stream":False}
                for case in ("reporter","omitted_stream","controlled_no_think"):
                    q=json.loads(json.dumps(payload))
                    if case=="omitted_stream":q.pop("stream")
                    if case=="controlled_no_think":q["messages"][0]["content"]+=" /no_think";q["max_tokens"]=64
                    started=time.monotonic();value=request(base,q)
                    result["requests"].append({"case":case,"request":q,"response":value,"elapsed_seconds":time.monotonic()-started})
                    check(value,case=="controlled_no_think")
            result["adapter_log"]=log.read_text()
            assert "backend=Metal" in result["adapter_log"],"Inference selected another backend"
    except BaseException as e:
        error=e;result["failure"]=str(e)
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            try:process.wait(timeout=20)
            except subprocess.TimeoutExpired:process.kill();process.wait()
        (out/"metal-result.json").write_text(json.dumps(result,indent=2)+"\n")
    if error is not None:raise error
    print(json.dumps({"diagnostic":facts["status"],"model":result["model"]["status"],"numerical":"NOT_RUN"}))
if __name__=="__main__":main()
