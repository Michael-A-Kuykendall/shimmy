#!/usr/bin/env python3
"""Bounded follow-up to a captured Metal warning; no production-code changes."""
import argparse
import copy
import json
import os
import pathlib
import platform
import subprocess
import sys
import time
from run import hardware, model_path
from verify import sha

HERE = pathlib.Path(__file__).resolve().parent
SMALL = "ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a"
LARGE = "72c5c3cb38fa32d5256e2fe30d03e7a64c6c79e668ad84057e3bd66e250b24fb"

def validate_embedding(data, contract):
    assert data["complete"] is True and data["status"] == "PASS"
    assert data["model_sha256"] == LARGE
    assert len(data["rows"]) == len(contract["rows"]) == 33
    expected = {r["token_id"]: r for r in contract["rows"]}
    assert len({r["token_id"] for r in data["rows"]}) == 33
    for row in data["rows"]:
        reference = expected[row["token_id"]]
        assert row["reference_sha256"] == reference["f32_sha256"]
        assert row["pass"] is True
        assert 0 <= row["max_abs"] <= 0.00002
        assert 0 <= row["rms"] <= 0.00002
        assert len(row["actual_sha256"]) == 64

def validate_report(root):
    report = json.loads((root/"report.json").read_text())
    assert report["host"]["system"] == "Darwin"
    assert report["host"]["machine"] == "arm64"
    assert report["M4"] == "NOT_RUN"
    for stage in ("compute", "embedding", "state"):
        entry = report["tests"][stage]
        assert entry["exit"] == 0 and entry["status"] == "PASS", (stage, entry)
        assert sha(root/entry["log"]) == entry["log_sha256"]
    probe = json.loads((root/"compute.log").read_text())
    assert probe["compute"] == "PASS"
    assert probe["selected"]["backend"] == "Metal"
    assert probe["selected"]["device_type"] != "Cpu"
    assert probe["actual"] == [float(2*i+1) for i in range(32)]
    validate_embedding(json.loads((root/"embedding.json").read_text()),
                       json.loads((HERE/"embedding-row-hashes.json").read_text()))
    for stage in ("embedding", "state"):
        log = (root/report["tests"][stage]["log"]).read_text()
        assert "backend=Metal" in log
    assert "EMBEDDING_ROWS_PASS 33" in (root/"embedding.log").read_text()
    assert "STATE_PROBE_PASS" in (root/"state.log").read_text()
    return report

def self_test():
    contract = json.loads((HERE/"embedding-row-hashes.json").read_text())
    good = dict(complete=True,status="PASS",model_sha256=LARGE,rows=[
        dict(token_id=r["token_id"],reference_sha256=r["f32_sha256"],
             actual_sha256=r["f32_sha256"],max_abs=0.0,rms=0.0,pass_=True)
        for r in contract["rows"]])
    for row in good["rows"]:
        row["pass"] = row.pop("pass_")
    validate_embedding(good, contract)
    bads = []
    for kind in ("missing", "wrong_hash", "zero_row", "duplicate", "wrong_model", "nan", "incomplete"):
        bad = copy.deepcopy(good)
        if kind == "missing": bad["rows"].pop()
        if kind == "wrong_hash": bad["rows"][0]["reference_sha256"] = "0"*64
        if kind == "zero_row": bad["rows"][0]["max_abs"] = 1.0
        if kind == "duplicate": bad["rows"][0] = bad["rows"][1]
        if kind == "wrong_model": bad["model_sha256"] = SMALL
        if kind == "nan": bad["rows"][0]["rms"] = float("nan")
        if kind == "incomplete": bad["complete"] = False
        try: validate_embedding(bad, contract)
        except (AssertionError, KeyError): bads.append(kind)
        else: raise AssertionError("accepted falsification: "+kind)
    print(json.dumps(dict(clean="PASS", rejected=bads)))

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", choices=["prepare","embedding","state","verify"])
    parser.add_argument("--output", type=pathlib.Path, default=pathlib.Path("cert-work/bounded"))
    parser.add_argument("--bins", type=pathlib.Path, default=HERE/"probes/target/release")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    assert platform.system() == "Darwin" and platform.machine().lower() == "arm64"
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=True)
    record = root/"report.json"
    if args.stage == "prepare":
        assert not record.exists(), "preserve previous evidence"
        report = dict(source_sha=os.environ["GITHUB_SHA"],run_id=os.environ["GITHUB_RUN_ID"],
                      host=hardware(),M4="NOT_RUN",full_matrix="NOT_RUN",tests={})
        record.write_text(json.dumps(report, indent=2)+"\n")
        subprocess.run(["system_profiler","SPHardwareDataType","SPDisplaysDataType","-json"],
                       stdout=(root/"system-profiler.json").open("w"),check=True)
        stage = "compute"
        command = [args.bins.resolve()/"metal"]
        timeout = 120
    elif args.stage == "verify":
        validate_report(root)
        print("BOUNDED_METAL_PASS")
        return
    else:
        report = json.loads(record.read_text())
        stage = args.stage
        if stage == "embedding":
            model = model_path(LARGE, root.parent/"test-models")
            command = [args.bins.resolve()/"embedding",model,HERE/"embedding-row-hashes.json",root/"embedding.json"]
            timeout = 420
        else:
            model = model_path(SMALL, root.parent/"test-models")
            command = [args.bins.resolve()/"state"]
            timeout = 600
    env = {k:v for k,v in os.environ.items() if not k.startswith(("AIRFRAME_","SHIMMY_","WGPU_","MESA_"))}
    env.update(WGPU_BACKEND="metal",SHIMMY_KV_CACHE_DTYPE="f32",AIRFRAME_PREFILL_CHUNK="64")
    if stage == "state": env["AIRFRAME_QWEN_FIXTURE"] = str(model)
    start = time.monotonic()
    log = root/(stage+".log")
    status = "FAIL"
    try:
        with log.open("w") as out:
            result = subprocess.run(list(map(str,command)),stdout=out,stderr=subprocess.STDOUT,env=env,timeout=timeout)
        code = result.returncode
        status = "PASS" if code == 0 else "FAIL"
    except subprocess.TimeoutExpired:
        code = 124
        status = "BLOCKED"
    report["tests"][stage] = dict(command=list(map(str,command)),exit=code,status=status,
                                    elapsed=time.monotonic()-start,log=log.name,log_sha256=sha(log))
    record.write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps(report["tests"][stage]))
    assert code == 0, (stage,status)

if __name__ == "__main__":
    main()
