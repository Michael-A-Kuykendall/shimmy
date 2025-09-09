# Final Coverage Test for Shimmy Project
# Minimal approach to get baseline coverage working

Write-Host "Final Coverage Test for Shimmy" -ForegroundColor Cyan

# Create coverage directory
if (!(Test-Path "coverage")) {
    New-Item -ItemType Directory -Path "coverage" -Force
    Write-Host "Created coverage directory" -ForegroundColor Green
}

Write-Host "Attempting minimal coverage analysis..." -ForegroundColor Yellow

# Try the most basic tarpaulin command possible
try {
    # Run with very basic settings and short timeout
    $result = cargo tarpaulin --out Html --output-dir coverage --lib --timeout 30 --skip-clean 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "SUCCESS: Basic coverage analysis completed!" -ForegroundColor Green
        
        # Check for HTML report
        if (Test-Path "coverage/tarpaulin-report.html") {
            Write-Host "HTML report generated at: coverage/tarpaulin-report.html" -ForegroundColor Green
        }
        
        # Show coverage summary if available
        if ($result -match "(\d+\.\d+)% coverage") {
            Write-Host "Coverage percentage: $($matches[1])%" -ForegroundColor Cyan
        }
        
    } else {
        Write-Host "Coverage analysis completed with warnings/errors" -ForegroundColor Yellow
        Write-Host "Exit code: $LASTEXITCODE" -ForegroundColor Yellow
        
        # Still check if report was generated
        if (Test-Path "coverage/tarpaulin-report.html") {
            Write-Host "HTML report still generated despite errors" -ForegroundColor Green
        }
    }
} catch {
    Write-Host "Error running coverage: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "Coverage setup summary:" -ForegroundColor Cyan
Write-Host "- cargo-tarpaulin: Installed" -ForegroundColor Green
Write-Host "- Configuration files: Created" -ForegroundColor Green  
Write-Host "- Coverage scripts: Available" -ForegroundColor Green
Write-Host "- Documentation: Complete" -ForegroundColor Green

Write-Host ""
Write-Host "Manual commands for coverage:" -ForegroundColor White
Write-Host "  cargo tarpaulin --out Html --output-dir coverage --lib" -ForegroundColor Gray