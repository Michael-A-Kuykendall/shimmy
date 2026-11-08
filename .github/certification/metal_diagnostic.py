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

def verified_identity(binary, candidate_checksum=None, env=None):
    env=os.environ if env is None else env
    if candidate_checksum is None:
        expected=BINARY
        identity=dict(source_sha=SOURCE,binary_sha256=expected,native_build_run=42404833070,identity_mode="published-release")
    else:
        lines=Path(candidate_checksum).read_text().splitlines()
        assert len(lines)==1 and len(lines[0].split())==2,"One build checksum required"
        expected,name=lines[0].split()
        assert len(expected)==64 and all(c in "0123456789abcdef" for c in expected),"Invalid build checksum"
        assert Path(name).name=="shimmy","Wrong binary member"
        assert env.get("CERT_ENGINE_SHA")=="3ae10b682f6b2732823ba685959393192a2492d3","Wrong candidate engine"
        assert env.get("CERT_ENGINE_TREE")=="708a53ed996e6d47811cf1aeceb98aa25d8455be","Wrong candidate source tree"
        control=env.get("GITHUB_SHA","")
        assert len(control)==40 and all(c in "0123456789abcdef" for c in control),"Missing control source"
        run=int(env.get("GITHUB_RUN_ID","0"));assert run>0,"Missing build run"
        identity=dict(source_sha=SOURCE,binary_sha256=expected,native_build_run=run,identity_mode="verified-candidate",
            engine_sha=env["CERT_ENGINE_SHA"],engine_tree=env["CERT_ENGINE_TREE"],control_sha=control,
            checksum_file_sha256=sha(candidate_checksum))
    assert sha(binary)==expected,"Wrong verified binary"
    assert env.get("SHIMMY_CERT_MODEL_DIR"),"Missing external model directory"
    return identity

def check(value,answer=False):
    assert value["object"]=="chat.completion","Expected JSON completion, not SSE"
    u=value["usage"];assert u["prompt_tokens"]>0 and u["completion_tokens"]>0,"Zero token usage"
    assert u["total_tokens"]==u["prompt_tokens"]+u["completion_tokens"],"Invalid usage"
    content=value["choices"][0]["message"]["content"]
    assert isinstance(content,str) and content.strip() and "<|im" not in content and "\ufffd" not in content,"Invalid generated content"
    if answer:
        import re
        assert re.sub(r"\s+", "", content).rstrip(".") in {"4","2+2=4"},"Generated arithmetic answer is incorrect"

def selftest():
    import copy
    good={"object":"chat.completion","usage":{"prompt_tokens":10,"completion_tokens":1,"total_tokens":11},"choices":[{"message":{"content":"4"}}]}
    check(good,True)
    expression=json.loads(json.dumps(good));expression["choices"][0]["message"]["content"]="2 + 2 = 4.";check(expression,True)
    for label,change in (("zero prompt",lambda v:v["usage"].update(prompt_tokens=0)),("wrong answer",lambda v:v["choices"][0]["message"].update(content="unrelated")),("wrong arithmetic",lambda v:v["choices"][0]["message"].update(content="2 + 2 = 5")),("SSE",lambda v:v.update(object="chat.completion.chunk"))):
        bad=copy.deepcopy(good);change(bad)
        try:check(bad,True)
        except AssertionError:pass
        else:raise AssertionError("Negative escaped: "+label)
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        binary=Path(tmp)/"shimmy";binary.write_bytes(b"candidate-binary")
        checksum=Path(tmp)/"SHA256SUMS"
        good_line=sha(binary)+"  shimmy\n";checksum.write_text(good_line)
        env=dict(CERT_ENGINE_SHA="3ae10b682f6b2732823ba685959393192a2492d3",
            CERT_ENGINE_TREE="708a53ed996e6d47811cf1aeceb98aa25d8455be",
            GITHUB_SHA="a"*40,GITHUB_RUN_ID="1",SHIMMY_CERT_MODEL_DIR=tmp)
        assert verified_identity(binary,checksum,env)["identity_mode"]=="verified-candidate"
        rejected=0
        def reject(check):
            try:check()
            except (AssertionError,ValueError):return 1
            raise AssertionError("Bad candidate identity accepted")
        rejected+=reject(lambda:verified_identity(binary,None,env))
        for key,value in (("CERT_ENGINE_SHA","0"*40),("CERT_ENGINE_TREE","0"*40),("GITHUB_SHA","invalid"),("GITHUB_RUN_ID","0"),("SHIMMY_CERT_MODEL_DIR","")):
            rejected+=reject(lambda:verified_identity(binary,checksum,{**env,key:value}))
        for line in ("0"*64+"  shimmy\n",sha(binary)+"  wrong-member\n",good_line+good_line,"invalid  shimmy\n"):
            checksum.write_text(line)
            rejected+=reject(lambda:verified_identity(binary,checksum,env))
        checksum.write_text(good_line);binary.write_bytes(b"substituted-binary")
        rejected+=reject(lambda:verified_identity(binary,checksum,env))
        assert rejected==11
    print(json.dumps({"candidate_identity_negatives":rejected,"result":"PASS","controls":["valid answer","zero prompt rejected","wrong answer rejected","SSE rejected"],"metal":"NOT_RUN"}))

def request(base,payload):
    q=urllib.request.Request(base+"/v1/chat/completions",data=json.dumps(payload).encode(),headers={"Content-Type":"application/json"})
    with urllib.request.urlopen(q,timeout=300) as r:
        assert "application/json" in r.headers.get("Content-Type",""),"Unexpected response format"
        return json.loads(r.read())

def main():
    p=argparse.ArgumentParser();p.add_argument("--self-test",action="store_true")
    for name in ("binary","probe","output"):p.add_argument("--"+name,type=Path)
    p.add_argument("--candidate-binary-sha256-file",type=Path)
    a=p.parse_args()
    if a.self_test:selftest();return
    assert all((a.binary,a.probe,a.output))
    binary=a.binary.resolve();identity=verified_identity(binary,a.candidate_binary_sha256_file)
    assert platform.system()=="Darwin" and platform.machine().lower() in {"arm64","aarch64"} and os.environ["RUNNER_ARCH"]=="ARM64","A real native Apple runner is required"
    binary.chmod(0o755)
    assert subprocess.check_output([str(binary),"--version"],text=True).strip()=="shimmy 2.6.6"
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=True)
    with (out/"system-profiler.json").open("w") as f:subprocess.run(["system_profiler","SPDisplaysDataType","-json"],stdout=f,check=True)
    with (out/"probe.stderr.log").open("w") as f:
        probe=subprocess.run([str(a.probe.resolve())],stdout=subprocess.PIPE,stderr=f,text=True,timeout=180,check=True)
    (out/"probe.json").write_text(probe.stdout)
    facts=json.loads(probe.stdout)
    result={"run_id":int(os.environ["GITHUB_RUN_ID"]),**identity,"host":{"system":platform.system(),"machine":platform.machine().lower(),"runner_arch":os.environ["RUNNER_ARCH"]},
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
            models=Path(os.environ["SHIMMY_CERT_MODEL_DIR"]);models.mkdir(parents=True,exist_ok=True)
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
                    check(value,True)
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
