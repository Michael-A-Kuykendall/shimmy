#!/usr/bin/env python3
"""Allow exactly the read-only HAL-source substitution; preserve versions/dependencies."""
import argparse,copy,json,pathlib,tomllib
HERE=pathlib.Path(__file__).resolve().parent
def verify(original,candidate):
    a=tomllib.loads(original);b=tomllib.loads(candidate)
    expected=copy.deepcopy(a)
    matches=[p for p in expected["package"] if p["name"]=="wgpu-hal"]
    assert len(matches)==1 and matches[0]["version"]=="27.0.4","HAL identity drift"
    hal=matches[0]
    assert hal.pop("source")=="registry+https://github.com/rust-lang/crates.io-index"
    assert hal.pop("checksum")=="5b21cb61c57ee198bc4aff71aeadff4cbb80b927beb912506af9c780d64313ce"
    assert expected==b,"unrelated source/version/dependency drift"
    return dict(status="PASS",source_only="wgpu-hal27.0.4",dependency_versions="IDENTICAL")
def main():
    p=argparse.ArgumentParser();p.add_argument("--self-test",action="store_true")
    p.add_argument("--candidate",type=pathlib.Path,default=HERE/"probes/metal-status.Cargo.lock")
    args=p.parse_args();original=(HERE/"probes/diagnostic.Cargo.lock").read_text();candidate=args.candidate.read_text()
    result=verify(original,candidate)
    if args.self_test:
        for bad in [
            original,
            candidate.replace('version = "27.0.4"','version = "27.0.5"',1),
            candidate.replace('name = "wgpu-hal"','name = "substituted-hal"',1),
            candidate.replace('version = 4','version = 3',1),
            candidate.replace("registry+https://github.com/rust-lang/crates.io-index","git+https://example.invalid/unofficial",1)]:
            try:verify(original,bad)
            except (AssertionError,KeyError):pass
            else:raise AssertionError("bad graph accepted")
        result["negative_controls"]=5
    print(json.dumps(result))
if __name__=="__main__":main()
