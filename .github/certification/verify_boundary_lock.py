"""Ensure diagnostic source substitutions do not upgrade dependency versions."""
import pathlib,tomllib
HERE=pathlib.Path(__file__).resolve().parent
REGISTRY="registry+https://github.com/rust-lang/crates.io-index"
def verify_locks():
    baseline=tomllib.loads((HERE/"probes/Cargo.lock").read_text())["package"]
    diagnostic=tomllib.loads((HERE/"probes/diagnostic.Cargo.lock").read_text())["package"]
    def registry(packages):
        result=set()
        for p in packages:
            if "source" in p:
                assert p["source"]==REGISTRY,"unofficial dependency source"
                result.add((p["name"],p["version"],p["checksum"]))
        return result
    before=registry(baseline);after=registry(diagnostic)
    removed={p for p in before if p[0] in {"airframe","libfse"}}
    assert after==before-removed,"unrelated dependency drift"
    assert {(p["name"],p["version"]) for p in diagnostic if "source" not in p}=={
        ("shimmy-metal-certification","0.0.0"),("airframe","0.4.5"),("libfse","0.1.4"),("airframe-observe","0.1.4")}
    assert {(p["name"],p["version"]) for p in diagnostic}=={(p["name"],p["version"]) for p in baseline}
    return True
