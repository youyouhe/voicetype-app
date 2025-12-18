# Quick fix for PowerShell execution policy issue
Write-Host "Fixing PowerShell execution policy for npm..." -ForegroundColor Green

# Set execution policy for current user only
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force

Write-Host "Execution policy set to: $(Get-ExecutionPolicy)" -ForegroundColor Yellow

# Test npm
try {
    Write-Host "Testing npm command..." -ForegroundColor Cyan
    $version = npm -v
    Write-Host "SUCCESS: npm version $version" -ForegroundColor Green
} catch {
    Write-Host "ERROR: npm command failed" -ForegroundColor Red
    Write-Host "Please try using npm.cmd instead of npm" -ForegroundColor Yellow
}

Write-Host "Fix complete! Please restart PowerShell and try npm -v again." -ForegroundColor Green