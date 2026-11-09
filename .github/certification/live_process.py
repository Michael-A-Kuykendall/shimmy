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

def execute(command,env,log_path,progress_path,timeout,stall_timeout=1200,heartbeat=30,echo=True,operation_timeout=0,sampler=None,sample_after=30,stop_on_native_error=False,stop_after_chunks=0):
    assert timeout>0 and stall_timeout>0 and heartbeat>0 and isinstance(stop_after_chunks,int) and stop_after_chunks>=0
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
    cleanup=[];operations={};last_boundary=None;sampled=set();sample_errors=[]
    def kill():
        # Cleanup failure is evidence, never a replacement for the timeout verdict.
        # A driver-blocked process may be unreapable even after SIGKILL.
        for sig,wait_seconds in ((signal.SIGTERM,.5),(signal.SIGKILL,2)):
            try:os.killpg(child.pid,sig)
            except ProcessLookupError:pass
            except OSError as error:cleanup.append(dict(operation="killpg",signal=int(sig),error=str(error)))
            try:child.wait(timeout=wait_seconds)
            except subprocess.TimeoutExpired:cleanup.append(dict(operation="wait",signal=int(sig),error="timeout"))
        cleanup.append(dict(operation="final",reaped=child.poll() is not None,returncode=child.returncode))
    with log_path.open("w") as log,progress_path.open("w") as progress:
        def event(kind):
            nonlocal events
            now=time.monotonic();entry=dict(event=kind,elapsed_seconds=round(now-started,3),
                idle_seconds=round(now-last_progress,3),completed_chunks=state.completed,
                completed_modes=state.modes.copy(),chunk_tokens=state.tokens,last_observed=state.last,
                wall_timeout_seconds=timeout,stall_timeout_seconds=stall_timeout,cleanup=cleanup.copy(),
                active_operations=list(operations),last_boundary=last_boundary,operation_timeout_seconds=operation_timeout)
            progress.write(json.dumps(entry)+"\n");progress.flush();events+=1
            if echo:
                notice=kind=="progress" and (state.last.startswith("full request") or (state.tokens>1 and (state.index==0 or state.tokens<512)))
                prefix="::notice title=Metal numerical progress::" if notice else ("CERT_PROGRESS " if kind=="progress" else "CERT_HEARTBEAT ")
                print(prefix+json.dumps(entry),flush=True)
        event("started")
        try:
            while True:
                now=time.monotonic()
                if sampler:
                    for operation,started_operation in list(operations.items()):
                        if operation not in sampled and now-started_operation>=sample_after:
                            sampled.add(operation)
                            try:sampler(child.pid,operation)
                            except Exception as error:sample_errors.append(str(error))
                if operation_timeout and any(now-t>=operation_timeout for t in operations.values()):
                    reason="operation-timeout";kill();break
                if now-started>=timeout:reason="wall-timeout";kill();break
                if now-last_progress>=stall_timeout:reason="no-GPU-progress-timeout";kill();break
                try:line=messages.get(timeout=min(.1,heartbeat))
                except queue.Empty:line=""
                if line is None:end=True
                elif line:
                    log.write(line);log.flush()
                    if echo:print(line,end="",flush=True)
                    if stop_on_native_error and line.startswith("METAL_COMMAND_ERROR "):
                        reason="native-command-error";kill();break
                    if operation_timeout and line.startswith("METAL_BOUNDARY "):
                        _,timestamp,edge,operation=line.strip().split(" ",3)
                        assert timestamp.isdigit() and edge in ("enter","exit","point"),"invalid operation boundary"
                        last_boundary=dict(timestamp_ns=timestamp,edge=edge,operation=operation)
                        if edge=="enter":
                            assert operation not in operations,"reentered incomplete operation"
                            operations[operation]=time.monotonic()
                        elif edge=="exit":
                            assert operation in operations,"unmatched operation completion"
                            del operations[operation]
                            # Diagnostic-only completed blocking GPU waits are real progress.
                            # Point/enter/chatter records cannot extend the stall deadline.
                            if operation.startswith(("device.poll:","model.device.poll")):
                                last_progress=time.monotonic()
                        event("boundary")
                    if state.consume(line):
                        last_progress=time.monotonic();event("progress")
                        if stop_after_chunks and state.completed>=stop_after_chunks:
                            reason="diagnostic-prefix-completed";kill();break
                if now-last_heartbeat>=heartbeat:event("heartbeat");last_heartbeat=now
                if end and child.poll() is not None:break
                if child.poll() is not None:
                    if exit_observed is None:exit_observed=time.monotonic()
                    if not end and time.monotonic()-exit_observed>2:
                        reason="orphaned-output-process";kill();break
            code=child.poll()
            if code is None:
                reason=reason or "cleanup-blocked";code=1
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
        stall_timeout_seconds=stall_timeout,timeout_seconds=timeout,cleanup=cleanup,last_boundary=last_boundary,active_operations=list(operations),sample_errors=sample_errors,requested_prefix_chunks=stop_after_chunks)

def self_test():
    with tempfile.TemporaryDirectory() as tmp:
        root=pathlib.Path(tmp)
        from unittest import mock
        def run(name,code,timeout=6,stall=2):
            return execute([sys.executable,"-u","-c",code],os.environ.copy(),root/(name+".log"),
                root/(name+".jsonl"),timeout,stall,.05,False)
        def event_clock_run(name,code,timeout=3,stall=.2,operation_timeout=0):
            clock=[0.0];messages=queue.Queue();real_get=messages.get
            def timed_get(*args,**kwargs):
                item=real_get(*args,**kwargs)
                if item is not None:clock[0]+=.08
                return item
            with mock.patch.object(queue,"Queue",return_value=messages), \
                 mock.patch.object(messages,"get",side_effect=timed_get), \
                 mock.patch.object(time,"monotonic",side_effect=lambda:clock[0]):
                return execute([sys.executable,"-u","-c",code],os.environ.copy(),
                    root/(name+".log"),root/(name+".jsonl"),timeout,stall,.05,False,
                    operation_timeout=operation_timeout)
        good=run("good","import time;print('[PREFILL] chunk=0 tokens=1 current_pos=0 seq_len=1');time.sleep(.1);print('[PREFILL] chunk=0 complete');print('REQUEST_COMPLETED off')")
        assert good["exit"]==0 and good["completed_chunks"]==1 and good["completed_modes"]==["off"]
        stalled=run("stalled","import time;time.sleep(5)")
        assert stalled["exit"]==124 and stalled["reason"]=="no-GPU-progress-timeout"
        chatter=run("chatter","import time\nfor _ in range(100):\n print('heartbeat alive');time.sleep(.03)")
        assert chatter["exit"]==124 and chatter["completed_chunks"]==0
        start_only=run("start-only","import time;print('[PREFILL] chunk=0 tokens=1 current_pos=0 seq_len=1');time.sleep(5)")
        assert start_only["exit"]==124
        wall=event_clock_run("wall","for i in range(8):\n print(f'[PREFILL] chunk={i} tokens=1 current_pos={i} seq_len={i+1}');print(f'[PREFILL] chunk={i} complete')",timeout=.3,stall=2)
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
        timers=[];real_signal=signal.signal
        def registered_then_cancel(sig,handler):
            previous=real_signal(sig,handler)
            if sig==signal.SIGTERM and callable(handler) and not timers:
                timer=threading.Timer(.1,lambda:os.kill(os.getpid(),signal.SIGTERM))
                timers.append(timer);timer.start()
            return previous
        with mock.patch.object(signal,"signal",side_effect=registered_then_cancel):
            cancelled=run("cancelled","import time;time.sleep(30)")
        for timer in timers:timer.join()
        assert cancelled["exit"]==130 and cancelled["reason"]=="cancelled"
        assert all(any(json.loads(line)["event"].endswith("timeout") for line in (root/(name+".jsonl")).read_text().splitlines()) for name in ("stalled","chatter","start-only","wall","descendant"))
        real_killpg=os.killpg
        # Reproduce EPERM without permitting a long-lived process to leak.
        with mock.patch.object(os,"killpg",side_effect=PermissionError(1,"simulated driver denial")):
            denied=run("permission-denied","import time;time.sleep(.6)",stall=.1)
        assert denied["exit"]==124 and denied["reason"]=="no-GPU-progress-timeout"
        assert any(x.get("operation")=="killpg" for x in denied["cleanup"])
        assert denied["cleanup"][-1]["reaped"]
        # Reproduce an unreapable driver task: bounded cleanup must retain logs/verdict.
        spawned=[];real_popen=subprocess.Popen
        def wrapped(*args,**kwargs):
            child=real_popen(*args,**kwargs);spawned.append(child)
            child.wait=mock.Mock(side_effect=subprocess.TimeoutExpired(args[0],2))
            return child
        try:
            with mock.patch.object(subprocess,"Popen",side_effect=wrapped):
                unreaped=run("unreapable","import time;print('preserved',flush=True);time.sleep(30)",stall=2)
            assert unreaped["exit"]==124 and unreaped["reason"]=="no-GPU-progress-timeout"
            assert any(x.get("operation")=="wait" for x in unreaped["cleanup"])
            assert "preserved" in (root/"unreapable.log").read_text()
            assert json.loads((root/"unreapable.jsonl").read_text().splitlines()[-1])["event"]=="no-GPU-progress-timeout"
        finally:
            for child in spawned:
                # Darwin may report EPERM for an already-reaped empty process group.
                # Do not signal it again; always verify the owned child is reaped.
                if child.poll() is None:
                    try:real_killpg(child.pid,signal.SIGKILL)
                    except ProcessLookupError:pass
                real_popen.wait(child,timeout=3)
        operation=execute([sys.executable,"-u","-c","import time;print('METAL_BOUNDARY 123 enter queue.submit:first');time.sleep(5)"],
            os.environ.copy(),root/"operation.log",root/"operation.jsonl",3,2,.05,False,operation_timeout=.2)
        assert operation["exit"]==124 and operation["reason"]=="operation-timeout"
        assert operation["active_operations"]==["queue.submit:first"]
        assert operation["last_boundary"]["edge"]=="enter"
        completed=event_clock_run("operation-good","print('METAL_BOUNDARY 123 enter queue.submit:first');print('METAL_BOUNDARY 124 exit queue.submit:first')",stall=2,operation_timeout=.2)
        assert completed["exit"]==0 and not completed["active_operations"]
        point=execute([sys.executable,"-u","-c","import time\nfor i in range(50):\n print(f'METAL_BOUNDARY {i} point alive');time.sleep(.03)"],
            os.environ.copy(),root/"point.log",root/"point.jsonl",3,.2,.05,False,operation_timeout=1)
        assert point["reason"]=="no-GPU-progress-timeout"
        # Use a virtual event clock: OS process startup/scheduling is not the contract.
        # Eight completed waits span 1.28s, exceeding the .2s idle deadline.
        def scheduled_boundaries(name,operation):
            code="for i in range(8):\n print(f'METAL_BOUNDARY {i} enter "+operation+"');print(f'METAL_BOUNDARY {i} exit "+operation+"')"
            return event_clock_run(name,code,operation_timeout=1)
        waits=scheduled_boundaries("waits","device.poll:case")
        assert waits["exit"]==0 and not waits["active_operations"] and waits["elapsed"]>1
        non_gpu=scheduled_boundaries("non-gpu","queue.submit:case")
        assert non_gpu["exit"]==124 and non_gpu["reason"]=="no-GPU-progress-timeout"
        calls=[]
        def sample_failure(pid,operation):
            calls.append((pid,operation));raise RuntimeError("sample unavailable")
        sampled_result=execute([sys.executable,"-u","-c","import time;print('METAL_BOUNDARY 1 enter device.poll:stuck');time.sleep(5)"],
            os.environ.copy(),root/"sample.log",root/"sample.jsonl",3,2,.05,False,operation_timeout=.3,sampler=sample_failure,sample_after=.05)
        assert len(calls)==1 and calls[0][1]=="device.poll:stuck"
        assert sampled_result["reason"]=="operation-timeout" and sampled_result["sample_errors"]==["sample unavailable"]
        native=execute([sys.executable,"-u","-c","import time;print('METAL_COMMAND_ERROR fence=1 index=0 total=1 code=3 synthetic',flush=True);time.sleep(5)"],
            os.environ.copy(),root/"native.log",root/"native.jsonl",3,2,.05,False,stop_on_native_error=True)
        assert native["exit"]!=0 and native["reason"]=="native-command-error"
        assert "code=3 synthetic" in (root/"native.log").read_text()
        assert native["cleanup"][-1]["reaped"]
    return dict(clean="PASS",negative_controls=15,actual_Metal="NOT_RUN")

if __name__=="__main__":print(json.dumps(self_test()))
