#!/usr/bin/env python3
"""Execute unchanged published inference on native Metal using frozen FP32 fixtures."""
import argparse, hashlib, json, os, pathlib, platform, shutil, subprocess, sys, tarfile, time, traceback, urllib.request
from verify import EXPECTED, compare_capture, sha, validate_matrix, falsify
HERE=pathlib.Path(__file__).resolve().parent
MODELS={
"ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a":("Qwen3-0.6B-Q4_K_M.gguf","https://huggingface.co/unsloth/Qwen3-0.6B-GGUF/resolve/50968a4468ef4233ed78cd7c3de230dd1d61a56b/Qwen3-0.6B-Q4_K_M.gguf"),
"72c5c3cb38fa32d5256e2fe30d03e7a64c6c79e668ad84057e3bd66e250b24fb":("Qwen3-1.7B-Q4_K_M.gguf","https://huggingface.co/bartowski/Qwen_Qwen3-1.7B-GGUF/resolve/9fb40622394b5dc0913453573ffed1eebee90383/Qwen_Qwen3-1.7B-Q4_K_M.gguf"),
"dde5aa3fc5ffc17176b5e8bdc82f587b24b2678c6c66101bf7da77af9f7ccdff":("Llama-3.2-3B-Q4_K_M.gguf","https://registry.ollama.ai/v2/library/llama3.2/blobs/sha256:dde5aa3fc5ffc17176b5e8bdc82f587b24b2678c6c66101bf7da77af9f7ccdff")}
def prepare(root):
    archive=HERE/"frozen.tar.gz"
    assert sha(archive)=="49ef0539592eebdd6c867fa9d168bbfcdc1910b5424c3bd127694e7194d00f53"
    root.mkdir(parents=True,exist_ok=True)
    with tarfile.open(archive,"r:gz") as tar:
        for m in tar:
            assert m.isfile() and not pathlib.PurePosixPath(m.name).is_absolute() and ".." not in pathlib.PurePosixPath(m.name).parts
        tar.extractall(root,filter="data")
    data=json.loads((root/"manifest.json").read_text())
    assert data["reference_precision"]=="FP32-dequantized-weights/F32-KV" and data["max_abs"]==0.1 and data["rms"]==0.03
    assert {j["name"]:j["comparisons"] for j in data["jobs"]}==EXPECTED
    for job in data["jobs"]:
        assert sha(root/job["cases"])==job["case_sha256"]
        for refs in job["references"].values():
            for ref in refs:assert sha(root/ref["path"])==ref["sha256"]
    return data
def model_path(expected,root):
    name,url=MODELS[expected];p=root/name;p.parent.mkdir(parents=True,exist_ok=True)
    if not p.exists():
        with urllib.request.urlopen(url,timeout=180) as source,p.open("wb") as out:shutil.copyfileobj(source,out,1024*1024)
    assert sha(p)==expected,"model checksum"
    return p
def execute(command,path,env,timeout=3600):
    start=time.monotonic()
    with path.open("w") as log:
        result=subprocess.run(list(map(str,command)),stdout=log,stderr=subprocess.STDOUT,env=env,timeout=timeout)
    return dict(command=list(map(str,command)),exit=result.returncode,elapsed=time.monotonic()-start,log_sha256=sha(path))
def hardware():
    def sysctl(name):
        return subprocess.check_output(["sysctl","-n",name],text=True).strip()
    return dict(system=platform.system(),machine=platform.machine().lower(),macos=platform.mac_ver()[0],uname=list(platform.uname()),cpu_brand=sysctl("machdep.cpu.brand_string"),hardware_model=sysctl("hw.model"),memory_bytes=int(sysctl("hw.memsize")),cpus=int(sysctl("hw.ncpu")),runner_arch=os.environ.get("RUNNER_ARCH"),runner_name=os.environ.get("RUNNER_NAME"),runner_os=os.environ.get("RUNNER_OS"))
def main():
    p=argparse.ArgumentParser();p.add_argument("--output",type=pathlib.Path);p.add_argument("--binary",type=pathlib.Path);p.add_argument("--bins",type=pathlib.Path);p.add_argument("--fixtures",type=pathlib.Path,required=True);p.add_argument("--self-test",action="store_true");a=p.parse_args()
    frozen=prepare(a.fixtures)
    if a.self_test:print(json.dumps(dict(status="PASS",controls=falsify(),jobs=list(EXPECTED))));return
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=True);bins=a.bins.resolve();active="metal_compute"
    models=out.parent/"test-models";host=hardware()
    assert host["system"]=="Darwin" and host["machine"]=="arm64" and host["runner_arch"]=="ARM64"
    env={k:v for k,v in os.environ.items() if not k.startswith(("AIRFRAME_","SHIMMY_","WGPU_","MESA_"))}
    env.update(WGPU_BACKEND="metal",RUST_LOG="info",SHIMMY_CERT_MODEL_DIR=str(models))
    result=dict(schema="apple.dense.certification.v1",host=host,control_sha=os.environ["GITHUB_SHA"],run_id=int(os.environ["GITHUB_RUN_ID"]),limits=dict(max_abs=0.1,rms=0.03),reference_precision=frozen["reference_precision"],reference_manifest_sha256=sha(a.fixtures/"manifest.json"),lock_sha256=sha(HERE/"probes/Cargo.lock"),m4="NOT_RUN",complete=False,probe={},tests={name:dict(status="NOT_RUN",reason="not yet executed") for name in (*EXPECTED,"metal_compute","http","state_reset_cancel")},commands=[],files=[])
    def save():
        result["files"]=[dict(path=str(f.relative_to(out)),sha256=sha(f)) for f in sorted(out.rglob("*")) if f.is_file() and f.name!="matrix.json"]
        (out/"matrix.json").write_text(json.dumps(result,indent=2)+"\n")
    save()
    try:
        subprocess.run(["system_profiler","SPHardwareDataType","SPDisplaysDataType","-json"],stdout=(out/"system-profiler.json").open("w"),check=True)
        # Separate exact published server HTTP diagnostic (never uploads its models).
        diag=execute([sys.executable,HERE/"metal_diagnostic.py","--binary",a.binary.resolve(),"--probe",bins/"metal","--output",out/"http"],out/"http-driver.log",env,1200)
        result["commands"].append(diag)
        if (out/"http/probe.json").exists():result["probe"]=json.loads((out/"http/probe.json").read_text())
        if result["probe"].get("status") in ("METAL_UNAVAILABLE","REQUIRED_DEVICE_UNAVAILABLE"):
            result["tests"]["metal_compute"]=dict(status="BLOCKED",reason="Runner exposed no usable Metal compute device",artifact="http/probe.json")
            raise RuntimeError("BLOCKED: no usable Metal compute device")
        assert result["probe"].get("compute")=="PASS" and result["probe"]["selected"]["backend"]=="Metal","Metal device/compute unavailable"
        result["tests"]["metal_compute"]=dict(status="PASS",artifact="http/probe.json")
        active="http"
        assert diag["exit"]==0,"HTTP diagnostic failed"
        http=json.loads((out/"http/metal-result.json").read_text())
        assert len(http["requests"])==3 and "backend=Metal" in http["adapter_log"]
        from server_contract import run as server_contract
        server_contract(a.binary.resolve(),model_path(next(iter(MODELS)),models),out/"http-contract",env)
        result["tests"]["http"]=dict(status="PASS",artifact="http/metal-result.json",protocol_contracts="http-contract/results.json");save()
        # Qwen first, then the original larger-Qwen and Llama control, then long contexts.
        order=["qwen512","qwen2k","qwen17","llama512","qwen4k","qwen8k"]
        for name in order:
            active=name
            job=next(j for j in frozen["jobs"] if j["name"]==name)
            model=model_path(job["model_sha256"],models)
            output=out/(name+".json")
            settings={**env,"SHIMMY_MAX_CTX":str(job["context"]),"AIRFRAME_CERT_LOGITS_ONLY":"1"}
            command=[bins/job["kind"],model,a.fixtures/job["cases"],output]
            if job["kind"]=="original":
                settings.update(AIRFRAME_CAPTURE_POSITION="2031",AIRFRAME_CAPTURE_FULL_LAYER="all",AIRFRAME_CAPTURE_KV_LAYERS="none")
                command.append(a.fixtures/name/"reference-0")
            print("START",name,flush=True)
            record=execute(command,out/(name+".log"),settings)
            result["commands"].append(record)
            assert "backend=Metal" in (out/(name+".log")).read_text(),"Inference did not select Metal"
            assert record["exit"]==0,(name,record["exit"])
            capture=json.loads(output.read_text())
            if job["kind"]=="original":
                assert capture["complete"] and capture["model_sha256"]==job["model_sha256"]
                case=json.loads((a.fixtures/job["cases"]).read_text())[0]
                normalized=[]
                for row in capture["rows"]:
                    assert row["mode"] in ("off","on","repeat")
                    normalized.append({**row,"case":case["name"],"encoded_ids":capture["ids"],"injected_ids":capture["ids"]})
                capture=dict(complete=True,model_sha256_at_load=capture["model_sha256"],results=normalized)
            result["tests"][name]=compare_capture(capture,job,a.fixtures)
            print("PASS",name,result["tests"][name]["comparisons"],flush=True);save()
        fixture=model_path(next(iter(MODELS)),models)
        active="state_reset_cancel"
        state=execute([bins/"state"],out/"state.log",{**env,"AIRFRAME_QWEN_FIXTURE":str(fixture)},900)
        result["commands"].append(state)
        assert state["exit"]==0 and "STATE_PROBE_PASS" in (out/"state.log").read_text() and "backend=Metal" in (out/"state.log").read_text()
        result["tests"]["state_reset_cancel"]=dict(status="PASS",artifact="state.log")
        result["complete"]=True;validate_matrix(result);save()
    except BaseException as error:
        result["failure"]=str(error);result["traceback"]=traceback.format_exc()
        if result["tests"][active]["status"]!="BLOCKED":
            result["tests"][active]=dict(status="FAIL",reason=str(error))
        save();raise
    print(json.dumps(dict(status="PASS",comparisons=198,m4="NOT_RUN")))
if __name__=="__main__":main()
