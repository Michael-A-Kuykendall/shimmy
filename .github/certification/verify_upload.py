"""Integrity checks for actual initialization contrasts, not model certification."""
import copy
MODEL="72c5c3cb38fa32d5256e2fe30d03e7a64c6c79e668ad84057e3bd66e250b24fb"
SIZES=[536870912,1073741824,1282439584]
METHODS=["device-ext","device-ext-drop-source","explicit-mapped","bounded-queue"]
def validate_upload(data):
    assert data["complete"] is True and data["model_sha256"]==MODEL
    assert len(data["results"])==12
    assert {(r["size"],r["method"]) for r in data["results"]}=={(n,m) for n in SIZES for m in METHODS}
    for r in data["results"]:
        assert len(r["samples"])==4
        assert [(s["offset"],s["size"]) for s in r["samples"]]==[(0,256),(261225248,1152),(435906464,1152),(r["size"]-256,256)]
        assert r["source_dropped_before_submit"]==(r["method"]=="device-ext-drop-source")
        if r["method"]=="explicit-mapped":assert isinstance(r["cpu_mapped_samples_match"],bool)
        for sample in r["samples"]:
            assert isinstance(sample["matches"],bool)
            for key in ("zero_bytes","cpu_zero_bytes"):assert 0<=sample[key]<=sample["size"]
        assert r["pass"]==all(s["matches"] for s in r["samples"]),"invented success"
    return all(r["pass"] for r in data["results"])

def validate_load_trace(path):
    import json
    rows=[json.loads(x) for x in path.read_text().splitlines()]
    assert [r["stage"] for r in rows]==["after-model","after-pipelines","after-output-head","after-kv"]
    for row in rows:
        assert [r["token"] for r in row["rows"]]==[11,151644]
        for r in row["rows"]:assert isinstance(r["matches"],bool) and 0<=r["zero_bytes"]<=1152
    return rows

def self_test():
    good=dict(complete=True,model_sha256=MODEL,results=[
        dict(size=n,method=m,source_dropped_before_submit=m=="device-ext-drop-source",cpu_mapped_samples_match=True if m=="explicit-mapped" else None,
             samples=[dict(offset=o,size=s,matches=True,zero_bytes=0,cpu_zero_bytes=0) for o,s in [(0,256),(261225248,1152),(435906464,1152),(n-256,256)]],**{"pass":True})
        for n in SIZES for m in METHODS])
    assert validate_upload(good)
    changes=[
        lambda d:d.update(model_sha256="0"*64),
        lambda d:d["results"].pop(),
        lambda d:d["results"][0].update(method="unknown"),
        lambda d:d["results"][0]["samples"].pop(),
        lambda d:d["results"][0]["samples"][0].update(matches=False),
        lambda d:d["results"][0].update(size=0),
        lambda d:d["results"][2].update(cpu_mapped_samples_match=None),
    ]
    for change in changes:
        bad=copy.deepcopy(good);change(bad)
        try:validate_upload(bad)
        except (AssertionError,KeyError):pass
        else:raise AssertionError("accepted false evidence")
    bad=copy.deepcopy(good);bad["results"][0]["samples"][0]["matches"]=False;bad["results"][0]["pass"]=False
    assert validate_upload(bad) is False
    return len(changes)
