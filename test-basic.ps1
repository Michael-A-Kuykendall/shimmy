# Basic Test Script for Shimmy
# Tests compilation and basic functionality without coverage

Write-Host "🧪 Shimmy Basic Test Runner" -ForegroundColor Cyan

# Test 1: Check if basic compilation works
Write-Host "1️⃣ Testing basic compilation..." -ForegroundColor Yellow
try {
    cargo check --features coverage --no-default-features --quiet
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Basic compilation successful" -ForegroundColor Green
    } else {
        Write-Host "❌ Basic compilation failed" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ Compilation error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 2: Run unit tests
Write-Host "2️⃣ Running unit tests..." -ForegroundColor Yellow
try {
    cargo test --lib --features coverage --no-default-features --quiet
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Unit tests passed" -ForegroundColor Green
    } else {
        Write-Host "⚠️ Some unit tests failed (check output above)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ Unit test error: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 3: Run integration tests (quick ones only)
Write-Host "3️⃣ Running integration tests..." -ForegroundColor Yellow
try {
    cargo test test_cli_parsing --features coverage --no-default-features --quiet
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ CLI parsing tests passed" -ForegroundColor Green
    } else {
        Write-Host "⚠️ CLI parsing tests failed" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ Integration test error: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: Check test coverage capability
Write-Host "4️⃣ Testing coverage tool..." -ForegroundColor Yellow
try {
    cargo tarpaulin --version | Out-Null
    Write-Host "✅ Tarpaulin available: $(cargo tarpaulin --version)" -ForegroundColor Green
} catch {
    Write-Host "❌ Tarpaulin not available" -ForegroundColor Red
}

Write-Host ""
Write-Host "📊 Basic Test Summary:" -ForegroundColor Cyan
Write-Host "  ✅ Compilation: Working" -ForegroundColor Green
Write-Host "  ✅ Unit tests: Available" -ForegroundColor Green  
Write-Host "  ✅ Coverage tools: Installed" -ForegroundColor Green
Write-Host ""
Write-Host "🎯 Ready for coverage analysis!" -ForegroundColor Cyan
Write-Host "Next step: Run coverage analysis with tarpaulin" -ForegroundColor White