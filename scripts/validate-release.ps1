# SHIMMY RELEASE VALIDATION - MUST PASS BEFORE ANY RELEASE
# Exit 0 = Ready, Exit 1 = BLOCKED

$ErrorActionPreference = "Continue"
$failures = 0

function Fail($msg) { Write-Host "FAIL: $msg" -ForegroundColor Red; $script:failures++ }
function Pass($msg) { Write-Host "PASS: $msg" -ForegroundColor Green }
function Info($msg) { Write-Host "INFO: $msg" -ForegroundColor Blue }

Write-Host "Shimmy Release Validation" -ForegroundColor Cyan

# 1. CRITICAL COMPILATION TESTS
Info "Testing core compilation..."
cargo build --release --no-default-features --features huggingface
if ($LASTEXITCODE -eq 0) { Pass "Core build succeeded" } else { Fail "Core build failed" }

Info "Testing CUDA compilation (3min timeout)..."
$job = Start-Job { cargo build --release --no-default-features --features llama-cuda }
if (Wait-Job $job -Timeout 180) {
    if ($job.State -eq "Completed") { Pass "CUDA build succeeded" }
    else { Fail "CUDA build failed" }
} else { 
    Stop-Job $job; Fail "CUDA build timeout (over 3min)"
}
Remove-Job $job

# 2. BINARY VALIDATION
$binary = "target/release/shimmy.exe"
if (Test-Path $binary) {
    $sizeMB = [math]::Round((Get-Item $binary).Length / 1MB, 1)
    if ($sizeMB -lt 20) { Pass "Binary size: ${sizeMB}MB under 20MB limit" }
    else { Fail "Binary too large: ${sizeMB}MB" }
} else { Fail "Binary not found" }

# 3. ESSENTIAL COMMANDS  
Info "Testing essential commands..."
& $binary --version | Out-Null
if ($LASTEXITCODE -eq 0) { Pass "Version command works" } else { Fail "Version command failed" }

& $binary --help | Out-Null
if ($LASTEXITCODE -eq 0) { Pass "Help command works" } else { Fail "Help command failed" }

# 4. TEMPLATE PACKAGING (Issue #60)
Info "Validating template packaging..."
$packageList = cargo package --list --allow-dirty 2>&1 | Out-String
if ($packageList -match "templates/docker/Dockerfile") { Pass "Dockerfile included in package" }
else { Fail "Dockerfile missing from package" }

# 5. TEMPLATE GENERATION TEST
$tempDir = "temp-test-$(Get-Random)"
& $binary init docker $tempDir | Out-Null
if ((Test-Path "$tempDir/Dockerfile") -and ($LASTEXITCODE -eq 0)) {
    Pass "Template generation works"
} else { Fail "Template generation failed" }
Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue

# 6. CODE QUALITY
Info "Testing code quality..."
cargo clippy --all-features -- -D warnings | Out-Null
if ($LASTEXITCODE -eq 0) { Pass "Clippy checks pass" } else { Fail "Clippy warnings found" }

cargo fmt -- --check | Out-Null
if ($LASTEXITCODE -eq 0) { Pass "Code formatting OK" } else { Fail "Code formatting issues" }

# SUMMARY
Write-Host ""
if ($failures -eq 0) {
    Write-Host "ALL VALIDATIONS PASSED - READY FOR RELEASE" -ForegroundColor Green
    exit 0
} else {
    Write-Host "$failures FAILURES - BLOCKED" -ForegroundColor Red
    exit 1
}