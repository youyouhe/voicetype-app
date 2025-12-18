@echo off
echo Testing npm command...
echo.

echo Using npm.cmd instead of npm:
npm.cmd -v

echo.
echo If npm.cmd works, you can use it in PowerShell too:
echo Just type "npm.cmd install" instead of "npm install"

echo.
echo Testing npm with cmd wrapper:
cmd /c "npm -v"

echo.
echo If both methods work, your npm is properly configured!
pause