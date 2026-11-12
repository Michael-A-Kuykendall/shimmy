# Build portability — Shimmy 2.6.7

Rust/wgpu portability is not executed native GPU certification. The release
does not claim Windows/macOS/Linux/ARM inference correctness merely because a
target compiles. Apple M1 (Virtual) / Paravirtual Metal numerical results are certified for the exact tested 2.6.7/0.4.6 fixtures; physical M4 and other unavailable native GPU configurations remain NOT_RUN.

The default feature includes Airframe; no-default-features fast is useful for
non-inference checks and does not provide a CPU engine. Deprecated llama feature
names are empty compatibility stubs.

Cross compilation requires the Rust target plus its actual linker/system
dependencies. Run native package/install/CLI/protocol and model tests on the
target before making support claims. Hosted macOS ARM64 runners can exercise Apple Paravirtual Metal, as the bounded M1 numerical certification demonstrated; they do not replace physical M4 or certify all Macs.
No hosted build, tag or release is authorized by this guide.

Shimmy 2.6.7 ships independently verified GitHub native binaries; a crates.io upload alone does not create them. Any later
binary distribution needs its own exact source/build/hash verification.
See [release gates](../RELEASE_GATES_CHECKLIST.md), [Windows scope](WINDOWS_GPU_BUILD_GUIDE.md)
and [fixtures](SUPPORTED_MODELS.md).
