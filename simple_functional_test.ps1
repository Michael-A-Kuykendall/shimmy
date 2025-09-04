#!/usr/bin/env pwsh
# Shimmy Functional Test Suite - Simple Version
# Tests all adapter/interface combinations for production readiness

Write-Host "🧪 SHIMMY FUNCTIONAL VERIFICATION SUITE" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

$TestResults = @{}
$Port = 11437  # Use non-standard port for testing

function Test-Component {
    param([string]$Name, [scriptblock]$TestCode)
    
    Write-Host "🔍 Testing: $Name" -ForegroundColor Yellow
    try {
        $result = & $TestCode
        if ($result) {
            Write-Host "✅ PASS: $Name" -ForegroundColor Green
            $TestResults[$Name] = "PASS"
        } else {
            Write-Host "❌ FAIL: $Name" -ForegroundColor Red
            $TestResults[$Name] = "FAIL"
        }
    } catch {
        Write-Host "❌ ERROR: $Name - $($_.Exception.Message)" -ForegroundColor Red
        $TestResults[$Name] = "ERROR"
    }
    Write-Host ""
}

# PHASE 1: Build and Basic CLI Tests
Write-Host "📦 PHASE 1: BUILD AND CLI VERIFICATION" -ForegroundColor Magenta

Test-Component "Cargo Build" {
    $result = & cargo build --release --all-features
    return $LASTEXITCODE -eq 0
}

Test-Component "CLI Help Command" {
    $help = & cargo run --release --bin shimmy -- --help
    return $help -like "*Usage:*"
}

Test-Component "CLI List Models" {
    $list = & cargo run --release --bin shimmy -- list
    return $LASTEXITCODE -eq 0
}

Test-Component "CLI Discover Models" {
    $discover = & cargo run --release --bin shimmy -- discover
    return $LASTEXITCODE -eq 0
}

# PHASE 2: Server Tests
Write-Host "🌐 PHASE 2: SERVER AND API VERIFICATION" -ForegroundColor Magenta

# Start shimmy server for API tests
Write-Host "🚀 Starting Shimmy server on port $Port..."
$env:SHIMMY_BASE_GGUF = "test_model.gguf"
$ShimmyProcess = Start-Process -FilePath "cargo" -ArgumentList "run","--release","--bin","shimmy","--","serve","--bind","127.0.0.1:$Port" -PassThru -WindowStyle Hidden

# Wait for server to start
Start-Sleep -Seconds 5

Test-Component "Health Endpoint" {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/health" -Method GET -TimeoutSec 5
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

Test-Component "Models Endpoint" {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/api/models" -Method GET -TimeoutSec 5
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

Test-Component "Generate Endpoint" {
    try {
        $body = @{
            model = "default"
            prompt = "Hello"
            max_tokens = 10
        } | ConvertTo-Json
        
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/api/generate" -Method POST -Body $body -ContentType "application/json" -TimeoutSec 10
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

Test-Component "OpenAI Chat Completions" {
    try {
        $body = @{
            model = "default"
            messages = @(
                @{ role = "user"; content = "Say hello" }
            )
        } | ConvertTo-Json -Depth 3
        
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/v1/chat/completions" -Method POST -Body $body -ContentType "application/json" -TimeoutSec 10
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

Test-Component "Streaming Response" {
    try {
        $body = @{
            model = "default"
            prompt = "Count 1, 2, 3"
            max_tokens = 20
            stream = $true
        } | ConvertTo-Json
        
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/api/generate" -Method POST -Body $body -ContentType "application/json" -TimeoutSec 10
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

# PHASE 3: Engine Adapter Tests  
Write-Host "🔧 PHASE 3: ENGINE ADAPTER VERIFICATION" -ForegroundColor Magenta

Test-Component "Engine Registry" {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/api/models" -Method GET -TimeoutSec 5
        $data = $response.Content | ConvertFrom-Json
        return $data.data.Count -gt 0
    } catch {
        return $false
    }
}

# PHASE 4: Performance Tests
Write-Host "⚡ PHASE 4: PERFORMANCE VERIFICATION" -ForegroundColor Magenta

Test-Component "Server Response Time" {
    try {
        $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/health" -Method GET -TimeoutSec 5
        $stopwatch.Stop()
        $responseTime = $stopwatch.ElapsedMilliseconds
        Write-Host "Response time: ${responseTime}ms"
        return $responseTime -lt 1000  # Should respond in under 1 second
    } catch {
        return $false
    }
}

Test-Component "Memory Usage Check" {
    if ($ShimmyProcess) {
        try {
            $process = Get-Process -Id $ShimmyProcess.Id -ErrorAction Stop
            $memoryMB = [math]::Round($process.WorkingSet64 / 1MB, 2)
            Write-Host "Memory usage: ${memoryMB}MB"
            return $memoryMB -lt 50  # Should use less than 50MB
        } catch {
            return $false
        }
    }
    return $false
}

# PHASE 5: Integration Tests
Write-Host "🔗 PHASE 5: INTEGRATION VERIFICATION" -ForegroundColor Magenta

Test-Component "VSCode Extension Compatibility" {
    $packageJson = "shimmy-vscode/package.json"
    if (Test-Path $packageJson) {
        $content = Get-Content $packageJson | ConvertFrom-Json
        return $content.name -eq "shimmy-vscode"
    }
    return $false
}

Test-Component "OpenAI API Schema Compliance" {
    try {
        $body = @{
            model = "default"
            messages = @(@{ role = "user"; content = "test" })
            temperature = 0.7
        } | ConvertTo-Json -Depth 3
        
        $response = Invoke-WebRequest -Uri "http://localhost:$Port/v1/chat/completions" -Method POST -Body $body -ContentType "application/json" -TimeoutSec 10
        $data = $response.Content | ConvertFrom-Json
        return $data.choices -and $data.choices.Count -gt 0
    } catch {
        return $false
    }
}

# Cleanup
if ($ShimmyProcess) {
    Write-Host "🛑 Stopping Shimmy server..." -ForegroundColor Yellow
    Stop-Process -Id $ShimmyProcess.Id -Force -ErrorAction SilentlyContinue
}

# RESULTS SUMMARY
Write-Host "📊 FUNCTIONAL TEST RESULTS" -ForegroundColor Cyan
Write-Host "==========================" -ForegroundColor Cyan

$passCount = 0
$failCount = 0
$errorCount = 0

foreach ($test in $TestResults.GetEnumerator()) {
    $name = $test.Key
    $status = $test.Value
    
    if ($status -eq "PASS") {
        Write-Host "✅ $name" -ForegroundColor Green
        $passCount++
    } elseif ($status -eq "FAIL") {
        Write-Host "❌ $name" -ForegroundColor Red
        $failCount++
    } else {
        Write-Host "⚠️  $name ($status)" -ForegroundColor Yellow
        $errorCount++
    }
}

Write-Host ""
Write-Host "📈 SUMMARY:" -ForegroundColor White
Write-Host "  ✅ Passed: $passCount" -ForegroundColor Green
Write-Host "  ❌ Failed: $failCount" -ForegroundColor Red
Write-Host "  ⚠️  Errors: $errorCount" -ForegroundColor Yellow

$successRate = if ($TestResults.Count -gt 0) { [math]::Round(($passCount / $TestResults.Count) * 100, 1) } else { 0 }
Write-Host "  🎯 Success Rate: $successRate%" -ForegroundColor $(if ($successRate -gt 80) { "Green" } elseif ($successRate -gt 60) { "Yellow" } else { "Red" })

Write-Host ""
if ($failCount -eq 0) {
    Write-Host "🚀 SHIMMY IS PRODUCTION READY!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "⚠️ Some tests failed - review before deployment" -ForegroundColor Yellow
    exit 1
}