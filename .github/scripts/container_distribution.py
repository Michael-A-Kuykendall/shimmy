#!/usr/bin/env python3
"""Prepare immutable release payloads and test real native container boundaries."""
import argparse, hashlib, json, os, pathlib, platform, secrets, socket, subprocess, tarfile, time, urllib.error, urllib.request
VALUES={
"amd64":("x86_64","13c5ea0b7c32e7021542a94ad961347b6b2a86deb834a1652f6e93e0e7204cf9","a22a6bd314eb26ac888ff2aa69e8306e975f504f90cf4bc20a2f8bf6194580d4"),
"arm64":("arm64","9916f1279d1a2d3eced746836fecbbe277f4816d21e06da45ebc6596f22f812e","5387a053d6be0d6562d797173f617a2390894d6076689eaf0cb06c8a68f9287a")}
def run(*args):
    return subprocess.check_output(args,text=True).strip()
def prepare(arch,root,archive=None):
    suffix,archive_sha,binary_sha=VALUES[arch]
    root.mkdir(parents=True,exist_ok=True)
    blob=pathlib.Path(archive).read_bytes() if archive else urllib.request.urlopen("https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.6/shimmy-linux-"+suffix+".tar.gz",timeout=120).read()
    assert hashlib.sha256(blob).hexdigest()==archive_sha
    bundle=root/"bundle.tar.gz";bundle.write_bytes(blob)
    payload=root/"payload";payload.mkdir()
    with tarfile.open(bundle,"r:gz") as tar:
        for member in tar.getmembers():
            assert member.isfile() or member.isdir()
            assert not pathlib.PurePosixPath(member.name).is_absolute() and ".." not in pathlib.PurePosixPath(member.name).parts
        tar.extractall(payload,filter="data")
    assert hashlib.sha256((payload/"shimmy").read_bytes()).hexdigest()==binary_sha
    assert (payload/"LICENSES/libfse-0.1.4/LICENSE-GRANT-0.1.4.md").is_file()
    assert (payload/"THIRD-PARTY-NOTICES.md").stat().st_size>10000
    return binary_sha
def request(port,path,key=None):
    req=urllib.request.Request("http://127.0.0.1:"+str(port)+path,headers={"Authorization":"Bearer "+key} if key else {})
    try:
        with urllib.request.urlopen(req,timeout=3) as response:return response.status,response.read()
    except urllib.error.HTTPError as exc:return exc.code,exc.read()
def test(image,arch,out,model):
    native={"x86_64":"amd64","aarch64":"arm64","arm64":"arm64"}[platform.machine().lower()]
    assert native==arch
    inspected=json.loads(run("docker","image","inspect",image))[0]
    assert inspected["Architecture"]==arch and inspected["Config"]["User"]=="1001:1001"
    assert "2.6.6" in run("docker","run","--rm",image,"--version")
    assert "serve" in run("docker","run","--rm",image,"--help")
    sha=run("docker","run","--rm","--entrypoint","sha256sum",image,"/usr/local/bin/shimmy").split()[0]
    assert sha==VALUES[arch][2]
    shell="test -s /usr/local/share/shimmy/THIRD-PARTY-NOTICES.md && test -s /usr/local/share/shimmy/LICENSES/libfse-0.1.4/LICENSE-GRANT-0.1.4.md && test -s /usr/share/doc/libc6/copyright"
    run("docker","run","--rm","--entrypoint","sh",image,"-ec",shell)
    packages=run("docker","run","--rm","--entrypoint","dpkg-query",image,"-W")
    out=pathlib.Path(out);out.parent.mkdir(parents=True,exist_ok=True);out.with_suffix(".packages.txt").write_text(packages+"\n")
    refused=subprocess.run(["docker","run","--rm",image],capture_output=True,text=True,timeout=20)
    assert refused.returncode!=0 and "SHIMMY_API_KEY" in refused.stderr+refused.stdout
    model=pathlib.Path(model).resolve()
    assert hashlib.sha256(model.read_bytes()).hexdigest()=="ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a"
    key=secrets.token_hex(24);name="shimmy-gate-"+secrets.token_hex(6)
    with socket.socket() as sock:sock.bind(("127.0.0.1",0));port=sock.getsockname()[1]
    try:
        run("docker","run","-d","--name",name,"-p","127.0.0.1:"+str(port)+":11435","-e","SHIMMY_API_KEY="+key,"--mount","type=bind,src="+str(model)+",dst=/app/models/Qwen3-0.6B-Q4_K_M.gguf,readonly",image)
        health=0
        for _ in range(30):
            assert run("docker","inspect","--format","{{.State.Running}}",name)=="true", "server exited: "+run("docker","logs",name)
            try:health,_=request(port,"/health")
            except (OSError,urllib.error.URLError):pass
            if health==200:break
            time.sleep(1)
        assert health==200
        anonymous,_=request(port,"/v1/models")
        wrong,_=request(port,"/v1/models","incorrect-key-value")
        authorized,body=request(port,"/v1/models",key)
        assert anonymous==wrong==401 and authorized==200
        assert isinstance(json.loads(body)["data"],list)
        result=dict(architecture=arch,binary_sha256=sha,user="1001:1001",version="2.6.6",license_grant=True,third_party_notices=True,os_notices=True,native=True,missing_key_rejected=True,health=health,anonymous=anonymous,wrong_key=wrong,authorized=authorized,image_id=inspected["Id"],base_digest="sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc",source="21c7d28381352e1cafc55e84ba941b48d2b12d3e")
        out.write_text(json.dumps(result,indent=2)+"\n")
    finally:subprocess.run(["docker","rm","-f",name],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
def main():
    parser=argparse.ArgumentParser();parser.add_argument("mode",choices=["prepare","test"]);parser.add_argument("--arch",required=True,choices=VALUES);parser.add_argument("--context");parser.add_argument("--archive");parser.add_argument("--image");parser.add_argument("--output");parser.add_argument("--model");a=parser.parse_args()
    if a.mode=="prepare":print(prepare(a.arch,pathlib.Path(a.context),a.archive))
    else:test(a.image,a.arch,a.output,a.model);print("PASS native container startup/auth/licenses")
if __name__=="__main__":main()
