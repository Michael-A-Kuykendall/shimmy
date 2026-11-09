"""Append-only shared-memory observations; parent timestamps receipt, never GPU duration."""
import mmap, struct, pathlib, time, json
MAGIC=int.from_bytes(b"AF8KFR01","little")
HEADER=64
SLOT=40
CAPACITY=262144
MASK=(1<<64)-1

def checksum(index,code,a,b):
    value=MAGIC
    for word in (index,code,a,b): value=((value^word)*1099511628211)&MASK
    return value

class Flight:
    def __init__(self,path,events,capacity=CAPACITY):
        self.path=pathlib.Path(path); self.capacity=capacity
        with self.path.open("xb") as f:
            f.truncate(HEADER+capacity*SLOT)
            f.seek(0); f.write(struct.pack("<QQ",MAGIC,capacity))
        self.file=self.path.open("r+b")
        self.memory=mmap.mmap(self.file.fileno(),0)
        self.output=pathlib.Path(events).open("x")
        self.index=0;self.chunks=0;self.pending=None;self.statuses={};self.last=None
    def poll(self):
        m=self.memory
        magic,capacity,next_index,overflow=struct.unpack_from("<QQQQ",m)
        assert magic==MAGIC and capacity==self.capacity,"flight header changed"
        assert overflow==0 and next_index<=capacity,"flight recorder overflow"
        lines=[]
        while self.index<next_index:
            index=self.index
            commit,code,a,b,check=struct.unpack_from("<QQQQQ",m,HEADER+index*SLOT)
            if commit==0: break
            assert commit==index+1 and check==checksum(index,code,a,b),"corrupt flight record"
            self.index+=1
            record=dict(index=index,code=code,a=a,b=b,received_monotonic=time.monotonic())
            self.last=record;self.output.write(json.dumps(record)+"\n")
            if code==20:
                assert self.pending is None and b>0,"invalid chunk start"
                self.pending=(a,b)
                lines.append(f"[PREFILL] chunk={self.chunks} tokens={b} current_pos={a} seq_len={a+b}\n")
            elif code==21:
                assert self.pending==(a,b),"unmatched chunk completion"
                self.pending=None
                lines.append(f"[PREFILL] chunk={self.chunks} complete\n");self.chunks+=1
            elif code in (10,11,12,13,14,15):
                operation={10:"encoder.finish",11:"encoder.finish",12:"queue.submit",13:"queue.submit",14:"device.poll:tdr",15:"device.poll:tdr"}[code]
                edge="enter" if code in (10,12,14) else "exit"
                lines.append(f"METAL_BOUNDARY {index} {edge} {operation}\n")
            elif code in (102,110):
                self.statuses[a]=b
                if b==5:lines.append(f"METAL_COMMAND_ERROR fence={a} status=Error\n")
        self.output.flush()
        return lines
    def close(self):
        self.output.close();self.memory.close();self.file.close()

def self_test():
    import tempfile,sys,subprocess,os
    from live_process import execute
    def write(f,code,a,b,commit=True):
        i=f.index;offset=HEADER+i*SLOT
        struct.pack_into("<QQQQQ",f.memory,offset,i+1 if commit else 0,code,a,b,checksum(i,code,a,b))
        struct.pack_into("<Q",f.memory,16,i+1)
    with tempfile.TemporaryDirectory() as tmp:
        root=pathlib.Path(tmp)
        f=Flight(root/"memory",root/"events",16)
        write(f,20,0,64,False);assert f.poll()==[],"partial record accepted"
        struct.pack_into("<Q",f.memory,HEADER,1);assert "tokens=64" in f.poll()[0]
        write(f,21,0,64);assert "complete" in f.poll()[0] and f.chunks==1
        write(f,110,7,5);assert "METAL_COMMAND_ERROR" in f.poll()[0]
        write(f,20,64,64);f.memory[HEADER+f.index*SLOT+32]^=1
        try:f.poll()
        except AssertionError:pass
        else:raise AssertionError("corrupt record accepted")
        f.close()
        f=Flight(root/"overflow",root/"overflow-events",2)
        struct.pack_into("<Q",f.memory,24,1)
        try:f.poll()
        except AssertionError:pass
        else:raise AssertionError("overflow accepted")
        f.close()
        # Real child writes mapping without stdout; parent progress controls watchdog.
        f=Flight(root/"child-memory",root/"child-events",16)
        code="""import mmap,struct,sys,time
file=open(sys.argv[1],"r+b");m=mmap.mmap(file.fileno(),0)
for i,(code,a,b) in enumerate(((20,0,64),(21,0,64))):
 h=int.from_bytes(b"AF8KFR01","little")
 for v in (i,code,a,b):h=((h^v)*1099511628211)&((1<<64)-1)
 struct.pack_into("<QQQQQ",m,64+i*40,i+1,code,a,b,h)
 struct.pack_into("<Q",m,16,i+1)
time.sleep(30)
"""
        result=execute([sys.executable,"-c",code,str(f.path)],os.environ.copy(),root/"child.log",root/"child-progress",6,3,.1,False,event_source=f.poll,stop_after_chunks=1)
        assert result["reason"]=="diagnostic-prefix-completed" and result["completed_chunks"]==1
        assert result["exit"]!=0 and result["cleanup"][-1]["reaped"]
        f.close()
    return dict(status="PASS",partial_corrupt_overflow_rejected=True,cross_process_progress=True,prefix_is_not_certification=True)
