#!/usr/bin/env python3
"""Reject unrelated resolution drift when replacing the exact engine source tree."""
import argparse,tomllib,pathlib
def verify(before,after):
    a=tomllib.loads(before)["package"];b=tomllib.loads(after)["package"]
    official="registry+https://github.com/rust-lang/crates.io-index"
    allowed={"airframe","airframe-observe","libfse"}
    for p in a+b:
        if "source" in p:assert p["source"]==official,"unofficial registry"
    def identities(rows):return {(p["name"],p["version"]) for p in rows}
    assert identities(a)==identities(b),"dependency version drift"
    def registry(rows):return {(p["name"],p["version"],p["checksum"]) for p in rows if "source" in p}
    removed=registry(a)-registry(b);added=registry(b)-registry(a)
    assert not added and all(p[0] in allowed for p in removed),"unrelated checksum/source drift"
    paths={(p["name"],p["version"]) for p in b if "source" not in p}
    original={(p["name"],p["version"]) for p in a if "source" not in p}
    assert paths-original=={("airframe","0.4.5"),("airframe-observe","0.1.4"),("libfse","0.1.4")},"wrong substituted engine components"
    return True
def self_test():
    before='[[package]]\nname="root"\nversion="0.0.0"\n'
    for name,version in (("airframe","0.4.5"),("airframe-observe","0.1.4"),("libfse","0.1.4"),("other","1.0.0")):
        before+=f'[[package]]\nname="{name}"\nversion="{version}"\nsource="registry+https://github.com/rust-lang/crates.io-index"\nchecksum="abc"\n'
    after=before
    for name in ("airframe","airframe-observe","libfse"):
        start=after.index('[[package]]\nname="'+name+'"');end=after.find("[[package]]",start+1)
        end=len(after) if end<0 else end
        chunk=after[start:end].replace('source="registry+https://github.com/rust-lang/crates.io-index"\nchecksum="abc"\n',"")
        after=after[:start]+chunk+after[end:]
    assert verify(before,after)
    for bad in (after.replace('version="1.0.0"','version="1.1.0"'),after.replace('checksum="abc"','checksum="wrong"'),after.replace("github.com/rust-lang/crates.io-index","unofficial.example/index"),after.replace('name="libfse"','name="wrong"')):
        try:verify(before,bad)
        except AssertionError:continue
        raise AssertionError("negative control accepted")
    return 4
if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--baseline",type=pathlib.Path);p.add_argument("--candidate",type=pathlib.Path);p.add_argument("--self-test",action="store_true");a=p.parse_args()
    if a.self_test:print("LOCK_NEGATIVES_PASS",self_test())
    else:assert verify(a.baseline.read_text(),a.candidate.read_text());print("LOCK_SOURCE_ONLY_PASS")
