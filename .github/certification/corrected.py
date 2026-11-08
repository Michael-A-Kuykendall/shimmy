#!/usr/bin/env python3
"""Exact-source Metal correction gates; partial shards never imply full certification."""
import argparse,json,os,pathlib,platform,subprocess,time,traceback
from run import HERE,MODELS,prepare,model_path,hardware
from verify import EXPECTED,compare_capture,sha,falsify
from bounded import validate_embedding
from process_control import execute as execute_group,self_test as process_self_test

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--job",choices=["qualify",*EXPECTED])
    p.add_argument("--output",type=pathlib.Path,required=True)
    p.add_argument("--fixtures",type=pathlib.Path,required=True)
    p.add_argument("--bins",type=pathlib.Path,default=HERE/"probes/target/release")
    p.add_argument("--binary",type=pathlib.Path)
    p.add_argument("--local",action="store_true")
    p.add_argument("--self-test",action="store_true")
    a=p.parse_args(); frozen=prepare(a.fixtures)
    if a.self_test:
        assert process_self_test()
        print(json.dumps(dict(status="PASS",negative_controls=falsify(),process_group_timeout="PASS",frozen_jobs=EXPECTED)))
        return
    assert a.job
    if not a.local:
        assert platform.system()=="Darwin" and platform.machine().lower()=="arm64"
    root=a.output.resolve();root.mkdir(parents=True,exist_ok=True)
    assert not (root/"matrix.json").exists(),"preserve prior attempt"
    host=hardware() if not a.local else dict(system=platform.system(),machine=platform.machine(),certification="NOT_RUN")
    bins=a.bins.resolve()
    env={k:v for k,v in os.environ.items() if not k.startswith(("AIRFRAME_","SHIMMY_","WGPU_","MESA_"))}
    env.update(WGPU_BACKEND="vulkan" if a.local else "metal",SHIMMY_KV_CACHE_DTYPE="f32",AIRFRAME_PREFILL_CHUNK="64")
    if a.local:
        env.update(MESA_D3D12_DEFAULT_ADAPTER_NAME="NVIDIA",VK_ICD_FILENAMES="/usr/share/vulkan/icd.d/dzn_icd.json",WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER="1")
    result=dict(schema="apple.dense.certification.v1",host=host,control_sha=os.environ.get("GITHUB_SHA"),
        run_id=int(os.environ.get("GITHUB_RUN_ID","0")),engine_sha=os.environ.get("CERT_ENGINE_SHA"),
        engine_tree=os.environ.get("CERT_ENGINE_TREE"),limits=dict(max_abs=0.1,rms=0.03),
        reference_precision=frozen["reference_precision"],reference_manifest_sha256=sha(a.fixtures/"manifest.json"),
        lock_sha256=sha(HERE/"probes/Cargo.lock"),m4="NOT_RUN",complete=False,shard=a.job,
        tests={name:dict(status="NOT_RUN",reason="outside this shard") for name in (*EXPECTED,"metal_compute","http","state_reset_cancel","embedding")},
        commands=[],files=[])
    def save():
        result["files"]=[dict(path=str(f.relative_to(root)),sha256=sha(f)) for f in sorted(root.rglob("*")) if f.is_file() and f.name!="matrix.json"]
        (root/"matrix.json").write_text(json.dumps(result,indent=2)+"\n")
    def execute(command,name,settings=env,timeout=900):
        log=root/(name+".log");start=time.monotonic()
        with log.open("w") as output:
            code=execute_group(list(map(str,command)),settings,output,timeout)
        result["commands"].append(dict(command=list(map(str,command)),exit=code,elapsed=time.monotonic()-start,
            timeout_seconds=timeout,log_sha256=sha(log)))
        if code==124:
            result["tests"][active]=dict(status="BLOCKED",reason="process-group execution deadline",timeout_seconds=timeout)
        assert code==0,(name,code)
        return log
    active="metal_compute";save()
    try:
        if not a.local:
            compute_log=execute([bins/"metal"],"compute",timeout=120)
            compute=json.loads(compute_log.read_text())
            assert compute["compute"]=="PASS" and compute["selected"]["backend"]=="Metal"
            assert compute["selected"]["device_type"]!="Cpu"
            result["probe"]=compute;result["tests"]["metal_compute"]=dict(status="PASS",artifact="compute.log");save()
        else:
            result["tests"]["metal_compute"]=dict(status="NOT_RUN",reason="local NVIDIA qualification is not Metal");save()
        if a.job=="qualify":
            active="embedding";model=model_path("72c5c3cb38fa32d5256e2fe30d03e7a64c6c79e668ad84057e3bd66e250b24fb",root.parent/"test-models")
            execute([bins/"embedding",model,HERE/"embedding-row-hashes.json",root/"embedding.json"],"embedding",timeout=420)
            validate_embedding(json.loads((root/"embedding.json").read_text()),json.loads((HERE/"embedding-row-hashes.json").read_text()))
            result["tests"]["embedding"]=dict(status="PASS",rows=33,artifact="embedding.json");save()
            active="state_reset_cancel";small=model_path(next(iter(MODELS)),root.parent/"test-models")
            log=execute([bins/"state"],"state",{**env,"AIRFRAME_QWEN_FIXTURE":str(small)},900)
            assert "STATE_PROBE_PASS" in log.read_text()
            result["tests"][active]=dict(status="PASS",artifact="state.log");save()
            active="http";assert a.binary and a.binary.is_file()
            result["server_binary_sha256"]=sha(a.binary)
            if not a.local:
                execute([os.sys.executable,HERE/"metal_diagnostic.py","--binary",a.binary.resolve(),"--probe",bins/"metal","--output",root/"http"],"http-driver",timeout=1200)
            from server_contract import run as server_contract
            server_contract(a.binary.resolve(),small,root/"http-contract",env)
            result["tests"][active]=dict(status="PASS",artifact="http-contract/results.json");save()
        else:
            active=a.job;job=next(j for j in frozen["jobs"] if j["name"]==active)
            model=model_path(job["model_sha256"],root.parent/"test-models")
            settings={**env,"SHIMMY_MAX_CTX":str(job["context"]),"AIRFRAME_CERT_LOGITS_ONLY":"1"}
            command=[bins/job["kind"],model,a.fixtures/job["cases"],root/(active+".json")]
            if job["kind"]=="original":
                settings.update(AIRFRAME_CAPTURE_POSITION="2031",AIRFRAME_CAPTURE_FULL_LAYER="all",AIRFRAME_CAPTURE_KV_LAYERS="none")
                command.append(a.fixtures/active/"reference-0")
            log=execute(command,active,settings,timeout=19500)
            assert ("backend=Vulkan" if a.local else "backend=Metal") in log.read_text()
            capture=json.loads((root/(active+".json")).read_text())
            if job["kind"]=="original":
                assert capture["complete"] and capture["model_sha256"]==job["model_sha256"]
                case=json.loads((a.fixtures/job["cases"]).read_text())[0]
                capture=dict(complete=True,model_sha256_at_load=capture["model_sha256"],results=[
                    {**row,"case":case["name"],"encoded_ids":capture["ids"],"injected_ids":capture["ids"]} for row in capture["rows"]])
            result["tests"][active]=compare_capture(capture,job,a.fixtures);save()
        result["shard_complete"]=True;save()
    except BaseException as error:
        if result["tests"][active]["status"]!="BLOCKED":
            result["tests"][active]=dict(status="FAIL",reason=str(error))
        result["failure"]=str(error);result["traceback"]=traceback.format_exc();save();raise
    print(json.dumps(dict(status="PASS",shard=a.job,full_certification="NOT_RUN",M4="NOT_RUN")))

if __name__=="__main__":main()
