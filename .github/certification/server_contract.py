"""Run the unchanged existing JSON/SSE/usage/stop/cancel assertions against the release."""
import json, os, pathlib, socket, subprocess, sys, time, urllib.request
HERE=pathlib.Path(__file__).resolve().parent
def run(binary,model,out,env,expected_backend="Metal"):
    assert expected_backend in ("Metal","Vulkan")
    out.mkdir(parents=True,exist_ok=True)
    with socket.socket() as sock:sock.bind(("127.0.0.1",0));port=sock.getsockname()[1]
    base="http://127.0.0.1:"+str(port)
    process=None
    with (out/"server.log").open("w") as log:
        try:
            process=subprocess.Popen([str(binary),"--ctx","512","serve","--model-path",str(model),"--bind","127.0.0.1:"+str(port)],stdout=log,stderr=subprocess.STDOUT,env={**env,"SHIMMY_WARMUP":"1"})
            deadline=time.monotonic()+300
            while True:
                assert process.poll() is None,"server exited before readiness"
                try:
                    with urllib.request.urlopen(base+"/health",timeout=2) as response:
                        if response.status==200:break
                except OSError:pass
                assert time.monotonic()<deadline,"server readiness timeout"
                time.sleep(.25)
            jobs=[]
            for name in ("test_dense_server.py","test_secondary_server_modes.py","test_stop_boundaries.py"):
                command=[sys.executable,str(HERE/name)]
                if name=="test_dense_server.py":command.extend(["--base",base,"--output",str(out/"dense-http.json")])
                with (out/(name+".log")).open("w") as output:
                    result=subprocess.run(command,stdout=output,stderr=subprocess.STDOUT,env={**env,"SHIMMY_CERT_BASE":base,"SHIMMY_CERT_OUTPUT":str(out)},timeout=600)
                jobs.append(dict(test=name,exit=result.returncode))
                (out/"results.json").write_text(json.dumps(jobs,indent=2)+"\n")
                assert result.returncode==0,(name,result.returncode)
            assert ("backend="+expected_backend) in (out/"server.log").read_text()
        finally:
            if process is not None:
                process.terminate()
                try:process.wait(timeout=20)
                except subprocess.TimeoutExpired:process.kill();process.wait()
