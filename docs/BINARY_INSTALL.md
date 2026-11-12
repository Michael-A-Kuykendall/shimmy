# Verified native downloads — Shimmy 2.6.7

[Release and all checksums](https://github.com/Michael-A-Kuykendall/shimmy/releases/tag/v2.6.7). Download the self-contained archive for your operating system/architecture; it retains the default Airframe engine, component licenses, libfse additional Apache-2.0 grant, notices and build identity. Preserve those licensing files with redistribution. See [stack licensing](../LICENSING.md). No separate libfse commercial permission is required.

| Platform | Licensed archive | SHA-256 |
|---|---|---|
| Linux x86-64 | shimmy-linux-x86_64.tar.gz | `ce23fcb71f56e5d84b5848d2f56a5aded40ad37e0a9b5238ce36da487d39c312` |
| Linux ARM64 | shimmy-linux-arm64.tar.gz | `c560fc3214984deb217eb96859e5b0ccd3a96130032a78963642769f208b229d` |
| macOS ARM64 | shimmy-macos-arm64.tar.gz | `989acb48fa452fd0f7992c1c968752ab8e22772d47ac1770fcd54dc64fbcd694` |
| Windows x86-64 | shimmy-windows-x86_64.zip | `eb1f6b0a67b4988f4d77d60e2b0040156c816a9eb2ad963907ef55fe62a87d7d` |

SHA256SUMS covers the four raw executables. BUNDLE-SHA256SUMS covers the archives above. RELEASE-SHA256SUMS covers companion metadata. These are separate from immutable crates.io package checksums for Shimmy 2.6.7, Airframe 0.4.6 and the four unchanged supporting crates.

## Linux x86-64 example

```bash
curl -fLO https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.7/shimmy-linux-x86_64.tar.gz
printf '%s  %s\n' ce23fcb71f56e5d84b5848d2f56a5aded40ad37e0a9b5238ce36da487d39c312 shimmy-linux-x86_64.tar.gz | sha256sum -c -
tar -xzf shimmy-linux-x86_64.tar.gz
./shimmy --version
./shimmy --help
```

For Linux ARM64 use its matching filename/checksum. On macOS ARM64 use its matching archive and `shasum -a 256` to compare the hash before extraction. Put the executable and licensing materials in your chosen installation directory; add that directory to PATH if desired. No automatic PATH change is made by extracting an archive.

## Windows PowerShell

Use Invoke-WebRequest explicitly; PowerShell's curl alias does not support curl's -L flag.

```powershell
$shimmyDir = Join-Path $env:TEMP "shimmy-2.6.7"
New-Item -ItemType Directory -Force $shimmyDir | Out-Null
$shimmyZip = Join-Path $shimmyDir "shimmy-windows-x86_64.zip"
Invoke-WebRequest -UseBasicParsing -Uri "https://github.com/Michael-A-Kuykendall/shimmy/releases/download/v2.6.7/shimmy-windows-x86_64.zip" -OutFile $shimmyZip
if ((Get-FileHash $shimmyZip -Algorithm SHA256).Hash.ToLowerInvariant() -ne "eb1f6b0a67b4988f4d77d60e2b0040156c816a9eb2ad963907ef55fe62a87d7d") { throw "Download checksum mismatch" }
Expand-Archive -Path $shimmyZip -DestinationPath (Join-Path $shimmyDir "installed") -Force
& (Join-Path $shimmyDir "installed\shimmy.exe") --version
& (Join-Path $shimmyDir "installed\shimmy.exe") --help
```

Keep the extracted LICENSES directory and notices beside the executable. Choose your own permanent location/PATH configuration if desired. [Windows build/driver scope](WINDOWS_GPU_BUILD_GUIDE.md) is separate from this executable installation.

## What the tests establish

Native tests, default-Airframe builds, installation, CLI version/help, architecture and license checks passed for all four targets in the [frozen 2.6.7 release](https://github.com/Michael-A-Kuykendall/shimmy/releases/tag/v2.6.7), with exact per-binary SHA256SUMS. The final macOS ARM64 native job passed 694 default and 665 fast-profile tests. These native CLI/build results are not GPU inference certification. Separately, Apple M1 (Virtual) / Paravirtual Metal passed **198/198** full-vector FP32 comparisons and actual HTTP/state gates with the released Airframe 0.4.6 source. **Physical M4, DGX Spark, INT4 and other unexecuted configurations remain NOT_RUN**. WSL destructor/device-loss investigations remain open.

## Homebrew

The repository provides a versioned custom tap (not Homebrew core):

```bash
brew tap Michael-A-Kuykendall/shimmy https://github.com/Michael-A-Kuykendall/shimmy.git
brew install Michael-A-Kuykendall/shimmy/shimmy
brew test Michael-A-Kuykendall/shimmy/shimmy
```

The formula selects the exact licensed archives for macOS ARM64 and Linux x86-64/ARM64. Linux x86-64 and macOS ARM64 Homebrew installs were verified; Linux ARM64 Homebrew installation remains NOT_RUN (its native binary was separately validated). macOS Intel has no release binary. Licenses and the libfse grant are retained under the formula prefix at share/shimmy. This custom formula does not certify additional GPU configurations.

[Versioned Docker/GHCR instructions](../README-DOCKER.md) are separate from native downloads; Docker latest remains unchanged. Scoop installation is described below. npm is not included.

## Scoop (Windows x86-64)

The repository provides an explicit custom bucket (not Scoop main):

```powershell
scoop bucket add shimmy https://github.com/Michael-A-Kuykendall/shimmy
scoop install shimmy/shimmy
shimmy --version
shimmy --help
```

The 2.6.7 manifest selects the exact Windows ZIP and checksum above. It retains LICENSES, the libfse additional grant and notices in the installed app directory. ARM Windows and native Windows GPU inference are not certified by the x86-64 CLI installation test. No npm package is implied.

Historical 2.6.6 binaries and their different checksums remain available on the [v2.6.6 release page](https://github.com/Michael-A-Kuykendall/shimmy/releases/tag/v2.6.6). Do not use older checksums when installing 2.6.7.
