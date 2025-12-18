@echo off
chcp 65001 >nul
echo ========================================
echo PowerShell Execution Policy Fix Tool
echo ========================================
echo.

echo Fixing PowerShell execution policy issue...
echo.

echo Method 1: Setting execution policy for current user...
powershell -Command "Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force"

echo.
echo Verifying execution policy...
powershell -Command "Get-ExecutionPolicy"

echo.
echo Method 2: Testing npm command...
echo Trying to execute npm -v...
powershell -Command "npm -v"

if %ERRORLEVEL% EQU 0 (
    echo SUCCESS: npm command is working!
) else (
    echo ERROR: npm command still failing, trying method 3...
    echo.
    echo Trying npm.cmd instead of npm...
    npm.cmd -v

    if %ERRORLEVEL% EQU 0 (
        echo SUCCESS: npm.cmd is working!
        echo INFO: Please use npm.cmd instead of npm
    ) else (
        echo ERROR: npm.cmd also failed
        echo.
        echo Manual solutions:
        echo 1. Open PowerShell as Administrator
        echo 2. Run: Set-ExecutionPolicy RemoteSigned
        echo 3. Restart PowerShell
        echo 4. Or use npm.cmd instead of npm
    )
)

echo.
echo ========================================
echo Fix Complete!
echo ========================================
echo.

echo If the problem persists, please try:
echo 1. Run PowerShell as Administrator
echo 2. Run: Set-ExecutionPolicy RemoteSigned
echo 3. Or use npm.cmd instead of npm
echo 4. Or use traditional cmd instead of PowerShell
echo.

pause