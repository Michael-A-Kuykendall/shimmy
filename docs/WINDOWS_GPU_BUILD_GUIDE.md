# Windows GPU build scope — Shimmy 2.6.6

A separately built [Windows 2.6.6 executable/archive is available with tested PowerShell installation and hashes](BINARY_INSTALL.md#windows-powershell). Use Invoke-WebRequest explicitly; no curl -L alias is needed. Native build/unit/CLI checks passed; this does not certify native Windows GPU inference.

A Rust build, adapter detection, WSL run and native Windows GPU certification
are different results. This release has NVIDIA WSL Dozen/Vulkan numerical
evidence; native Windows GPU correctness/stability is **NOT_RUN**.

A native Rust toolchain and its Windows
linker prerequisites can build the default engine:

```powershell
cargo install shimmy --version 2.6.6 --locked
shimmy --version
shimmy gpu-info
```

Adapter detection is not a numerical test. A working vendor driver/device
configuration is still required. --gpu-backend is not a supported way to force
CUDA/Vulkan/OpenCL in default Airframe. Empty legacy Cargo feature stubs do not
restore llama.cpp. A no-default-features fast build does not provide CPU
inference.

Use [quickstart](quickstart.md), [configuration](CONFIGURATION.md) and the
[exact fixtures](SUPPORTED_MODELS.md). In PowerShell use curl.exe when invoking
curl flags rather than an alias for Invoke-WebRequest. Do not assume fresh
Windows binaries were created by a crates.io-only release.

WSL destructor/device-loss investigations remain open. Smaller prefill chunks,
INT4, Docker and successful compilation do not certify a fix or emulate Metal.
