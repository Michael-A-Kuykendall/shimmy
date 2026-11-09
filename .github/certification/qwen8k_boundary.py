#!/usr/bin/env python3
"""Observe the first blocked 8K prefix with the original dense probe and inputs."""
import argparse, copy, hashlib, json, os, pathlib, platform, subprocess, sys, tempfile, traceback
from run import HERE, prepare, model_path, hardware
from verify import sha
from live_process import execute, self_test as watchdog_self_test

MODEL = "ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a"
TARGET_CHUNKS = 85

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

def prefix_self_test():
    with tempfile.TemporaryDirectory() as tmp:
        root = pathlib.Path(tmp)
        code = "import time\nfor i in range(3):\n print(f'[PREFILL] chunk={i} tokens=64 current_pos={i*64} seq_len={(i+1)*64}');print(f'[PREFILL] chunk={i} complete')\ntime.sleep(30)"
        result = execute([sys.executable,"-u","-c",code], os.environ.copy(),
            root/"prefix.log",root/"prefix.jsonl",6,2,.05,False,stop_after_chunks=3)
        assert result["reason"] == "diagnostic-prefix-completed" and result["exit"] != 0
        assert result["completed_chunks"] == result["requested_prefix_chunks"] == 3
        assert result["cleanup"][-1]["reaped"]
        bad = execute([sys.executable,"-u","-c","import time;print('[PREFILL] chunk=0 tokens=64 current_pos=0 seq_len=64');time.sleep(30)"],
            os.environ.copy(),root/"incomplete.log",root/"incomplete.jsonl",6,.4,.05,False,stop_after_chunks=1)
        assert bad["reason"] == "no-GPU-progress-timeout" and bad["completed_chunks"] == 0
        return dict(status="PASS",bounded_prefix_stop="PASS",incomplete_chunk_rejected="PASS")

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--self-test",action="store_true")
    p.add_argument("--fixtures",type=pathlib.Path,default=pathlib.Path("cert-work/frozen"))
    p.add_argument("--output",type=pathlib.Path,default=pathlib.Path("cert-work/result/qwen8k-boundary"))
    args = p.parse_args()
    job, case = original_case(args.fixtures)
    if args.self_test:
        report = watchdog_self_test()
        report["prefix"] = prefix_self_test()
        rejected=0
        for key,value in (("ids",case["ids"][:5440]),("max_tokens",1),("modes",["text"]),("prompt","altered")):
            bad=copy.deepcopy(case);bad[key]=value
            try:validate_case(bad)
            except AssertionError:rejected+=1
            else:raise AssertionError("changed original input accepted")
        assert rejected==4
        report["input_negative_controls"]=rejected
        report["original_ids"] = len(case["ids"])
        report["original_input_preserved"] = True
        print(json.dumps(report))
        return
    assert platform.system() == "Darwin" and platform.machine() == "arm64"
    root = args.output.resolve()
    root.mkdir(parents=True,exist_ok=True)
    assert not (root/"result.json").exists(),"preserve prior attempt"
    cases = root/"original-first-case.json"
    cases.write_text(json.dumps([case]))
    result = dict(status="NOT_RUN",numerical_certification="NOT_RUN",full_certification="INCOMPLETE",M4="NOT_RUN",
        host=hardware(),source_control=os.environ.get("GITHUB_SHA"),
        engine_sha=os.environ.get("CERT_ENGINE_SHA"),engine_tree=os.environ.get("CERT_ENGINE_TREE"),
        model_sha256=MODEL,context=8192,prompt_tokens=8176,max_tokens=17,mode="injected",
        original_cases_sha256=sha(args.fixtures/job["cases"]),selected_case_sha256=sha(cases),
        input_ids_sha256=hashlib.sha256(json.dumps(case["ids"],separators=(",",":")).encode()).hexdigest(),
        runtime_prefill_limit=64,qkv_microbatch=1,kv_dtype="f32",target_completed_chunks=TARGET_CHUNKS,
        last_original_completed_position=5375,first_original_blocked_positions=[5376,5439],
        per_operation_timeout_seconds=180,wall_timeout_seconds=5400)
    try:
        compute = json.loads((root.parent/"compute.json").read_text())
        assert compute["compute"] == "PASS" and compute["selected"]["backend"] == "Metal"
        assert compute["selected"]["device_type"] != "Cpu"
        result["metal_compute"] = compute
        model = model_path(MODEL,HERE.parents[1]/"cert-work/test-models")
        env = {k:v for k,v in os.environ.items() if not k.startswith(("AIRFRAME_","SHIMMY_","WGPU_","MESA_"))}
        env.update(WGPU_BACKEND="metal",SHIMMY_KV_CACHE_DTYPE="f32",SHIMMY_MAX_CTX="8192",
            AIRFRAME_CERT_LOGITS_ONLY="1",AIRFRAME_TRACE_PREFILL_CHUNKS="1",
            AIRFRAME_DIAGNOSTIC_BOUNDARIES="1",AIRFRAME_DIAGNOSTIC_BOUNDED_PREFILL="1")
        samples=[]
        def sample(pid,operation):
            index=len(samples)
            entry=dict(pid=pid,operation=operation)
            samples.append(entry)
            for label,command in [
                ("stack",["sample",str(pid),"1","1","-file",str(root/("sample-"+str(index)+".txt"))]),
                ("memory",["vm_stat"])]:
                try:
                    c=subprocess.run(command,capture_output=True,text=True,timeout=8)
                    entry[label]=dict(exit=c.returncode,stdout=c.stdout,stderr=c.stderr)
                except Exception as error:
                    entry[label]=dict(error=str(error))
            (root/"samples.json").write_text(json.dumps(samples,indent=2))
        binary=HERE/"probes/target/release/dense"
        result["binary_sha256"]=sha(binary)
        record=execute([binary,model,cases,root/"capture.json"],env,root/"diagnostic.log",root/"progress.jsonl",
            timeout=5400,stall_timeout=600,operation_timeout=180,sampler=sample,
            stop_on_native_error=True,stop_after_chunks=TARGET_CHUNKS)
        result["execution"]=record
        lines=(root/"diagnostic.log").read_text().splitlines()
        errors=[s for s in lines if s.startswith("METAL_COMMAND_ERROR ")]
        statuses=[s for s in lines if s.startswith("METAL_WAIT_STATUS ")]
        result["native_status"]=dict(errors=errors,records=len(statuses),tail=statuses[-16:])
        assert "backend=Metal" in "\n".join(lines)
        if errors:
            result.update(status="FAIL",reason="native-command-error")
        elif record["reason"] == "diagnostic-prefix-completed":
            assert record["completed_chunks"] == TARGET_CHUNKS and record["cleanup"][-1]["reaped"]
            result.update(status="PREFIX_COMPLETED",reason="original blocked boundary completed with instrumentation")
        else:
            result.update(status="BLOCKED",reason=record["reason"] or "unexpected process exit")
    except BaseException as error:
        result.update(status="FAIL",error=str(error),traceback=traceback.format_exc())
    finally:
        result["files"]=[dict(path=str(f.relative_to(root)),sha256=sha(f)) for f in sorted(root.rglob("*")) if f.is_file() and f.name!="result.json"]
        (root/"result.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps(result))
    if result["status"] != "PREFIX_COMPLETED":
        sys.exit(1)

if __name__ == "__main__":
    main()
