# Basic Test Script for Shimmy
Write-Host "Testing Shimmy basic functionality..." -ForegroundColor Cyan

# Test compilation
Write-Host "1. Testing compilation..." -ForegroundColor Yellow
cargo check --features coverage --no-default-features --quiet
if ($LASTEXITCODE -eq 0) {
    Write-Host "   Compilation: SUCCESS" -ForegroundColor Green
} else {
    Write-Host "   Compilation: FAILED" -ForegroundColor Red
}

# Test unit tests
Write-Host "2. Testing unit tests..." -ForegroundColor Yellow
cargo test --lib --features coverage --no-default-features --quiet
if ($LASTEXITCODE -eq 0) {
    Write-Host "   Unit tests: SUCCESS" -ForegroundColor Green
} else {
    Write-Host "   Unit tests: FAILED or WARNINGS" -ForegroundColor Yellow
}

# Check coverage tool
Write-Host "3. Checking coverage tool..." -ForegroundColor Yellow
$version = cargo tarpaulin --version 2>$null
if ($version) {
    Write-Host "   Tarpaulin: Available ($version)" -ForegroundColor Green
} else {
    Write-Host "   Tarpaulin: NOT AVAILABLE" -ForegroundColor Red
}

Write-Host ""
Write-Host "Basic testing complete!" -ForegroundColor Cyan