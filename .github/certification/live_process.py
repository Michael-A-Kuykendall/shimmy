"""Numerical-run watchdog: real chunk completion advances the stall clock; chatter does not."""
import json,os,pathlib,queue,re,signal,subprocess,sys,threading,time,tempfile

class Progress:
    def __init__(self):
        self.pending=None;self.completed=0;self.last="process starting";self.modes=[];self.tokens=0;self.index=-1
    def consume(self,line):
        start=re.fullmatch(r"\[PREFILL\] chunk=(\d+) tokens=(\d+) current_pos=(\d+) seq_len=(\d+)",line.strip())
        done=re.fullmatch(r"\[PREFILL\] chunk=(\d+) complete",line.strip())
        if start:
            index,tokens,pos,end=map(int,start.groups())
            assert tokens>0 and end==pos+tokens and self.pending is None,"invalid prefill progress"
            self.pending=index;self.tokens=tokens;self.index=index;self.last=f"GPU chunk {index} submitted: positions {pos}..{end-1}"
            return False
        if done:
            index=int(done[1]);assert self.pending==index,"unmatched chunk completion"
            self.pending=None;self.completed+=1;self.last=f"GPU chunk {index} completed ({self.completed} total)"
            return True
        if line.startswith("REQUEST_COMPLETED "):
            mode=line.strip().split()[-1]
            assert mode in ("off","on","repeat") and mode not in self.modes and self.pending is None,"invalid request completion"
            self.modes.append(mode);self.last=f"full request completed: {mode}"
            return True
        return False

def execute(command,env,log_path,progress_path,timeout,stall_timeout=1200,heartbeat=30,echo=True):
    assert timeout>0 and stall_timeout>0 and heartbeat>0
    log_path=pathlib.Path(log_path);progress_path=pathlib.Path(progress_path)
    child=subprocess.Popen(list(map(str,command)),stdout=subprocess.PIPE,stderr=subprocess.STDOUT,
        env=env,start_new_session=True,bufsize=1,text=True,errors="replace")
    messages=queue.Queue()
    def read():
        try:
            for line in child.stdout:messages.put(line)
        finally:messages.put(None)
    reader=threading.Thread(target=read,daemon=True);reader.start()
    def cancelled(signum,frame):raise InterruptedError("runner cancellation")
    previous_term=signal.signal(signal.SIGTERM,cancelled)
    started=time.monotonic();last_progress=started;last_heartbeat=started
    state=Progress();reason=None;end=False;events=0;exit_observed=None
    def kill():
        try:os.killpg(child.pid,signal.SIGKILL)
        except ProcessLookupError:pass
        child.wait(timeout=10)
    with log_path.open("w") as log,progress_path.open("w") as progress:
        def event(kind):
            nonlocal events
            now=time.monotonic();entry=dict(event=kind,elapsed_seconds=round(now-started,3),
                idle_seconds=round(now-last_progress,3),completed_chunks=state.completed,
                completed_modes=state.modes.copy(),chunk_tokens=state.tokens,last_observed=state.last,
                wall_timeout_seconds=timeout,stall_timeout_seconds=stall_timeout)
            progress.write(json.dumps(entry)+"\n");progress.flush();events+=1
            if echo:
                notice=kind=="progress" and (state.last.startswith("full request") or (state.tokens>1 and (state.index==0 or state.tokens<512)))
                prefix="::notice title=Metal numerical progress::" if notice else ("CERT_PROGRESS " if kind=="progress" else "CERT_HEARTBEAT ")
                print(prefix+json.dumps(entry),flush=True)
        event("started")
        try:
            while True:
                now=time.monotonic()
                if now-started>=timeout:reason="wall-timeout";kill();break
                if now-last_progress>=stall_timeout:reason="no-GPU-progress-timeout";kill();break
                try:line=messages.get(timeout=min(.1,heartbeat))
                except queue.Empty:line=""
                if line is None:end=True
                elif line:
                    log.write(line);log.flush()
                    if echo:print(line,end="",flush=True)
                    if state.consume(line):
                        last_progress=time.monotonic();event("progress")
                if now-last_heartbeat>=heartbeat:event("heartbeat");last_heartbeat=now
                if end and child.poll() is not None:break
                if child.poll() is not None:
                    if exit_observed is None:exit_observed=time.monotonic()
                    if not end and time.monotonic()-exit_observed>2:
                        reason="orphaned-output-process";kill();break
            code=child.wait()
            # A descendant must not survive its parent and keep the pipe open.
            if not end and reason is None:
                reason="orphaned-output-process";kill()
            reader.join(timeout=2)
            while not messages.empty():
                line=messages.get_nowait()
                if line:log.write(line)
            if reason:code=124 if reason.endswith("timeout") else 1
            event(reason or ("process-completed" if code==0 else "process-failed"))
        except (InterruptedError,KeyboardInterrupt):
            reason="cancelled";code=130;kill();reader.join(timeout=2);event("cancelled")
        except BaseException:
            kill();reader.join(timeout=2);event("watchdog-error");raise
        finally:
            signal.signal(signal.SIGTERM,previous_term)
            if not reader.is_alive():child.stdout.close()
    return dict(exit=code,reason=reason,elapsed=time.monotonic()-started,
        completed_chunks=state.completed,completed_modes=state.modes,progress_events=events,
        stall_timeout_seconds=stall_timeout,timeout_seconds=timeout)

def self_test():
    with tempfile.TemporaryDirectory() as tmp:
        root=pathlib.Path(tmp)
        def run(name,code,timeout=3,stall=.4):
            return execute([sys.executable,"-u","-c",code],os.environ.copy(),root/(name+".log"),
                root/(name+".jsonl"),timeout,stall,.05,False)
        good=run("good","import time;print('[PREFILL] chunk=0 tokens=1 current_pos=0 seq_len=1');time.sleep(.1);print('[PREFILL] chunk=0 complete');print('REQUEST_COMPLETED off')")
        assert good["exit"]==0 and good["completed_chunks"]==1 and good["completed_modes"]==["off"]
        stalled=run("stalled","import time;time.sleep(5)")
        assert stalled["exit"]==124 and stalled["reason"]=="no-GPU-progress-timeout"
        chatter=run("chatter","import time\nfor _ in range(100):\n print('heartbeat alive');time.sleep(.03)")
        assert chatter["exit"]==124 and chatter["completed_chunks"]==0
        start_only=run("start-only","import time;print('[PREFILL] chunk=0 tokens=1 current_pos=0 seq_len=1');time.sleep(5)")
        assert start_only["exit"]==124
        wall=run("wall","import time\nfor i in range(100):\n print(f'[PREFILL] chunk={i} tokens=1 current_pos={i} seq_len={i+1}');print(f'[PREFILL] chunk={i} complete');time.sleep(.05)",timeout=.3)
        assert wall["exit"]==124 and wall["reason"]=="wall-timeout" and wall["completed_chunks"]>0
        try:run("bad-progress","print('[PREFILL] chunk=0 complete')")
        except AssertionError:pass
        else:raise AssertionError("unmatched progress accepted")
        descendant=run("descendant","import subprocess,time,sys;p=subprocess.Popen([sys.executable,'-c','import time;time.sleep(30)']);print('DESCENDANT',p.pid,flush=True);time.sleep(30)")
        assert descendant["exit"]==124
        pid=int((root/"descendant.log").read_text().split()[1])
        status=subprocess.run(["ps","-o","stat=","-p",str(pid)],capture_output=True,text=True).stdout.strip()
        assert not status or status.startswith("Z"),"live descendant survived timeout"
        orphan=run("orphan","import subprocess,sys;p=subprocess.Popen([sys.executable,'-c','import time;time.sleep(30)']);print('DESCENDANT',p.pid,flush=True)",timeout=5,stall=4)
        assert orphan["exit"]==1 and orphan["reason"]=="orphaned-output-process"
        timer=threading.Timer(.1,lambda:os.kill(os.getpid(),signal.SIGTERM));timer.start()
        cancelled=run("cancelled","import time;time.sleep(30)")
        timer.join();assert cancelled["exit"]==130 and cancelled["reason"]=="cancelled"
        assert all(any(json.loads(line)["event"].endswith("timeout") for line in (root/(name+".jsonl")).read_text().splitlines()) for name in ("stalled","chatter","start-only","wall","descendant"))
    return dict(clean="PASS",negative_controls=8,actual_Metal="NOT_RUN")

if __name__=="__main__":print(json.dumps(self_test()))
