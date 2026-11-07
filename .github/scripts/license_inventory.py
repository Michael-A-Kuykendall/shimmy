#!/usr/bin/env python3
"""Build redistribution notices from locked cargo-about data and exact upstream evidence."""
import argparse,gzip,hashlib,io,json,os,shutil,subprocess,tarfile,tempfile,urllib.request
from pathlib import Path
HERE=Path(__file__).resolve().parents[1]/"distribution"
OWNED={"shimmy":"2.6.6","airframe":"0.4.5","shimmytok":"0.8.5","libfse":"0.1.4","airframe-observe":"0.1.4"}
OFFICIAL="registry+https://github.com/rust-lang/crates.io-index"
def sha(data):return hashlib.sha256(data).hexdigest()
def download(row):
    data=urllib.request.urlopen(row["url"],timeout=60).read()
    assert sha(data)==row["sha256"],"Upstream license content changed: "+row["url"]
    return data
def tool(policy,target,work):
    existing=shutil.which("cargo-about")
    if existing and subprocess.check_output([existing,"--version"],text=True).strip()=="cargo-about "+policy["cargo_about"]["version"]:return existing
    entry=policy["cargo_about"]["downloads"][target];row=dict(entry,url="https://github.com/EmbarkStudios/cargo-about/releases/download/0.9.3/"+entry["asset"])
    data=download(row)
    with tarfile.open(fileobj=io.BytesIO(data),mode="r:gz") as archive:
        name="cargo-about.exe" if os.name=="nt" else "cargo-about"
        matches=[m for m in archive.getmembers() if m.isfile() and Path(m.name).name==name]
        assert len(matches)==1,"Unexpected tool archive"
        binary=work/name;binary.write_bytes(archive.extractfile(matches[0]).read());binary.chmod(0o755)
    return str(binary)
def about(source,target,work,policy):
    path=work/("about-"+target+".json")
    subprocess.run([tool(policy,target,work),"generate","--locked","--fail","--format","json","--target",target,
                    "--config",str(HERE/"about.toml"),"--output-file",str(path)],cwd=source,check=True)
    return json.loads(path.read_text())
def materials(source,target,output,data,policy):
    source=Path(source).resolve()
    assert subprocess.check_output(["git","-C",str(source),"rev-parse","HEAD"],text=True).strip()==policy["source_sha"],"Wrong immutable binary source"
    subprocess.run(["git","-C",str(source),"diff","--exit-code"],check=True)
    output=Path(output);output.mkdir(parents=True,exist_ok=True)
    assert not (output/"LICENSES/index.json").exists(),"Use a fresh licensing output directory"
    licenses=output/"LICENSES";licenses.mkdir(exist_ok=True)
    records={};ids=set()
    def emit(relative,body):
        file=output/relative;file.parent.mkdir(parents=True,exist_ok=True);file.write_bytes(body);return sha(body)
    for i,row in enumerate(data["licenses"]):
        assert row["id"] in policy["allowed_license_ids"],"Unknown/unreviewed license: "+str(row["id"])
        assert row["text"].strip(),"Missing license text"
        name="LICENSES/texts/"+str(i).zfill(3)+"-"+row["id"]+".txt"
        h=emit(name,row["text"].encode());ids.add(row["id"])
        for used in row["used_by"]:
            package=used["crate"];key=package["name"]+"@"+package["version"]
            records.setdefault(key,{})[name]=h
    import tomllib
    locked=tomllib.loads((source/"Cargo.lock").read_text())
    checksums={(p["name"],p["version"]):p["checksum"] for p in locked["package"] if p.get("source")==OFFICIAL and p.get("checksum")}
    packages=[];exceptions=[];components={}
    for row in data["crates"]:
        p=row["package"];name=p["name"];version=p["version"];key=name+"@"+version
        assert p.get("source")==OFFICIAL or (name=="shimmy" and p.get("source") is None),"Unofficial dependency source"
        root=Path(p["manifest_path"]).parent;prefix="LICENSES/"+key.replace("@","-")+"/"
        files=dict(records.get(key,{}));assert files,"No resolved licensing material: "+key
        candidates=[]
        for f in root.iterdir():
            if f.is_file() and f.name.lower().startswith(("license","licence","copying","notice","copyright","patent")) and f.suffix.lower() not in {".rs",".py",".toml",".json",".yaml",".yml"}:candidates.append(f)
        for n in ("LICENSES","licenses","license"):
            if (root/n).is_dir():candidates.extend(f for f in (root/n).rglob("*") if f.is_file())
        declared=p.get("license_file")
        if declared:
            f=Path(declared);f=f if f.is_absolute() else root/f
            assert f.is_file() and f.resolve().is_relative_to(root.resolve()),"Invalid declared license file"
            candidates.append(f)
        for f in sorted(set(candidates)):
            assert f.resolve().is_relative_to(root.resolve()) and f.stat().st_size<2*1024*1024,"Invalid license member"
            relative=prefix+f.relative_to(root).as_posix()
            if name=="libfse" and f.name=="LICENSE":relative=prefix+"HISTORICAL-ORIGINAL-LICENSE"
            files[relative]=emit(relative,f.read_bytes())
        if not candidates:
            exception=policy["exceptions"].get(key);assert exception,"Unresolved missing upstream licensing material: "+key
            assert sha((root/"Cargo.toml").read_bytes())==exception["manifest_sha256"],"Exception no longer matches upstream package"
            for evidence in exception["files"]:
                relative=prefix+"upstream/"+evidence["path"]
                files[relative]=emit(relative,download(evidence))
            statement={"package":key,"original_declared_license":p.get("license"),"manifest_sha256":exception["manifest_sha256"],
                       "authors":p.get("authors",[]),"source_archive":"https://static.crates.io/crates/"+name+"/"+name+"-"+version+".crate",
                       "reason":exception["reason"],"upstream_evidence":exception["files"]}
            if name=="shimmyjinja":statement["maintainer_attribution"]="Michael A. Kuykendall; repository maintainer and owner of the Shimmy stack"
            relative=prefix+"LICENSING-EXCEPTION.json";files[relative]=emit(relative,(json.dumps(statement,indent=2)+"\n").encode())
            exceptions.append(statement)
        if name=="libfse":
            assert version=="0.1.4" and row["license"]=="Apache-2.0","Wrong libfse release/license"
            for evidence in policy["libfse"]["files"]:
                relative=prefix+evidence["path"];files[relative]=emit(relative,download(evidence))
        if name=="airframe-observe":
            evidence=policy["observe_notice"];relative=prefix+evidence["path"];files[relative]=emit(relative,download(evidence))
        if name in OWNED:
            assert version==OWNED[name],"Wrong component version";components[name]=version
        checksum=checksums.get((name,version))
        if p.get("source")==OFFICIAL:assert checksum and len(checksum)==64,"Package absent from official locked graph"
        if (root/".cargo-checksum.json").is_file():checksum=json.loads((root/".cargo-checksum.json").read_text()).get("package")
        packages.append({"name":name,"version":version,"original_declared_license":p.get("license"),"effective_license":row["license"],
                         "authors":p.get("authors",[]),"repository":p.get("repository"),"source":p.get("source"),
                         "registry_checksum":checksum,"license_files":files,
                         "source_archive":"https://static.crates.io/crates/"+name+"/"+name+"-"+version+".crate" if p.get("source")==OFFICIAL else "https://github.com/Michael-A-Kuykendall/shimmy/tree/v2.6.6"})
    assert components==OWNED,"Missing default component"
    index={"source_sha":policy["source_sha"],"source_lock_sha256":sha((source/"Cargo.lock").read_bytes()),"target":target,
           "features":["default","airframe"],"components":components,"libfse_effective_license":"Apache-2.0",
           "inventory_scope":"Locked cargo-about normal/build dependency graph; a conservative cross-target union may include uncompiled dependencies. Actual native compiler records must be a covered subset.",
           "selected_license_ids":sorted(ids),"packages":packages,"exceptions":exceptions}
    emit("LICENSES/index.json",(json.dumps(index,indent=2)+"\n").encode())
    notice="# Shimmy 2.6.6 distribution licenses\n\nShimmy: Apache-2.0. Airframe and airframe-observe: MIT. ShimmyTok: MIT OR Apache-2.0. libfse 0.1.4: additional Apache-2.0 grant, included with its complete license and NOTICE. Commercial use is permitted subject to these terms; no separate libfse commercial permission is required.\n\nThe immutable libfse archive still contains its original historical restricted license. It is retained as HISTORICAL-ORIGINAL-LICENSE and superseded to the licensor's extent by LICENSE-GRANT-0.1.4.md. No published package or metadata was replaced.\n\nAll included dependency license texts, copyright, patent and NOTICE materials remain applicable. Packages with omitted upstream files have explicit LICENSING-EXCEPTION.json evidence; canonical texts are not represented as files originally shipped upstream. No copyright year or ownership assertion is fabricated for missing upstream notices. Authors and repository attribution are retained separately.\n\n## Source availability\n\nExact original source archives are linked below. In particular, recipients of MPL-2.0 components may obtain their corresponding unmodified source from these links under MPL-2.0. No third-party source changes are made by this distribution.\n\n"
    for p in packages:
        notice+="- "+p["name"]+" "+p["version"]+" — "+p["effective_license"]+" — [source]("+p["source_archive"]+")\n"
    notice+="\n## Explicit missing-file dispositions\n\n"
    for e in exceptions:notice+="- "+e["package"]+": "+e["reason"]+"\n"
    emit("THIRD-PARTY-NOTICES.md",notice.encode())
    return index
def pack(output,target):
    output=Path(output);archive=output.parent/(target+"-licenses.tar.gz")
    entries=[p for p in sorted(output.rglob("*")) if p.is_file()]
    with archive.open("wb") as raw,gzip.GzipFile(fileobj=raw,mode="wb",mtime=0) as zipped,tarfile.open(fileobj=zipped,mode="w") as tar:
        for file in entries:
            name=file.relative_to(output).as_posix();body=file.read_bytes();info=tarfile.TarInfo(name);info.size=len(body);info.mode=0o644;info.mtime=0;tar.addfile(info,io.BytesIO(body))
    return archive
def main():
    p=argparse.ArgumentParser();p.add_argument("--source",type=Path,required=True);p.add_argument("--target",required=True);p.add_argument("--output",type=Path,required=True);p.add_argument("--about",type=Path);a=p.parse_args()
    policy=json.loads((HERE/"license-policy.json").read_text())
    a.output.parent.mkdir(parents=True,exist_ok=True)
    data=json.loads(a.about.read_text()) if a.about else about(a.source,a.target,a.output.parent,policy)
    index=materials(a.source,a.target,a.output,data,policy);archive=pack(a.output,a.target)
    print(json.dumps({"result":"PASS","path":str(archive.resolve()),"sha256":sha(archive.read_bytes()),"packages":len(index["packages"]),"explicit_exceptions":len(index["exceptions"])}))
if __name__=="__main__":main()
