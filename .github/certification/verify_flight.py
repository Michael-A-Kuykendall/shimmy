"""Verify frozen graph and that diagnostic edits preserve original scheduling/math."""
import argparse,copy,json,pathlib,re,subprocess,tomllib
HERE=pathlib.Path(__file__).resolve().parent

def graph(baseline,candidate):
    before=tomllib.loads(pathlib.Path(baseline).read_text())["package"]
    after=tomllib.loads(pathlib.Path(candidate).read_text())["package"]
    expected=copy.deepcopy(before)
    matches=[p for p in expected if p["name"]=="wgpu-hal"]
    assert len(matches)==1 and matches[0]["version"]=="27.0.4"
    assert matches[0].pop("source")=="registry+https://github.com/rust-lang/crates.io-index"
    assert matches[0].pop("checksum")=="5b21cb61c57ee198bc4aff71aeadff4cbb80b927beb912506af9c780d64313ce"
    assert expected==after,"dependency version/source/edge drift"
    return True

def strip_events(source):
    return re.sub(r"crate::diagnostic_flight::emit\([^;]*?\);","",source)

def normalize(source):
    return re.sub(r"\s+","",source)

def preserve(baseline,candidate):
    restored=strip_events(candidate)
    restored=restored.replace("crate::diagnostic_flight::initialize()?;","").replace("mod diagnostic_flight;","")
    assert normalize(baseline)==normalize(restored),"non-observation engine change"
    assert "eprintln!" not in "".join(re.findall(r"crate::diagnostic_flight::emit\([^;]*?\);",candidate))
    return True

def verify_engine(engine):
    expected={"src/lib.rs","src/runtime/gpu.rs","src/backend/tdr.rs","src/backend/bindless/pipeline/inference.rs",
              "src/diagnostic_flight/core.rs","src/diagnostic_flight/mod.rs"}
    changed=set(subprocess.check_output(["git","-C",str(engine),"diff","--name-only","68091da8c55a2f96bc4309b46ece92630c996f71","HEAD"],text=True).splitlines())
    assert changed==expected,"unexpected engine file change"
    for file in expected-{"src/diagnostic_flight/core.rs","src/diagnostic_flight/mod.rs"}:
        before=subprocess.check_output(["git","-C",str(engine),"show","68091da8c55a2f96bc4309b46ece92630c996f71:"+file],text=True)
        preserve(before,(engine/file).read_text())
    core=(engine/"src/diagnostic_flight/core.rs").read_text()
    assert "fetch_add" in core and "Ordering::Release" in core
    assert not any(x in core for x in ("println!","std::env","Instant","sleep","queue.submit","device.poll"))
    # Falsify changes to scheduler policy, extra waits and altered computation.
    baseline="fn f(){let t0=Instant::now(); device.poll(wait_indefinitely());}"
    assert preserve(baseline,baseline.replace("device.poll","crate::diagnostic_flight::emit(14,0,0);device.poll"))
    negatives=0
    for candidate in (baseline.replace("wait_indefinitely","wait_timeout"),baseline+"device.poll(wait_indefinitely());",baseline.replace("Instant::now()","Instant::now()+delta")):
        try:preserve(baseline,candidate)
        except AssertionError:negatives+=1
        else:raise AssertionError("policy mutation accepted")
    assert negatives==3
    return True

def main():
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        root=pathlib.Path(tmp);nested=root/"hal";nested.mkdir()
        subprocess.run(["git","init","-q",str(root)],check=True)
        patch=root/"fixture.patch"
        patch.write_text("diff --git a/src/flight b/src/flight\nnew file mode 100644\n--- /dev/null\n+++ b/src/flight\n@@ -0,0 +1 @@\n+fixture\n")
        subprocess.run(["git","-C",str(nested),"apply",str(patch)],check=True)
        assert not (nested/"src/flight").exists(),"nested precondition changed"
        subprocess.run(["git","init","-q",str(nested)],check=True)
        subprocess.run(["git","-C",str(nested),"apply","--check",str(patch)],check=True)
        subprocess.run(["git","-C",str(nested),"apply",str(patch)],check=True)
        assert (nested/"src/flight").read_text()=="fixture\n"
    p=argparse.ArgumentParser();p.add_argument("--engine",type=pathlib.Path);p.add_argument("--hal",type=pathlib.Path);a=p.parse_args()
    assert graph(HERE/"probes/diagnostic.Cargo.lock",HERE/"probes/flight.Cargo.lock")
    if a.engine:
        assert verify_engine(a.engine)
        assert (a.engine/"src/diagnostic_flight/core.rs").read_bytes()==(a.hal/"src/diagnostic_flight/core.rs").read_bytes()
        source=(a.hal/"src/metal/mod.rs").read_text()
        baseline=subprocess.check_output(["git","-C",str(a.hal),"show","HEAD~1:src/metal/mod.rs"],text=True)
        restored=strip_events(source).replace("crate::diagnostic_flight::initialize();","").replace("move |cmd_buf: &metal::CommandBufferRef|","move |_cmd_buf|")
        assert normalize(restored)==normalize(baseline),"HAL submission/callback behavior changed"
        assert source.count("add_completed_handler")==baseline.count("add_completed_handler"),"added callback"
        assert source.count(".commit()")==baseline.count(".commit()"),"submission changed"
        device=(a.hal/"src/metal/device.rs").read_text()
        before=subprocess.check_output(["git","-C",str(a.hal),"show","HEAD~1:src/metal/device.rs"],text=True)
        restored=strip_events(device).replace("let mut last_status = None;","")
        restored=re.sub(r"let status = cmd_buf.status\(\);\s*if last_status != Some\(status\) \{\s*last_status = Some\(status\);\s*\}\s*if let MTLCommandBufferStatus::Completed = status", "if let MTLCommandBufferStatus::Completed = cmd_buf.status()", restored)
        assert normalize(restored)==normalize(before),"HAL wait policy changed"
        assert device.count("cmd_buf.status()")==before.count("cmd_buf.status()"),"added native status query"
        assert device.count("thread::sleep")==before.count("thread::sleep"),"wait policy changed"
    print(json.dumps(dict(status="PASS",graph_versions_and_edges_unchanged=True,policy_negatives=3)))
if __name__=="__main__":main()
