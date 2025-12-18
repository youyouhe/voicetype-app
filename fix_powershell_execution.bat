@echo off
echo ========================================
echo PowerShell 执行策略修复工具
echo ========================================
echo.

echo 正在修复 PowerShell 执行策略问题...
echo.

echo 方法1: 为当前用户设置执行策略...
powershell -Command "Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force"

echo.
echo 验证执行策略设置...
powershell -Command "Get-ExecutionPolicy"

echo.
echo 方法2: 测试 npm 命令...
echo 尝试执行 npm -v...
powershell -Command "npm -v"

if %ERRORLEVEL% EQU 0 (
    echo ✅ npm 命令执行成功！
) else (
    echo ❌ npm 命令仍然失败，尝试方法3...
    echo.
    echo 尝试使用 npm.cmd 而不是 npm...
    npm.cmd -v

    if %ERRORLEVEL% EQU 0 (
        echo ✅ npm.cmd 执行成功！
        echo 💡 建议使用 npm.cmd 而不是 npm
    ) else (
        echo ❌ npm.cmd 也失败了
        echo.
        echo 手动解决方案:
        echo 1. 以管理员身份打开 PowerShell
        echo 2. 运行: Set-ExecutionPolicy RemoteSigned
        echo 3. 重启 PowerShell
    )
)

echo.
echo ========================================
echo 修复完成！
echo ========================================
echo.

echo 如果问题仍然存在，请尝试:
echo 1. 以管理员身份运行 PowerShell
echo 2. 运行: Set-ExecutionPolicy RemoteSigned
echo 3. 或者直接使用 npm.cmd 而不是 npm
echo 4. 或者使用传统的 cmd 而不是 PowerShell
echo.

pause