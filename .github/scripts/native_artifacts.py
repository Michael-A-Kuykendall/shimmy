#!/usr/bin/env python3
"""Package an already-built native default-engine executable with its licenses."""
import argparse,gzip,hashlib,io,json,os,platform,shutil,subprocess,tarfile,tempfile,zipfile
from pathlib import Path
SOURCE="21c7d28381352e1cafc55e84ba941b48d2b12d3e"
VERSION="2.6.6"
OFFICIAL="registry+https://github.com/rust-lang/crates.io-index"
OWNED={"shimmy":"2.6.6","airframe":"0.4.5","libfse":"0.1.4","airframe-observe":"0.1.4","shimmytok":"0.8.5"}
FSE_SHA="cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30"
NOTICE="""SHIMMY 2.6.6 DEFAULT DISTRIBUTION LICENSE NOTICE

Shimmy is Apache-2.0. Airframe and airframe-observe are MIT.
ShimmyTok is MIT OR Apache-2.0. The default dependency chain
Shimmy -> Airframe -> libfse includes libfse 0.1.4 under its additional
Apache-2.0 grant, permitting commercial use, modification, redistribution
and incorporation in products without separate commercial permission.

The original published libfse archive and metadata remain unchanged.
Its earlier noncommercial restrictions are superseded to the licensor's
extent by the included LICENSE-GRANT-0.1.4.md and complete Apache license.
Grant: https://github.com/Michael-A-Kuykendall/airframe/blob/8efb1e7a830b0822cfc5ac1b121e149948d5ffe3/crates/libfse/LICENSE-GRANT-0.1.4.md

Retain LICENSES, THIRD-PARTY-NOTICES.md and applicable component copyright,
attribution, NOTICE and patent materials. This mixed permissive stack is
not a claim that every component is solely Apache-2.0. Models retain
their separate terms. Actual native compiler packages are recorded in
native-manifest.json; the licensing inventory may conservatively include
additional dependencies. Build/CLI validation is not GPU numerical certification.
"""
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def host_info(target):
    rust=subprocess.check_output(["rustc","-vV"],text=True)
    host=next(s.split(":",1)[1].strip() for s in rust.splitlines() if s.startswith("host:"))
    assert host==target,"cross-compilation is not native validation"
    arch="ARM64" if platform.machine().lower() in {"arm64","aarch64"} else "X64" if platform.machine().lower() in {"x86_64","amd64"} else "UNKNOWN"
    assert arch==("ARM64" if target.startswith("aarch64") else "X64"),"host architecture mismatch"
    if os.environ.get("RUNNER_ARCH"):assert os.environ["RUNNER_ARCH"]==arch,"runner architecture mismatch"
    assert "1.99.1" in rust,"unqualified compiler version"
    return host,arch,rust
def compiled(metadata,messages,binary):
    byid={p["id"]:p for p in metadata["packages"]}
    artifacts=[json.loads(s) for s in messages.splitlines() if s.strip().startswith("{")]
    artifacts=[a for a in artifacts if a.get("reason")=="compiler-artifact"]
    roots=[a for a in artifacts if byid[a["package_id"]]["name"]=="shimmy" and a.get("executable") and "bin" in a["target"]["kind"]]
    assert len(roots)==1 and Path(roots[0]["executable"]).resolve()==binary.resolve(),"wrong compiled executable"
    assert {"default","airframe"}<=set(roots[0]["features"]) and "fast" not in roots[0]["features"],"final binary must use default Airframe"
    selected={a["package_id"] for a in artifacts}
    assert set(OWNED)<={byid[i]["name"] for i in selected},"default component missing"
    return selected
def collect_licenses(metadata,selected,dest,target,source):
    import license_inventory
    policy=json.loads((license_inventory.HERE/"license-policy.json").read_text())
    data=license_inventory.about(source,target,dest.parent.parent,policy)
    index=license_inventory.materials(source,target,dest.parent,data,policy)
    byid={p["id"]:p for p in metadata["packages"]}
    actual={(byid[i]["name"],byid[i]["version"]) for i in selected}
    covered={(p["name"],p["version"]) for p in index["packages"]}
    assert actual<=covered,"Actual compiled packages absent from licensing inventory: "+str(actual-covered)
    return {p["name"] if p["name"] in OWNED else p["name"]+"@"+p["version"]:{"version":p["version"],"license":p["effective_license"],"files":p["license_files"]} for p in index["packages"]}

def bundle(folder,asset,target):
    executable="shimmy.exe" if "windows" in target else "shimmy"
    entries=[(folder/asset,executable),(folder/"DISTRIBUTION-LICENSE.txt","DISTRIBUTION-LICENSE.txt"),(folder/"native-manifest.json","native-manifest.json"),(folder/"THIRD-PARTY-NOTICES.md","THIRD-PARTY-NOTICES.md")]
    entries += [(p,p.relative_to(folder).as_posix()) for p in sorted((folder/"LICENSES").rglob("*")) if p.is_file()]
    if "windows" in target:
        output=folder/(Path(asset).stem+".zip")
        with zipfile.ZipFile(output,"w",compression=zipfile.ZIP_DEFLATED) as archive:
            for file,name in entries:
                info=zipfile.ZipInfo(name,date_time=(1980,1,1,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;info.external_attr=(0o755 if name==executable else 0o644)<<16;archive.writestr(info,file.read_bytes())
    else:
        output=folder/(asset+".tar.gz")
        with output.open("wb") as raw:
            with gzip.GzipFile(fileobj=raw,mode="wb",mtime=0) as zipped:
                with tarfile.open(fileobj=zipped,mode="w") as archive:
                    for file,name in entries:
                        info=tarfile.TarInfo(name);data=file.read_bytes();info.size=len(data);info.mode=0o755 if name==executable else 0o644;info.mtime=0;archive.addfile(info,io.BytesIO(data))
    (folder/"BUNDLE-SHA256SUMS").write_text(sha(output)+"  "+output.name+"\n",encoding="utf-8")
    return output
def smoke(binary):
    with tempfile.TemporaryDirectory(prefix="shimmy-install-") as temp:
        target=Path(temp)/("shimmy.exe" if os.name=="nt" else "shimmy");shutil.copy2(binary,target)
        if os.name!="nt":target.chmod(0o755)
        env=os.environ.copy();env["PATH"]=str(target.parent)+os.pathsep+env.get("PATH","")
        actual=shutil.which("shimmy",path=env["PATH"]);assert actual and Path(actual).resolve()==target.resolve(),"installed PATH resolution failed"
        version=subprocess.check_output([actual,"--version"],text=True,env=env).strip();assert version=="shimmy 2.6.6","wrong installed version"
        subprocess.run([actual,"--help"],check=True,env=env,stdout=subprocess.DEVNULL)
    return version
def selftest():
    with tempfile.TemporaryDirectory(prefix="native-fixture-") as temp:
        binary=Path(temp)/"fixture";binary.write_bytes(b"fixture")
        metadata={"packages":[{"id":name,"name":name,"version":version} for name,version in OWNED.items()]}
        artifacts=[{"reason":"compiler-artifact","package_id":name,"target":{"kind":["lib"]},"features":[]} for name in OWNED if name!="shimmy"]
        root={"reason":"compiler-artifact","package_id":"shimmy","target":{"kind":["bin"]},"executable":str(binary),"features":["default","airframe"]}
        good="\n".join(json.dumps(v) for v in artifacts+[root])
        assert compiled(metadata,good,binary)==set(OWNED)
        for label,change in (("fast final binary",lambda r:r.update(features=["fast"])),("missing Airframe",lambda r:r.update(features=["default"])),("wrong executable",lambda r:r.update(executable=str(binary)+"-wrong"))):
            bad=dict(root);change(bad);messages="\n".join(json.dumps(v) for v in artifacts+[bad])
            try:compiled(metadata,messages,binary)
            except AssertionError:pass
            else:raise AssertionError("Negative escaped: "+label)
    assert "commercial use" in NOTICE and "LICENSE-GRANT-0.1.4.md" in NOTICE
    print(json.dumps({"result":"PASS","isolated_controls":["default engine compiler facts","fast rejected","missing Airframe rejected","wrong executable rejected"],"native_certification":"NOT_RUN"}))

def main():
    p=argparse.ArgumentParser();p.add_argument("--self-test",action="store_true");p.add_argument("--local-preflight",action="store_true")
    for key in ("source","binary","metadata","build","output"):p.add_argument("--"+key,type=Path)
    for key in ("target","asset","run-id"):p.add_argument("--"+key)
    a=p.parse_args()
    if a.self_test:selftest();return
    assert all((a.source,a.binary,a.metadata,a.output,a.target,a.asset)),"artifact arguments missing"
    assert a.local_preflight or (a.build and a.run_id and a.run_id.isdigit() and int(a.run_id)>0),"real native run identity required"
    source=a.source.resolve();binary=a.binary.resolve();a.output=a.output.resolve()
    assert subprocess.check_output(["git","rev-parse","HEAD"],cwd=source,text=True).strip()==SOURCE,"wrong immutable source"
    subprocess.run(["git","diff","--exit-code"],cwd=source,check=True)
    host,arch,rust=host_info(a.target);metadata=json.loads(a.metadata.read_text())
    if a.local_preflight:
        selected={q["id"] for q in metadata["packages"]};native=False
    else:selected=compiled(metadata,a.build.read_text(),binary);native=True
    a.output.mkdir(parents=True,exist_ok=True);raw=a.output/a.asset;shutil.copy2(binary,raw)
    if os.name!="nt":raw.chmod(0o755)
    version=smoke(raw);index=collect_licenses(metadata,selected,a.output/"LICENSES",a.target,source)
    (a.output/"DISTRIBUTION-LICENSE.txt").write_text(NOTICE,encoding="utf-8")
    manifest={"source_sha":SOURCE,"source_ref":"v2.6.6","target":a.target,"host":host,"runner_arch":arch,"rustc":rust,"run_id":a.run_id or "0","native":native,"default_airframe":True,"version":version,"binary_sha256":sha(raw),"asset":a.asset,"source_lock_sha256":sha(source/"Cargo.lock"),"components":{n:index[n]["version"] for n in OWNED},"compiled_packages":sorted({p["name"]+"@"+p["version"] for p in metadata["packages"] if p["id"] in selected}),"smoke":{"installed_version":"PASS","installed_help":"PASS"},"inference_certification":"NOT_RUN: Metal/DGX/native GPU model correctness"}
    (a.output/"native-manifest.json").write_text(json.dumps(manifest,indent=2)+"\n",encoding="utf-8")
    (a.output/"SHA256SUMS").write_text(sha(raw)+"  "+a.asset+"\n",encoding="utf-8")
    packed=bundle(a.output,a.asset,a.target)
    print(json.dumps({"result":"PASS","native":native,"asset":a.asset,"sha256":sha(raw),"bundle":packed.name,"bundle_sha256":sha(packed),"compiled_license_records":len(index)}))
if __name__=="__main__":main()
