#!/usr/bin/env python3
"""Package an already-built native default-engine executable with its licenses."""
import argparse,gzip,hashlib,io,json,os,platform,shutil,subprocess,tarfile,tempfile,zipfile
from pathlib import Path
SOURCE="21c7d28381352e1cafc55e84ba941b48d2b12d3e"
VERSION="2.6.6"
OFFICIAL="registry+https://github.com/rust-lang/crates.io-index"
OWNED={"shimmy":"2.6.6","airframe":"0.4.5","libfse":"0.1.4","airframe-observe":"0.1.4","shimmytok":"0.8.5"}
FSE_SHA="bb4ad2b266c4446cec9ceeac8bd48dd18cb9e2ca0b4d3e700fbe6fee10f5a819"
NOTICE="""SHIMMY 2.6.6 DEFAULT DISTRIBUTION LICENSE NOTICE

The complete default distribution is not unrestricted Apache-2.0 software.
The default dependency chain includes Shimmy -> Airframe -> libfse.
libfse 0.1.4 uses the FSE Noncommercial Evaluation License.
Commercial use, embedding in products, and derivative works require separate permission for libfse.
Contact: michaelallenkuykendall@gmail.com

Shimmy source is Apache-2.0. Airframe and airframe-observe have MIT terms;
ShimmyTok has MIT OR Apache-2.0 terms. These do not waive libfse restrictions.
Retain the component licenses, copyright, attribution and patent notices in
LICENSES. LICENSES/index.json records the actual compiled packages and files.
No license rights are expanded by this notice.
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
def collect_licenses(metadata,selected,dest):
    byid={p["id"]:p for p in metadata["packages"]};index={}
    owner_rows={n:[byid[i] for i in selected if byid[i]["name"]==n] for n in OWNED}
    for name,rows in owner_rows.items():assert len(rows)==1 and rows[0]["version"]==OWNED[name],"wrong default component/version: "+name
    for identifier in sorted(selected):
        p=byid[identifier];name=p["name"];root=Path(p["manifest_path"]).parent
        assert p.get("source")==OFFICIAL or (name=="shimmy" and p.get("source") is None),"nonofficial dependency source"
        candidates=[]
        for file in root.iterdir():
            if file.is_file() and file.name.lower().startswith(("license","licence","copying","notice","copyright","patent")) and file.suffix.lower() not in {".rs",".py",".yml",".yaml",".json",".toml"}:candidates.append(file)
        for folder in ("LICENSES","licenses","license"):
            if (root/folder).is_dir():candidates += [p for p in (root/folder).rglob("*") if p.is_file()]
        declared=p.get("license_file")
        if declared:
            file=Path(declared);file=file if file.is_absolute() else root/file
            assert file.is_file() and file.resolve().is_relative_to(root.resolve()),"declared license-file missing/outside package"
            candidates.append(file)
        assert candidates,"no packaged license material for "+name+" "+p["version"]
        if name in OWNED:
            for filename in ("README.md","README","NOTICE","PATENTS"):
                if (root/filename).is_file():candidates.append(root/filename)
        key=name if name in OWNED else name+"@"+p["version"];files={}
        for file in sorted(set(candidates)):
            assert file.stat().st_size<2*1024*1024,"oversized license material"
            relative=Path(name+"-"+p["version"])/file.relative_to(root)
            target=dest/relative;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(file,target)
            files[relative.as_posix()]=sha(target)
        row={"name":name,"version":p["version"],"license":p.get("license"),"authors":p.get("authors",[]),"files":files}
        if name=="libfse":
            assert p.get("license") is None and declared,"libfse must retain its approved custom file"
            licensed=Path(declared);licensed=licensed if licensed.is_absolute() else root/licensed
            assert sha(licensed)==FSE_SHA,"libfse license rights changed"
            row["license_file_sha256"]=FSE_SHA
        index[key]=row
    dest.mkdir(parents=True,exist_ok=True)
    (dest/"index.json").write_text(json.dumps(index,indent=2)+"\n",encoding="utf-8")
    return index
def bundle(folder,asset,target):
    executable="shimmy.exe" if "windows" in target else "shimmy"
    entries=[(folder/asset,executable),(folder/"DISTRIBUTION-LICENSE.txt","DISTRIBUTION-LICENSE.txt"),(folder/"native-manifest.json","native-manifest.json")]
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
    license_text="""FSE Noncommercial Evaluation License

Copyright (c) 2026 Michael Kuykendall / DZERO. All rights reserved.

This source is distributed for non-commercial, evaluation, and internal research
purposes ONLY. Commercial use, embedding in products, or creation of derivative
works requires a separate commercial license.

Contact: michaelallenkuykendall@gmail.com

PATENT PENDING. This library implements inventions covered by a pending US patent
application for Fail-Closed Semantic Enforcement methods.

Portions of the graph compilation logic may utilize algorithms from aho-corasick
(MIT License, Andrew Gallant). The Execution Kernel and Policy Fusion logic are
original works subject to the patent notice above. Third-party code retains its
existing license and attribution.
"""
    assert hashlib.sha256(license_text.encode()).hexdigest()==FSE_SHA
    with tempfile.TemporaryDirectory(prefix="license-fixture-") as temp:
        root=Path(temp);packages=[]
        for name,version in OWNED.items():
            package=root/name;package.mkdir();(package/"Cargo.toml").write_text("[package]\n")
            (package/"LICENSE").write_bytes(license_text.encode() if name=="libfse" else b"Fixture MIT license\n")
            packages.append({"id":name,"name":name,"version":version,"manifest_path":str(package/"Cargo.toml"),"source":None if name=="shimmy" else OFFICIAL,"license":None if name=="libfse" else "MIT","license_file":"LICENSE" if name=="libfse" else None})
        metadata={"packages":packages};selected=set(OWNED);out=root/"out";index=collect_licenses(metadata,selected,out)
        assert index["libfse"]["license_file_sha256"]==FSE_SHA
        for label in ("changed license","missing file","missing component"):
            file=root/"libfse/LICENSE";file.write_bytes(license_text.encode())
            bad=set(selected)
            if label=="changed license":file.write_bytes(b"Commercial use now permitted")
            elif label=="missing file":file.unlink()
            else:bad.remove("libfse")
            try:collect_licenses(metadata,bad,root/label)
            except AssertionError:pass
            else:raise AssertionError("license negative escaped: "+label)
        file.write_bytes(license_text.encode())
        binary=root/"fixture-binary";binary.write_bytes(b"fixture")
        messages=json.dumps({"reason":"compiler-artifact","package_id":"shimmy","target":{"kind":["bin"]},"executable":str(binary),"features":["default","airframe","fast"]})
        try:compiled(metadata,messages,binary)
        except AssertionError:pass
        else:raise AssertionError("fast binary accepted")
    print(json.dumps({"result":"PASS","isolated_controls":["correct license copies","changed rights rejected","missing file rejected","missing component rejected","fast final binary rejected"],"native_certification":"NOT_RUN"}))
def main():
    p=argparse.ArgumentParser();p.add_argument("--self-test",action="store_true");p.add_argument("--local-preflight",action="store_true")
    for key in ("source","binary","metadata","build","output"):p.add_argument("--"+key,type=Path)
    for key in ("target","asset","run-id"):p.add_argument("--"+key)
    a=p.parse_args()
    if a.self_test:selftest();return
    assert all((a.source,a.binary,a.metadata,a.output,a.target,a.asset)),"artifact arguments missing"
    assert a.local_preflight or (a.build and a.run_id and a.run_id.isdigit() and int(a.run_id)>0),"real native run identity required"
    source=a.source.resolve();binary=a.binary.resolve()
    assert subprocess.check_output(["git","rev-parse","HEAD"],cwd=source,text=True).strip()==SOURCE,"wrong immutable source"
    subprocess.run(["git","diff","--exit-code"],cwd=source,check=True)
    host,arch,rust=host_info(a.target);metadata=json.loads(a.metadata.read_text())
    if a.local_preflight:
        selected={q["id"] for q in metadata["packages"]};native=False
    else:selected=compiled(metadata,a.build.read_text(),binary);native=True
    a.output.mkdir(parents=True,exist_ok=True);raw=a.output/a.asset;shutil.copy2(binary,raw)
    if os.name!="nt":raw.chmod(0o755)
    version=smoke(raw);index=collect_licenses(metadata,selected,a.output/"LICENSES")
    (a.output/"DISTRIBUTION-LICENSE.txt").write_text(NOTICE,encoding="utf-8")
    manifest={"source_sha":SOURCE,"source_ref":"v2.6.6","target":a.target,"host":host,"runner_arch":arch,"rustc":rust,"run_id":a.run_id or "0","native":native,"default_airframe":True,"version":version,"binary_sha256":sha(raw),"asset":a.asset,"source_lock_sha256":sha(source/"Cargo.lock"),"components":{n:index[n]["version"] for n in OWNED},"smoke":{"installed_version":"PASS","installed_help":"PASS"},"inference_certification":"NOT_RUN: Metal/DGX/native GPU model correctness"}
    (a.output/"native-manifest.json").write_text(json.dumps(manifest,indent=2)+"\n",encoding="utf-8")
    (a.output/"SHA256SUMS").write_text(sha(raw)+"  "+a.asset+"\n",encoding="utf-8")
    packed=bundle(a.output,a.asset,a.target)
    print(json.dumps({"result":"PASS","native":native,"asset":a.asset,"sha256":sha(raw),"bundle":packed.name,"bundle_sha256":sha(packed),"compiled_license_records":len(index)}))
if __name__=="__main__":main()
