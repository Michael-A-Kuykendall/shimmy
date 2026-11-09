"""One bounded original-state 8K diagnostic; never numerical certification."""
import argparse,copy,hashlib,json,os,pathlib,platform,subprocess,sys,traceback
from run import HERE,prepare,model_path,hardware
from verify import sha
from live_process import execute
from flight import Flight,self_test
MODEL="ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a"

def validate_case(case):
    assert case["name"] == "qwen-context-8192-occupied-8176"
    assert len(case["ids"]) == 8176 and case["max_tokens"] == 17 and case["modes"] == ["injected"]
    assert hashlib.sha256(json.dumps(case,sort_keys=True,separators=(",",":")).encode()).hexdigest() == "7669723a14073b6ce33e59771385aaa51395ed5ddd0a8eeaa4e5dfc90ae7001a", "original case changed"
    return True

def original_case(fixtures):
    manifest = prepare(fixtures)
    job = next(j for j in manifest["jobs"] if j["name"] == "qwen8k")
    assert job["model_sha256"] == MODEL and job["context"] == 8192 and job["kind"] == "dense"
    cases = json.loads((fixtures/job["cases"]).read_text())
    assert sha(fixtures/job["cases"]) == "5e182f673a1fa379b3d465aec293619933074f6738274402df022d4d79c1bd88", "frozen cases changed"
    case = copy.deepcopy(cases[0])
    validate_case(case)
    assert case["name"] == "qwen-context-8192-occupied-8176"
    assert len(case["ids"]) == 8176 and case["max_tokens"] == 17 and case["modes"] == ["injected"]
    # Do not truncate IDs or rerender the prompt: the dense probe checks both.
    assert "prompt" in case
    return job, case

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--self-test",action="store_true")
    p.add_argument("--fixtures",type=pathlib.Path,default=pathlib.Path("cert-work/frozen"))
    p.add_argument("--output",type=pathlib.Path,default=pathlib.Path("cert-work/result/flight"))
    args=p.parse_args()
    job,case=original_case(args.fixtures)
    if args.self_test:
        result=self_test()
        rejected=0
        for key,value in (("ids",case["ids"][:5440]),("max_tokens",1),("modes",["text"]),("prompt","altered")):
            bad=copy.deepcopy(case);bad[key]=value
            try:validate_case(bad)
            except AssertionError:rejected+=1
            else:raise AssertionError("changed original input accepted")
        assert rejected==4
        print(json.dumps(dict(**result,input_negatives=rejected)));return
    assert platform.system()=="Darwin" and platform.machine()=="arm64"
    root=args.output.resolve();root.mkdir(parents=True,exist_ok=True)
    assert not (root/"result.json").exists(),"preserve previous attempt"
    result=dict(status="NOT_RUN",numerical_certification="NOT_RUN",M4="NOT_RUN",release="HOLD",
        hardware=hardware(),engine_sha=os.environ.get("CERT_ENGINE_SHA"),engine_tree=os.environ.get("CERT_ENGINE_TREE"),
        control_sha=os.environ.get("GITHUB_SHA"),model_sha256=MODEL,context=8192,prompt_tokens=8176,max_tokens=17,
        ids_sha256=hashlib.sha256(json.dumps(case["ids"],separators=(",",":")).encode()).hexdigest(),
        cases_sha256=sha(args.fixtures/job["cases"]),prefill_cap=64,qkv_microbatch=1,kv_dtype="f32",
        target_completed_chunks=85,operation_timeout=180,wall_timeout=5400,
        hot_path_logging=False,extra_GPU_readbacks=False,extra_completion_handlers=False)
    recorder=None
    try:
        compute=json.loads((root.parent/"compute.json").read_text())
        assert compute["compute"]=="PASS" and compute["selected"]["backend"]=="Metal" and compute["selected"]["device_type"]!="Cpu"
        result["compute"]=compute
        model=model_path(MODEL,HERE.parents[1]/"cert-work/test-models")
        cases=root/"case.json";cases.write_text(json.dumps([case]))
        recorder=Flight(root/"flight.bin",root/"events.jsonl")
        env={k:v for k,v in os.environ.items() if not k.startswith(("AIRFRAME_","SHIMMY_","WGPU_","MESA_"))}
        env.update(WGPU_BACKEND="metal",SHIMMY_KV_CACHE_DTYPE="f32",SHIMMY_MAX_CTX="8192",
                   AIRFRAME_CERT_LOGITS_ONLY="1",AIRFRAME_FLIGHT_PATH=str(recorder.path))
        assert "AIRFRAME_TRACE_PREFILL_CHUNKS" not in env
        binary=HERE/"probes/target/release/dense";result["binary_sha256"]=sha(binary)
        samples=[]
        def sample(pid,operation):
            entry=dict(pid=pid,operation=operation);samples.append(entry)
            for label,command in (("stack",["sample",str(pid),"1","1","-file",str(root/("stack-"+str(len(samples))+".txt"))]),
                                  ("memory",["vm_stat"]),("rss",["ps","-o","pid,ppid,rss,vsz,state,%cpu","-p",str(pid)])):
                try:
                    c=subprocess.run(command,capture_output=True,text=True,timeout=8)
                    entry[label]=dict(exit=c.returncode,stdout=c.stdout,stderr=c.stderr)
                except Exception as error:entry[label]=dict(error=str(error))
            (root/"samples.json").write_text(json.dumps(samples,indent=2))
        record=execute([binary,model,cases,root/"capture.json"],env,root/"child.log",root/"progress.jsonl",
                       timeout=5400,stall_timeout=600,operation_timeout=180,sampler=sample,
                       stop_on_native_error=True,event_source=recorder.poll,stop_after_chunks=85)
        result["execution"]=record
        recorder.poll()
        result["last_record"]=recorder.last
        result["native_statuses"]=recorder.statuses
        result["record_count"]=recorder.index
        assert recorder.index>0 and "backend=Metal" in (root/"child.log").read_text(),"missing actual runtime evidence"
        if record["reason"]=="diagnostic-prefix-completed":
            assert recorder.chunks==85 and record["cleanup"][-1]["reaped"]
            result.update(status="INCONCLUSIVE",reason="known boundary completed; no production correction justified")
        else:
            result.update(status="BLOCKED",reason=record["reason"] or "unexpected child exit")
    except BaseException as error:
        result.update(status="FAIL",reason=str(error),traceback=traceback.format_exc())
    finally:
        if recorder:recorder.close()
        result["files"]=[dict(path=str(f.relative_to(root)),sha256=sha(f)) for f in sorted(root.rglob("*")) if f.is_file() and f.name!="result.json"]
        (root/"result.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps(result))
    if result["status"]!="INCONCLUSIVE":sys.exit(1)

if __name__=="__main__":main()
