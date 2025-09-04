param()

Write-Host "🧪 SHIMMY BASIC FUNCTIONAL TESTS" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

$TestResults = @()
$Port = 11439

function Test-Item($Name, $TestScript) {
    Write-Host "🔍 Testing: $Name" -ForegroundColor Yellow
    try {
        $result = & $TestScript
        if ($result) {
            Write-Host "✅ PASS: $Name" -ForegroundColor Green
            $TestResults += @{Name=$Name; Status="PASS"}
        } else {
            Write-Host "❌ FAIL: $Name" -ForegroundColor Red  
            $TestResults += @{Name=$Name; Status="FAIL"}
        }
    } catch {
        Write-Host "❌ ERROR: $Name - $($_.Exception.Message)" -ForegroundColor Red
        $TestResults += @{Name=$Name; Status="ERROR"}
    }
    Write-Host ""
}

# Test 1: Build
Test-Item "Cargo Build" {
    Write-Host "Building shimmy..."
    cargo build --release --all-features
    return $LASTEXITCODE -eq 0
}

# Test 2: CLI Help
Test-Item "CLI Help" {
    $help = cargo run --release --bin shimmy -- --help 2>&1
    return $help -like "*Usage:*"
}

# Test 3: CLI List
Test-Item "CLI List Models" {
    cargo run --release --bin shimmy -- list 2>&1
    return $LASTEXITCODE -eq 0
}

# Test 4: CLI Discover  
Test-Item "CLI Discover" {
    cargo run --release --bin shimmy -- discover 2>&1
    return $LASTEXITCODE -eq 0
}

# Test 5: Start Server
Write-Host "🚀 Starting Shimmy server on port $Port..." -ForegroundColor Cyan
$env:SHIMMY_BASE_GGUF = "test_model.gguf"
$ShimmyProcess = Start-Process -FilePath "cargo" -ArgumentList "run","--release","--bin","shimmy","--","serve","--bind","127.0.0.1:$Port" -PassThru -WindowStyle Hidden

Start-Sleep -Seconds 8

# Test 6: Health Check
Test-Item "Health Endpoint" {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/health" -Method GET -TimeoutSec 10
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

# Test 7: Models API
Test-Item "Models API" {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/api/models" -Method GET -TimeoutSec 10  
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

# Test 8: Generate API
Test-Item "Generate API" {
    try {
        $body = '{"model":"default","prompt":"Hello","max_tokens":10}'
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/api/generate" -Method POST -Body $body -ContentType "application/json" -TimeoutSec 15
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

# Test 9: OpenAI Compatibility
Test-Item "OpenAI API Compatibility" {
    try {
        $body = '{"model":"default","messages":[{"role":"user","content":"Hello"}]}'
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/v1/chat/completions" -Method POST -Body $body -ContentType "application/json" -TimeoutSec 15
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

# Cleanup
if ($ShimmyProcess -and !$ShimmyProcess.HasExited) {
    Write-Host "🛑 Stopping Shimmy server..." -ForegroundColor Yellow
    Stop-Process -Id $ShimmyProcess.Id -Force -ErrorAction SilentlyContinue
}

# Results Summary
Write-Host "📊 TEST RESULTS SUMMARY" -ForegroundColor Cyan
Write-Host "=======================" -ForegroundColor Cyan

$passCount = ($TestResults | Where-Object { $_.Status -eq "PASS" }).Count
$failCount = ($TestResults | Where-Object { $_.Status -eq "FAIL" }).Count  
$errorCount = ($TestResults | Where-Object { $_.Status -eq "ERROR" }).Count

foreach ($test in $TestResults) {
    if ($test.Status -eq "PASS") {
        Write-Host "✅ $($test.Name)" -ForegroundColor Green
    } elseif ($test.Status -eq "FAIL") {
        Write-Host "❌ $($test.Name)" -ForegroundColor Red
    } else {
        Write-Host "⚠️ $($test.Name)" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "📈 FINAL SUMMARY:" -ForegroundColor White
Write-Host "  ✅ Passed: $passCount" -ForegroundColor Green
Write-Host "  ❌ Failed: $failCount" -ForegroundColor Red
Write-Host "  ⚠️ Errors: $errorCount" -ForegroundColor Yellow

$totalTests = $TestResults.Count
if ($totalTests -gt 0) {
    $successRate = [math]::Round(($passCount / $totalTests) * 100, 1)
    Write-Host "  🎯 Success Rate: $successRate%" -ForegroundColor Green
} else {
    Write-Host "  🎯 Success Rate: 0%" -ForegroundColor Red
}

Write-Host ""
if ($failCount -eq 0 -and $errorCount -eq 0) {
    Write-Host "🚀 SHIMMY FUNCTIONAL TESTS PASSED!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "⚠️ Some tests failed - review required" -ForegroundColor Yellow
    exit 1
}