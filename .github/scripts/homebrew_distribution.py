#!/usr/bin/env python3
"""Native installation gate for an isolated/CI Homebrew client."""
import argparse, hashlib, json, os, pathlib, platform, subprocess
TAP="michael-a-kuykendall/shimmy"
def run(*args):
    return subprocess.check_output(args,text=True).strip()
def main():
    p=argparse.ArgumentParser();p.add_argument("--brew",default="brew");p.add_argument("--source",required=True);p.add_argument("--output",required=True);a=p.parse_args()
    os.environ.update(HOMEBREW_NO_AUTO_UPDATE="1",HOMEBREW_NO_ANALYTICS="1",HOMEBREW_NO_INSTALL_FROM_API="1",HOMEBREW_NO_INSTALL_CLEANUP="1")
    source=pathlib.Path(a.source).resolve()
    tap=pathlib.Path(run(a.brew,"--repository",TAP))
    if tap.exists():
        assert run("git","-C",str(tap),"remote","get-url","origin")==str(source)
        run(a.brew,"untap",TAP)
    run(a.brew,"tap",TAP,str(source))
    run(a.brew,"install","--formula",TAP+"/shimmy")
    run(a.brew,"test",TAP+"/shimmy")
    prefix=pathlib.Path(run(a.brew,"--prefix",TAP+"/shimmy"))
    exe=prefix/"bin/shimmy";share=prefix/"share/shimmy"
    assert "2.6.6" in run(str(exe),"--version") and "serve" in run(str(exe),"--help")
    manifest=json.loads((share/"native-manifest.json").read_text())
    assert manifest["default_airframe"] is True and manifest["source_sha"]=="21c7d28381352e1cafc55e84ba941b48d2b12d3e"
    sha=hashlib.sha256(exe.read_bytes()).hexdigest();assert sha==manifest["binary_sha256"]
    grant=share/"LICENSES/libfse-0.1.4/LICENSE-GRANT-0.1.4.md"
    assert hashlib.sha256(grant.read_bytes()).hexdigest()=="f6d6b4df28b08d1dcfe3d1b3d3d30339c7d8cfd6edbc14b6fe5c1f6818eb531f"
    assert (share/"THIRD-PARTY-NOTICES.md").stat().st_size>10000
    arch="macos-arm64" if platform.system()=="Darwin" and platform.machine()=="arm64" else "linux-x86_64" if platform.system()=="Linux" and platform.machine()=="x86_64" else None
    assert arch
    result=dict(version="2.6.6",engine="Airframe",licenses=True,grant=True,cli_test=True,architecture=arch,package_manager="Homebrew",binary_sha256=sha,formula_sha256=hashlib.sha256((source/"Formula/shimmy.rb").read_bytes()).hexdigest(),source_control=run("git","-C",str(source),"rev-parse","HEAD"),brew_version=run(a.brew,"--version"))
    output=pathlib.Path(a.output);output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result))
if __name__=="__main__":main()
