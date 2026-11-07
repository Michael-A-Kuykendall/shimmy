# Build portability — Shimmy 2.6.6

Rust/wgpu portability is not executed native GPU certification. The release
does not claim Windows/macOS/Linux/ARM inference correctness merely because a
target compiles. Metal/M4 and unavailable native GPU configurations are NOT_RUN.

The default feature includes Airframe; no-default-features fast is useful for
non-inference checks and does not provide a CPU engine. Deprecated llama feature
names are empty compatibility stubs.

Cross compilation requires the Rust target plus its actual linker/system
dependencies. Run native package/install/CLI/protocol and model tests on the
target before making support claims. CI cannot emulate Apple GPU execution.
No hosted build, tag or release is authorized by this guide.

A crates.io-only release creates no new GitHub platform binaries. Any later
binary distribution needs its own exact source/build/hash verification.
See [release gates](../RELEASE_GATES_CHECKLIST.md), [Windows scope](WINDOWS_GPU_BUILD_GUIDE.md)
and [fixtures](SUPPORTED_MODELS.md).
