"""Bounded process groups: a timed-out profiler wrapper cannot leave GPU children."""
import os,signal,subprocess,tempfile,time,sys,pathlib
def execute(command,env,log,timeout):
    child=subprocess.Popen(command,stdout=log,stderr=subprocess.STDOUT,env=env,start_new_session=True)
    try:return child.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        try:os.killpg(child.pid,signal.SIGKILL)
        except ProcessLookupError:pass
        child.wait()
        return 124

def self_test():
    with tempfile.TemporaryDirectory() as tmp:
        path=pathlib.Path(tmp)/"children.log"
        code="import subprocess,sys,time;p=subprocess.Popen([sys.executable,'-c','import time;time.sleep(120)']);print(p.pid,flush=True);time.sleep(120)"
        with path.open("w") as log:
            result=execute([sys.executable,"-c",code],os.environ.copy(),log,1.0)
        assert result==124
        pid=int(path.read_text().strip().splitlines()[0])
        for _ in range(100):
            state=subprocess.run(["ps","-o","stat=","-p",str(pid)],capture_output=True,text=True).stdout.strip()
            if not state or state.startswith("Z"):return True
            time.sleep(0.02)
        raise AssertionError("timed-out descendant still running")
