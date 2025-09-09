# Shimmy Test Coverage Script
# 
# This script provides comprehensive test coverage measurement for the shimmy project
# using cargo-tarpaulin with optimized settings for the project structure.

param(
    [string]$OutputFormat = "Html",
    [string]$OutputDir = "coverage", 
    [switch]$OpenReport = $false,
    [switch]$Fast = $false,
    [switch]$AllFeatures = $false
)

Write-Host "🎯 Shimmy Test Coverage Analysis" -ForegroundColor Cyan

# Ensure coverage directory exists
if (!(Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    Write-Host "✅ Created coverage output directory: $OutputDir" -ForegroundColor Green
}

# Configure coverage command based on parameters
$CoverageCmd = "cargo tarpaulin"
$CoverageArgs = @()

if ($Fast) {
    Write-Host "🚀 Using fast coverage mode (huggingface only)" -ForegroundColor Yellow
    $CoverageArgs += "--features", "coverage", "--no-default-features"
} elseif ($AllFeatures) {
    Write-Host "🔥 Using full feature coverage (may be slow)" -ForegroundColor Yellow
    $CoverageArgs += "--features", "full"
} else {
    Write-Host "⚡ Using default coverage mode" -ForegroundColor Yellow
    $CoverageArgs += "--features", "fast"
}

# Add output configuration
$CoverageArgs += "--out", $OutputFormat
$CoverageArgs += "--output-dir", $OutputDir

# Exclusions to speed up coverage
$CoverageArgs += "--exclude-files", "target/*"
$CoverageArgs += "--exclude-files", "test-models/*"  
$CoverageArgs += "--exclude-files", "models/*"
$CoverageArgs += "--exclude-files", "*/build.rs"

# Coverage timeout
$CoverageArgs += "--timeout", "120"

# Verbose output for debugging
$CoverageArgs += "--verbose"

Write-Host "Running: $CoverageCmd $($CoverageArgs -join ' ')" -ForegroundColor Cyan

try {
    # Run coverage analysis
    & $CoverageCmd @CoverageArgs
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Coverage analysis completed successfully!" -ForegroundColor Green
        
        # Check for generated reports
        $HtmlReport = Join-Path $OutputDir "tarpaulin-report.html"
        $XmlReport = Join-Path $OutputDir "cobertura.xml"
        
        if (Test-Path $HtmlReport) {
            Write-Host "📊 HTML Report: $HtmlReport" -ForegroundColor Green
            if ($OpenReport) {
                Start-Process $HtmlReport
            }
        }
        
        if (Test-Path $XmlReport) {
            Write-Host "📊 XML Report: $XmlReport" -ForegroundColor Green
        }
        
        Write-Host ""
        Write-Host "Coverage analysis complete! Use the following commands:" -ForegroundColor Cyan
        Write-Host "  - View HTML report: .\coverage.ps1 -OpenReport" -ForegroundColor White
        Write-Host "  - Fast coverage: .\coverage.ps1 -Fast" -ForegroundColor White
        Write-Host "  - Full coverage: .\coverage.ps1 -AllFeatures" -ForegroundColor White
        
    } else {
        Write-Host "❌ Coverage analysis failed with exit code: $LASTEXITCODE" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} catch {
    Write-Host "❌ Error running coverage analysis: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}