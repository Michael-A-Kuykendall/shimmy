# Verified native downloads — Shimmy 2.6.6

[Release and all checksums](https://github.com/Michael-A-Kuykendall/shimmy/releases/tag/v2.6.6). Download the self-contained archive for your operating system/architecture; it retains the default Airframe engine, component licenses, libfse additional Apache-2.0 grant, notices and build identity. Preserve those licensing files with redistribution. See [stack licensing](../LICENSING.md). No separate libfse commercial permission is required.

| Platform | Licensed archive | SHA-256 |
|---|---|---|
| Linux x86-64 | shimmy-linux-x86_64.tar.gz | `13c5ea0b7c32e7021542a94ad961347b6b2a86deb834a1652f6e93e0e7204cf9` |
| Linux ARM64 | shimmy-linux-arm64.tar.gz | `9916f1279d1a2d3eced746836fecbbe277f4816d21e06da45ebc6596f22f812e` |
| macOS ARM64 | shimmy-macos-arm64.tar.gz | `eabba8acf94450242f0b8da221a391a935185b3c3877f1350b1ff7ffec2d4a2e` |
| Windows x86-64 | shimmy-windows-x86_64.zip | `6c7b854ba851f54347601827d5948c661283ae0a60f03c9a9f6e0ec7ddfd6dd4` |

SHA256SUMS covers the four raw executables. BUNDLE-SHA256SUMS covers the archives above. RELEASE-SHA256SUMS covers companion metadata. These are separate from the five immutable crates.io package checksums.

## Linux x86-64 example

```bash
curl -fLO https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.6/shimmy-linux-x86_64.tar.gz
printf '%s  %s\n' 13c5ea0b7c32e7021542a94ad961347b6b2a86deb834a1652f6e93e0e7204cf9 shimmy-linux-x86_64.tar.gz | sha256sum -c -
tar -xzf shimmy-linux-x86_64.tar.gz
./shimmy --version
./shimmy --help
```

For Linux ARM64 use its matching filename/checksum. On macOS ARM64 use its matching archive and `shasum -a 256` to compare the hash before extraction. Put the executable and licensing materials in your chosen installation directory; add that directory to PATH if desired. No automatic PATH change is made by extracting an archive.

## Windows PowerShell

Use Invoke-WebRequest explicitly; PowerShell's curl alias does not support curl's -L flag.

```powershell
$shimmyDir = Join-Path $env:TEMP "shimmy-2.6.6"
New-Item -ItemType Directory -Force $shimmyDir | Out-Null
$shimmyZip = Join-Path $shimmyDir "shimmy-windows-x86_64.zip"
Invoke-WebRequest -UseBasicParsing -Uri "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.6/shimmy-windows-x86_64.zip" -OutFile $shimmyZip
if ((Get-FileHash $shimmyZip -Algorithm SHA256).Hash.ToLowerInvariant() -ne "6c7b854ba851f54347601827d5948c661283ae0a60f03c9a9f6e0ec7ddfd6dd4") { throw "Download checksum mismatch" }
Expand-Archive -Path $shimmyZip -DestinationPath (Join-Path $shimmyDir "installed") -Force
& (Join-Path $shimmyDir "installed\shimmy.exe") --version
& (Join-Path $shimmyDir "installed\shimmy.exe") --help
```

Keep the extracted LICENSES directory and notices beside the executable. Choose your own permanent location/PATH configuration if desired. [Windows build/driver scope](WINDOWS_GPU_BUILD_GUIDE.md) is separate from this executable installation.

## What the tests establish

Native unit tests, default-engine build, install, CLI version/help and architecture checks passed for all four targets in [the selected manual run](https://github.com/Michael-A-Kuykendall/shimmy/actions/runs/42404833070). This is not native GPU model certification. An Apple Paravirtual Metal diagnostic produced correct Qwen3-0.6B arithmetic/JSON/usage; its extra bare-number assertion was corrected by local evaluation of preserved outputs. Full-vector Metal/M4, DGX Spark, INT4 and other unexecuted modes remain NOT_RUN. WSL destructor/device-loss investigations remain open.

Docker/GHCR, Homebrew and Scoop have separate publication/installation gates; a GitHub executable release does not automatically update them. npm is not included.
