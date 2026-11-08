#!/usr/bin/env python3
"""Same binary, same host, fresh-process load-order falsification."""
import argparse,copy,json,math,os,pathlib,platform,subprocess,time
from run import hardware,model_path
from verify import sha
from bounded import validate_embedding
HERE=pathlib.Path(__file__).resolve().parent
MODEL="72c5c3cb38fa32d5256e2fe30d03e7a64c6c79e668ad84057e3bd66e250b24fb"
CASES=[
 ("original-a",None,None),("readback-all","readback","all"),("original-b",None,None),
 ("poll-after-model","poll","after-model"),("submit-after-model","submit","after-model"),
 ("wait-after-model","wait","after-model"),("submit-after-kv","submit","after-kv"),
 ("wait-after-kv","wait","after-kv"),("readback-after-model","readback","after-model"),
 ("original-c",None,None)]

def inspect_capture(data,exit_code):
    assert data["complete"] is True and data["model_sha256"]==MODEL
    contract=json.loads((HERE/"embedding-row-hashes.json").read_text())
    refs={r["token_id"]:r["f32_sha256"] for r in contract["rows"]}
    rows=data["rows"]
    assert len(rows)==33 and {r["token_id"] for r in rows}==set(refs)
    for r in rows:
        assert r["reference_sha256"]==refs[r["token_id"]]
        assert r["pass"]==(math.isfinite(r["max_abs"]) and r["max_abs"]<=0.00002)
    passed=all(r["pass"] for r in rows)
    assert data["status"]==("PASS" if passed else "FAIL")
    assert exit_code==(0 if passed else 1),"non-numerical process failure"
    if passed:validate_embedding(data,contract)
    return "PASS" if passed else "FAIL"

def validate_pairs(data):
    assert data["complete"] is True and data["model_sha256"]==MODEL and data["kind"]=="pairs"
    expected={(n,m) for n in [536870912,1073741824,1282439584] for m in ["device-ext-pair","device-ext-pair-flushed"]}
    assert len(data["results"])==6 and {(r["size"],r["method"]) for r in data["results"]}==expected
    for r in data["results"]:
        assert r["peer_payload_distinct"] is True
        groups=[r["samples"],r["peer_samples"]]
        for group in groups:
            assert len(group)==4
            assert [(s["offset"],s["size"]) for s in group]==[(0,256),(261225248,1152),(435906464,1152),(r["size"]-256,256)]
            assert all(isinstance(s["matches"],bool) for s in group)
        assert r["pass"]==all(s["matches"] for group in groups for s in group)
    return all(r["pass"] for r in data["results"])

def self_test():
    refs=json.loads((HERE/"embedding-row-hashes.json").read_text())["rows"]
    good=dict(complete=True,status="PASS",model_sha256=MODEL,rows=[dict(token_id=r["token_id"],reference_sha256=r["f32_sha256"],actual_sha256=r["f32_sha256"],max_abs=0.0,rms=0.0,**{"pass":True}) for r in refs])
    assert inspect_capture(good,0)=="PASS"
    bad=copy.deepcopy(good);bad["rows"][0]["max_abs"]=1
    try:inspect_capture(bad,0)
    except AssertionError:pass
    else:raise AssertionError("false numerical PASS")
    bad["status"]="FAIL";bad["rows"][0]["pass"]=False
    assert inspect_capture(bad,1)=="FAIL"
    for change in [lambda x:x["rows"].pop(),lambda x:x.update(model_sha256="0"*64),lambda x:x["rows"][0].update(reference_sha256="0"*64)]:
        wrong=copy.deepcopy(good);change(wrong)
        try:inspect_capture(wrong,0)
        except AssertionError:pass
        else:raise AssertionError("false provenance")
    print(json.dumps(dict(clean="PASS",false_success_rejected=4,observed_failure="FAIL")))

def main():
    p=argparse.ArgumentParser();p.add_argument("--self-test",action="store_true");p.add_argument("--local",action="store_true");p.add_argument("--pairs",action="store_true")
    p.add_argument("--model",type=pathlib.Path);p.add_argument("--baseline-binary",type=pathlib.Path);p.add_argument("--output",type=pathlib.Path,required=False,default=pathlib.Path("cert-work/ablation"))
    a=p.parse_args()
    if a.self_test:return self_test()
    if not a.local:assert platform.system()=="Darwin" and platform.machine().lower()=="arm64"
    root=a.output.resolve();root.mkdir(parents=True,exist_ok=True)
    assert not (root/"report.json").exists(),"preserve evidence"
    model=a.model.resolve() if a.model else model_path(MODEL,pathlib.Path("cert-work/test-models").resolve())
    assert sha(model)==MODEL
    binary=HERE/("probes/target/release/upload" if a.pairs else "probes/target/release/embedding")
    base={k:v for k,v in os.environ.items() if not k.startswith(("AIRFRAME_","SHIMMY_","WGPU_","MESA_"))}
    base.update(WGPU_BACKEND="vulkan" if a.local else "metal",SHIMMY_KV_CACHE_DTYPE="f32",AIRFRAME_PREFILL_CHUNK="64")
    host=dict(system=platform.system(),machine=platform.machine(),certification="NOT_RUN") if a.local else hardware()
    report=dict(host=host,control_sha=os.environ.get("GITHUB_SHA"),run_id=os.environ.get("GITHUB_RUN_ID"),binary_sha256=sha(binary),M4="NOT_RUN",full_certification="NOT_RUN",tests=[])
    if a.baseline_binary:report["baseline_binary_sha256"]=sha(a.baseline_binary)
    def save():(root/"report.json").write_text(json.dumps(report,indent=2)+"\n")
    save()
    cases=[("pairs",None,None)] if a.pairs else CASES
    if a.baseline_binary and not a.pairs:cases=[("published-baseline",None,None)]+cases
    for name,action,stage in cases:
        folder=root/name;folder.mkdir()
        env=base.copy()
        if action:env.update(AIRFRAME_METAL_LOAD_TRACE_PATH=str(folder/"load-trace.jsonl"),AIRFRAME_METAL_LOAD_ACTION=action,AIRFRAME_METAL_LOAD_STAGE=stage)
        if a.pairs:env["AIRFRAME_METAL_UPLOAD_VARIANTS"]="pairs"
        command=[str(a.baseline_binary.resolve() if name=="published-baseline" else binary),str(model),str(HERE/"embedding-row-hashes.json"),str(folder/"capture.json")]
        if not a.local:command=["/usr/bin/time","-l"]+command
        start=time.monotonic()
        with (folder/"run.log").open("w") as log:
            try:result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,env=env,timeout=600 if a.pairs else 180);code=result.returncode
            except subprocess.TimeoutExpired:code=124
        item=dict(case=name,action=action,stage=stage,command=command,exit=code,elapsed=time.monotonic()-start,log_sha256=sha(folder/"run.log"),status="BLOCKED")
        report["tests"].append(item);save()
        capture=json.loads((folder/"capture.json").read_text())
        if a.pairs:
            assert code==0
            if not a.local:assert capture["backend"]=="Metal"
            item["status"]="PASS" if validate_pairs(capture) else "FAIL"
        else:item["status"]=inspect_capture(capture,code)
        if not a.local and not a.pairs:assert "backend=Metal" in (folder/"run.log").read_text()
        item["capture_sha256"]=sha(folder/"capture.json")
        save();print(json.dumps(item),flush=True)
        if not a.local and name=="published-baseline" and item["status"]!="FAIL":
            report["controlled_reproduction"]="UNRESOLVED";report["stop"]="preserved baseline did not fail on this runner";save()
            raise AssertionError(report["stop"])
        if not a.local and name=="original-a" and a.baseline_binary and item["status"]!="FAIL":
            report["controlled_reproduction"]="UNRESOLVED";report["stop"]="diagnostic binary does not reproduce preserved baseline on same runner";save()
            raise AssertionError(report["stop"])

    if not a.pairs:
        statuses={r["case"]:r["status"] for r in report["tests"]}
        report["controlled_reproduction"]="PASS" if all(statuses[n]=="FAIL" for n in ("original-a","original-b","original-c")) and statuses["readback-all"]=="PASS" else "UNRESOLVED"
    report["complete"]=True;save()
    if not a.local and not a.pairs:assert report["controlled_reproduction"]=="PASS","same-host failure not established; no cause/fix claim"

if __name__=="__main__":main()
