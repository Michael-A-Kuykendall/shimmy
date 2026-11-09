"""Strict existing dense FP32/Metal certification assertions; no relaxed gates."""
import argparse, copy, hashlib, json, math, pathlib, struct, tempfile
EXPECTED={"qwen512":42,"llama512":12,"qwen2k":32,"qwen17":12,"qwen4k":67,"qwen8k":33}
def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()
def metric(actual, expected):
    assert len(actual)==len(expected)>0, "missing/partial vector"
    assert all(math.isfinite(x) for x in actual+expected), "nonfinite vector"
    delta=[a-b for a,b in zip(actual,expected)]
    result=dict(max_abs=max(map(abs,delta)),rms=math.sqrt(math.fsum(x*x for x in delta)/len(delta)),gpu_argmax=max(range(len(actual)),key=actual.__getitem__),reference_argmax=max(range(len(expected)),key=expected.__getitem__))
    assert result["max_abs"]<=0.1 and result["rms"]<=0.03 and result["gpu_argmax"]==result["reference_argmax"], result
    return result
def validate_matrix(data):
    assert data["schema"]=="apple.dense.certification.v1"
    assert data["host"]["system"]=="Darwin" and data["host"]["machine"]=="arm64"
    assert data["probe"]["selected"]["backend"]=="Metal" and data["probe"]["compute"]=="PASS"
    assert data["probe"]["actual"]==[float(i*2+1) for i in range(32)]
    assert data["reference_precision"]=="FP32-dequantized-weights/F32-KV" and data["limits"]==dict(max_abs=0.1,rms=0.03)
    assert data["m4"]=="NOT_RUN"
    for name,count in EXPECTED.items():
        row=data["tests"][name]
        assert row["status"]=="PASS" and row["comparisons"]==count and row["vectors_complete"] and row["all_greedy_match"]
        assert math.isfinite(row["max_abs"]) and math.isfinite(row["rms"])
        assert row["max_abs"]<=0.1 and row["rms"]<=0.03
    for name in ("state_reset_cancel","http","metal_compute"):
        assert data["tests"][name]["status"]=="PASS"
    assert data["complete"] is True
def falsify():
    for actual,expected in [([0.11],[0.0]),([0.04]*10,[0.0]*10),([float("nan")],[0.0]),([0.0],[0.0,1.0]),([0.01,0.0],[0.0,0.01])]:
        try:metric(actual,expected)
        except AssertionError:pass
        else:raise AssertionError("false numerical success")
    metric([0.001,2.0],[0.0,2.0])
    good=dict(schema="apple.dense.certification.v1",host=dict(system="Darwin",machine="arm64"),probe=dict(selected=dict(backend="Metal"),compute="PASS",actual=[float(i*2+1) for i in range(32)]),reference_precision="FP32-dequantized-weights/F32-KV",limits=dict(max_abs=0.1,rms=0.03),m4="NOT_RUN",complete=True,tests={})
    for name,count in EXPECTED.items():good["tests"][name]=dict(status="PASS",comparisons=count,vectors_complete=True,all_greedy_match=True,max_abs=0.01,rms=0.001)
    for name in ("state_reset_cancel","http","metal_compute"):good["tests"][name]=dict(status="PASS")
    validate_matrix(good)
    bads=[]
    for key,value in [("reference_precision","Q4 CPU"),("m4","PASS"),("complete",False),("limits",dict(max_abs=1,rms=1))]:
        bad=copy.deepcopy(good);bad[key]=value;bads.append(bad)
    bad=copy.deepcopy(good);bad["probe"]["selected"]["backend"]="CPU";bads.append(bad)
    bad=copy.deepcopy(good);bad["tests"]["qwen2k"]["status"]="NOT_RUN";bads.append(bad)
    bad=copy.deepcopy(good);bad["tests"]["state_reset_cancel"]["status"]="NOT_RUN";bads.append(bad)
    for bad in bads:
        try:validate_matrix(bad)
        except (AssertionError,KeyError):pass
        else:raise AssertionError("false matrix success")
    return 12+capture_controls()
def compare_capture(capture,job,root):
    assert capture["complete"] and capture["model_sha256_at_load"]==job["model_sha256"]
    cases=json.loads((root/job["cases"]).read_text());rows=capture["results"];compared=[];repeat={}
    expected_rows=sum(3 if job.get("kind")=="original" else len(c.get("modes",["text","injected","repeat"])) for c in cases)
    assert len(rows)==expected_rows
    pairs=[(r["case"],r["mode"]) for r in rows]
    wanted=[(c["name"],m) for c in cases for m in (["off","on","repeat"] if job.get("kind")=="original" else c.get("modes",["text","injected","repeat"]))]
    assert len(set(pairs))==len(pairs) and sorted(pairs)==sorted(wanted),"missing or substituted request mode"
    for row in rows:
        case=next(c for c in cases if c["name"]==row["case"])
        assert row["encoded_ids"]==row["injected_ids"]==case["ids"],"input IDs differ"
        if "expected_error" in case:
            assert row["error_code"]==case["expected_error"] and row["logits"]==[] and row["events"]==[]
            continue
        refs=job["references"][case["name"]];assert len(row["logits"])==len(refs)==row["completion_tokens"]
        assert row["prompt_tokens"]==len(case["ids"])
        for step,(actual,ref) in enumerate(zip(row["logits"],refs)):
            path=root/ref["path"];assert sha(path)==ref["sha256"]
            raw=path.read_bytes();expected=list(struct.unpack("<"+"f"*(len(raw)//4),raw))
            compared.append(dict(case=row["case"],mode=row["mode"],step=step,reference_sha256=ref["sha256"],**metric(actual,expected)))
        previous=repeat.setdefault(row["case"],row)
        assert all(row[k]==previous[k] for k in ("text","prompt_tokens","completion_tokens","finish_reason","logits")),"nondeterministic repeated execution"
    assert len(compared)==EXPECTED[job["name"]]
    return dict(status="PASS",comparisons=len(compared),vectors_complete=True,all_greedy_match=True,max_abs=max(c["max_abs"] for c in compared),rms=max(c["rms"] for c in compared),details=compared)
def capture_controls():
    with tempfile.TemporaryDirectory() as tmp:
        root=pathlib.Path(tmp);ref=root/"ref.f32";ref.write_bytes(struct.pack("<ff",0.0,2.0))
        case=dict(name="fixture",ids=[1],max_tokens=14)
        (root/"cases.json").write_text(json.dumps([case]))
        job=dict(name="qwen512",model_sha256="known-model",cases="cases.json",references={"fixture":[dict(path="ref.f32",sha256=sha(ref))]*14})
        row=dict(case="fixture",encoded_ids=[1],injected_ids=[1],prompt_tokens=1,completion_tokens=14,text="ok",finish_reason="Length",logits=[[0.0,2.0] for _ in range(14)])
        capture=dict(complete=True,model_sha256_at_load="known-model",results=[{**copy.deepcopy(row),"mode":mode} for mode in ("text","injected","repeat")])
        compare_capture(capture,job,root)
        bads=[]
        bad=copy.deepcopy(capture);bad["model_sha256_at_load"]="substitute";bads.append((bad,job))
        bad=copy.deepcopy(capture);bad["results"][0]["injected_ids"]=[2];bads.append((bad,job))
        bad=copy.deepcopy(capture);bad["results"][0]["logits"].pop();bads.append((bad,job))
        bad=copy.deepcopy(capture);bad["results"].pop();bads.append((bad,job))
        wrong=copy.deepcopy(job);wrong["references"]["fixture"][0]["sha256"]="wrong";bads.append((capture,wrong))
        for bad,design in bads:
            try:compare_capture(bad,design,root)
            except (AssertionError,KeyError):pass
            else:raise AssertionError("capture falsification accepted")
    # Original request captures off/on/repeat independently of token-input modes.
    with tempfile.TemporaryDirectory() as tmp:
        root=pathlib.Path(tmp);ref=root/"ref.f32";ref.write_bytes(struct.pack("<ff",0.0,2.0))
        case=dict(name="original",ids=[1,2],max_tokens=4,modes=["injected"])
        (root/"cases.json").write_text(json.dumps([case]))
        job=dict(name="qwen17",kind="original",model_sha256="known-model",cases="cases.json",
            references={"original":[dict(path="ref.f32",sha256=sha(ref))]*4})
        row=dict(case="original",encoded_ids=[1,2],injected_ids=[1,2],prompt_tokens=2,
            completion_tokens=4,text="ok",finish_reason="Length",logits=[[0.0,2.0] for _ in range(4)])
        capture=dict(complete=True,model_sha256_at_load="known-model",
            results=[{**copy.deepcopy(row),"mode":mode} for mode in ("off","on","repeat")])
        assert compare_capture(capture,job,root)["comparisons"]==12
        bads=[]
        bad=copy.deepcopy(capture);bad["results"].pop();bads.append(bad)
        bad=copy.deepcopy(capture);bad["results"][1]["mode"]="off";bads.append(bad)
        bad=copy.deepcopy(capture);bad["results"][0]["logits"][0]=[1.0,2.0];bads.append(bad)
        bad=copy.deepcopy(capture);bad["results"][0]["injected_ids"]=[2,1];bads.append(bad)
        bad=copy.deepcopy(capture);bad["results"][0]["logits"].pop();bads.append(bad)
        for bad in bads:
            try:compare_capture(bad,job,root)
            except (AssertionError,KeyError):pass
            else:raise AssertionError("original capture falsification accepted")
    return 10

def main():
    p=argparse.ArgumentParser();p.add_argument("--self-test",action="store_true");p.add_argument("--matrix");a=p.parse_args()
    n=falsify()
    if a.matrix:validate_matrix(json.loads(pathlib.Path(a.matrix).read_text()))
    print(json.dumps(dict(status="PASS",negative_controls=n)))
if __name__=="__main__":main()
