#!/usr/bin/env python3
"""Capture valid boundary experiments; never treat capture as certification."""
import argparse, copy, json, math, os, pathlib, platform, subprocess, time
from run import hardware, model_path
from verify import sha
from verify_boundary_lock import verify_locks
HERE=pathlib.Path(__file__).resolve().parent
MODEL="72c5c3cb38fa32d5256e2fe30d03e7a64c6c79e668ad84057e3bd66e250b24fb"
OPS=("original","after_wait","whole_binding","prefix_binding","bounded_binding","gpu_copied_row","cpu_uploaded_row")

def validate(data):
    assert data["complete"] is True and data["model_sha256"]==MODEL
    contract=json.loads((HERE/"embedding-row-hashes.json").read_text())
    expected={r["token_id"]:r["f32_sha256"] for r in contract["rows"]}
    assert [r["token"] for r in data["rows"]]==[11,151644]
    for row,offset in zip(data["rows"],[261225248,435906464]):
        assert row["absolute_byte_offset"]==offset and row["reference_sha256"]==expected[row["token"]]
        assert row["count"]==2048 and row["row_bytes"]==1152 and row["workgroups"]==32
        assert row["local_byte_offset"]+row["chunk"]*row["chunk_words"]*4==offset
        assert row["buffer_size"]>=row["local_byte_offset"]+1152
        assert len(row["raw_cpu_sha256"])==64
        assert isinstance(row["raw_copy_matches"],bool)
        for field in ("sentinel_written","bytes_match"): assert isinstance(row["identity"][field],bool)
        for field in ("reported_array_words","reported_base","reported_count"): assert isinstance(row["identity"][field],int)
        assert len(row["read_blob_checkpoint"]["params"])==5
        assert isinstance(row["read_blob_checkpoint"]["bytes_match"],bool)
        for op in OPS:
            metric=row[op]
            assert isinstance(metric["finite"],bool) and isinstance(metric["pass"],bool)
            assert 0<=metric["zeros"]<=2048 and len(metric["first4"])==4
            if metric["finite"]:
                assert all(isinstance(metric[k],(int,float)) and math.isfinite(metric[k]) and metric[k]>=0 for k in ("max_abs","rms"))
                assert metric["pass"]==(metric["max_abs"]<=0.00002)
            else: assert metric["pass"] is False
    return all(r["original"]["pass"] for r in data["rows"])

def self_test():
    assert verify_locks()
    clean=json.loads((HERE/"boundary-clean-fixture.json").read_text())
    assert validate(clean)
    mutations=[
      lambda x:x.update(complete=False),
      lambda x:x.update(model_sha256="0"*64),
      lambda x:x["rows"][0].update(absolute_byte_offset=0),
      lambda x:x["rows"][0].update(reference_sha256="0"*64),
      lambda x:x["rows"][0].update(workgroups=1),
      lambda x:x["rows"][0].pop("raw_copy_matches"),
      lambda x:x["rows"][0]["identity"].pop("sentinel_written"),
      lambda x:x["rows"][0]["read_blob_checkpoint"].update(params=[]),
      lambda x:x["rows"][0]["original"].update(max_abs=1,pass_=True),
    ]
    for mutate in mutations:
        bad=copy.deepcopy(clean);mutate(bad)
        try: validate(bad)
        except (AssertionError,KeyError): pass
        else: raise AssertionError("accepted falsification")
    bad=copy.deepcopy(clean)
    for r in bad["rows"]:r["original"].update(max_abs=0.2,rms=0.04,zeros=2048,**{"pass":False})
    assert validate(bad) is False, "observed numerical failure must remain FAIL"
    print(json.dumps(dict(clean="PASS",rejected=len(mutations),numerical_failure="FAIL")))

def main():
    p=argparse.ArgumentParser();p.add_argument("--self-test",action="store_true")
    p.add_argument("--output",type=pathlib.Path,default=pathlib.Path("cert-work/boundaries"))
    a=p.parse_args()
    if a.self_test:return self_test()
    assert platform.system()=="Darwin" and platform.machine().lower()=="arm64"
    root=a.output.resolve();root.mkdir(parents=True,exist_ok=True)
    record=root/"report.json";assert not record.exists(),"preserve previous evidence"
    report=dict(host=hardware(),control_sha=os.environ["GITHUB_SHA"],run_id=os.environ["GITHUB_RUN_ID"],engine_base="366c8aa429116cf305e04c9dde182a842040b4b1",patch_sha256=sha(HERE/"engine-diagnostic.patch"),M4="NOT_RUN",full_certification="NOT_RUN",tests={})
    record.write_text(json.dumps(report,indent=2)+"\n")
    subprocess.run(["system_profiler","SPHardwareDataType","SPDisplaysDataType","-json"],stdout=(root/"system-profiler.json").open("w"),check=True)
    model=model_path(MODEL,root.parent/"test-models")
    command=[HERE/"probes/target/release/boundaries",model,HERE/"embedding-row-hashes.json",root/"boundaries.json"]
    env={k:v for k,v in os.environ.items() if not k.startswith(("AIRFRAME_","SHIMMY_","WGPU_","MESA_"))}
    env.update(WGPU_BACKEND="metal",SHIMMY_KV_CACHE_DTYPE="f32",AIRFRAME_PREFILL_CHUNK="64")
    start=time.monotonic();code=124;status="BLOCKED"
    try:
        with (root/"boundaries.log").open("w") as log:
            code=subprocess.run(list(map(str,command)),env=env,stdout=log,stderr=subprocess.STDOUT,timeout=600).returncode
        status="PASS" if code==0 else "FAIL"
    except subprocess.TimeoutExpired:pass
    report["tests"]["capture"]=dict(status=status,exit=code,command=list(map(str,command)),elapsed=time.monotonic()-start,log_sha256=sha(root/"boundaries.log"))
    record.write_text(json.dumps(report,indent=2)+"\n")
    assert code==0
    assert "backend=Metal" in (root/"boundaries.log").read_text()
    data=json.loads((root/"boundaries.json").read_text());numerical=validate(data)
    report["tests"]["embedding"]=dict(status="PASS" if numerical else "FAIL",rows=2)
    record.write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps(report,indent=2))
    assert numerical,"original production embedding still FAIL; checkpoints preserved"

if __name__=="__main__":main()
