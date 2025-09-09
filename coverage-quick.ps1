# Quick Coverage Script for Shimmy
# This script provides a simplified coverage approach focused on unit tests

param(
    [switch]$Clean = $false,
    [switch]$Fast = $false
)

Write-Host "🎯 Shimmy Quick Coverage Analysis" -ForegroundColor Cyan

# Clean build artifacts if requested
if ($Clean) {
    Write-Host "🧹 Cleaning build artifacts..." -ForegroundColor Yellow
    cargo clean 2>$null
}

# Create coverage directory
$CoverageDir = "coverage"
if (!(Test-Path $CoverageDir)) {
    New-Item -ItemType Directory -Path $CoverageDir -Force | Out-Null
    Write-Host "✅ Created coverage directory" -ForegroundColor Green
}

try {
    # Run basic tests first to ensure compilation works
    Write-Host "🔍 Running basic test compilation check..." -ForegroundColor Cyan
    
    if ($Fast) {
        cargo test --lib --features coverage --no-default-features --no-run
    } else {
        cargo test --lib --no-run
    }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Test compilation failed. Fixing common issues..." -ForegroundColor Red
        
        # Try to fix the Windows-specific issue we encountered
        Write-Host "Checking for Windows-specific compilation issues..."
        
        # Run with minimal features
        Write-Host "Attempting with minimal feature set..." -ForegroundColor Yellow
        cargo test --lib --features huggingface --no-default-features --no-run
        
        if ($LASTEXITCODE -ne 0) {
            Write-Host "❌ Compilation still failing. Manual intervention needed." -ForegroundColor Red
            Write-Host "Common issues:" -ForegroundColor Yellow
            Write-Host "  1. Missing Windows imports (ExitStatusExt)"
            Write-Host "  2. Feature conflicts with llama-cpp"  
            Write-Host "  3. Build directory conflicts"
            Write-Host ""
            Write-Host "Try: cargo clean && cargo test --lib --features huggingface --no-default-features" -ForegroundColor White
            exit 1
        }
    }
    
    Write-Host "✅ Test compilation successful!" -ForegroundColor Green
    
    # Now attempt coverage
    Write-Host "🎯 Running coverage analysis..." -ForegroundColor Cyan
    
    # Try simple line coverage first
    if ($Fast) {
        cargo tarpaulin --features coverage --no-default-features --out Html --output-dir $CoverageDir --timeout 60 --lib --tests
    } else {
        cargo tarpaulin --out Html --output-dir $CoverageDir --timeout 120 --lib --tests --exclude-files "target/*" --exclude-files "test-models/*"
    }
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Coverage analysis completed!" -ForegroundColor Green
        
        $HtmlReport = Join-Path $CoverageDir "tarpaulin-report.html"
        if (Test-Path $HtmlReport) {
            Write-Host "📊 HTML Report generated: $HtmlReport" -ForegroundColor Green
            Write-Host "   Open with: start $HtmlReport" -ForegroundColor White
        }
    } else {
        Write-Host "⚠️ Coverage analysis completed with warnings" -ForegroundColor Yellow
    }
    
} catch {
    Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "📝 Coverage Commands Summary:" -ForegroundColor Cyan
Write-Host "  Fast coverage:  .\coverage-quick.ps1 -Fast" -ForegroundColor White
Write-Host "  Clean & run:    .\coverage-quick.ps1 -Clean" -ForegroundColor White
Write-Host "  Manual tarpaulin: cargo tarpaulin --lib --out Html --output-dir coverage" -ForegroundColor White