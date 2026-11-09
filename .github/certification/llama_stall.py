#!/usr/bin/env python3
"""One instrumented original Llama prefill; never a full-certification shortcut."""
import argparse,copy,json,os,pathlib,platform,sys,traceback
from run import HERE,prepare,model_path,hardware
from verify import metric,sha
from live_process import execute,self_test

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--self-test",action="store_true")
    parser.add_argument("--fixtures",type=pathlib.Path,default=pathlib.Path("cert-work/frozen"))
    parser.add_argument("--output",type=pathlib.Path,default=pathlib.Path("cert-work/result/llama-stall"))
    args=parser.parse_args()
    if args.self_test:
        print(json.dumps(self_test()));return
    assert platform.system()=="Darwin" and platform.machine()=="arm64"
    frozen=prepare(args.fixtures)
    job=next(j for j in frozen["jobs"] if j["name"]=="llama512")
    original=json.loads((args.fixtures/job["cases"]).read_text())
    assert len(original)==1 and original[0]["name"]=="llama-rope-long"
    root=args.output.resolve();root.mkdir(parents=True,exist_ok=True)
    assert not (root/"result.json").exists(),"preserve previous attempt"
    case=copy.deepcopy(original[0]);case.update(max_tokens=1,modes=["injected"])
    cases=root/"minimal-cases.json";cases.write_text(json.dumps([case]))
    result=dict(status="NOT_RUN",full_certification="INCOMPLETE",M4="NOT_RUN",
        host=hardware(),source_control=os.environ.get("GITHUB_SHA"),
        engine_sha=os.environ.get("CERT_ENGINE_SHA"),engine_tree=os.environ.get("CERT_ENGINE_TREE"),
        original_case_sha256=sha(args.fixtures/job["cases"]),minimal_case_sha256=sha(cases),
        model_sha256=job["model_sha256"],limits=dict(max_abs=.1,rms=.03),
        reference_precision=job["reference_precision"],per_operation_timeout_seconds=180,wall_timeout_seconds=600)
    try:
        compute=json.loads((root.parent/"compute.json").read_text())
        assert compute["compute"]=="PASS" and compute["selected"]["backend"]=="Metal"
        assert compute["selected"]["device_type"]!="Cpu"
        result["metal_compute"]=compute
        model=model_path(job["model_sha256"],HERE.parents[1]/"cert-work/test-models")
        env={k:v for k,v in os.environ.items() if not k.startswith(("AIRFRAME_","SHIMMY_","WGPU_","MESA_"))}
        env.update(WGPU_BACKEND="metal",SHIMMY_KV_CACHE_DTYPE="f32",SHIMMY_MAX_CTX="512",
            AIRFRAME_CERT_LOGITS_ONLY="1",AIRFRAME_TRACE_PREFILL_CHUNKS="1",AIRFRAME_DIAGNOSTIC_BOUNDARIES="1")
        binary=HERE/"probes/target/release/generation"
        result["binary_sha256"]=sha(binary)
        record=execute([binary,model,cases,root/"capture.json"],env,root/"diagnostic.log",root/"progress.jsonl",
            timeout=600,stall_timeout=600,operation_timeout=180)
        result["execution"]=record
        if record["exit"]:
            result.update(status="BLOCKED",reason=record["reason"] or "process failed")
            raise RuntimeError("minimal Llama did not complete; inspect last_boundary/active_operations")
        assert "backend=Metal" in (root/"diagnostic.log").read_text(),"inference did not select Metal"
        capture=json.loads((root/"capture.json").read_text())
        import struct
        assert capture["complete"] and capture["model_sha256_at_load"]==job["model_sha256"]
        assert len(capture["results"])==1
        row=capture["results"][0]
        assert row["case"]==case["name"] and row["mode"]=="injected"
        assert row["encoded_ids"]==row["injected_ids"]==case["ids"]
        assert row["prompt_tokens"]==len(case["ids"]) and row["completion_tokens"]==1
        assert len(row["logits"])==1
        ref=job["references"][case["name"]][0];path=args.fixtures/ref["path"]
        assert sha(path)==ref["sha256"]
        raw=path.read_bytes();expected=list(struct.unpack("<"+"f"*(len(raw)//4),raw))
        result["comparison"]=dict(comparisons=1,reference_sha256=ref["sha256"],**metric(row["logits"][0],expected))
        result["status"]="PASS"
    except BaseException as error:
        if result["status"]!="BLOCKED":result["status"]="FAIL"
        result["error"]=str(error);result["traceback"]=traceback.format_exc()
    finally:
        result["files"]=[dict(path=str(p.relative_to(root)),sha256=sha(p)) for p in sorted(root.rglob("*")) if p.is_file() and p.name!="result.json"]
        (root/"result.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps(result))
    if result["status"]!="PASS":sys.exit(1)

if __name__=="__main__":main()
