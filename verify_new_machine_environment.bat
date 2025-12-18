@echo off
echo ========================================
echo 新机器环境验证工具
echo ========================================
echo.

echo [1] 检查 PowerShell 执行策略...
powershell -Command "Get-ExecutionPolicy"
echo.

echo [2] 检查 Node.js 和 npm...
node --version 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ Node.js: 已安装
    node --version
) else (
    echo ❌ Node.js: 未安装
    echo 💡 请从 https://nodejs.org 下载安装
)
echo.

powershell -Command "npm -v" 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ npm: 已安装
    powershell -Command "npm -v"
) else (
    echo ❌ npm: 未安装或无法执行
    echo 💡 尝试使用 npm.cmd 而不是 npm
)
echo.

echo [3] 检查 Git...
git --version 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ Git: 已安装
    git --version
) else (
    echo ❌ Git: 未安装
    echo 💡 请从 https://git-scm.com/download/win 下载安装
)
echo.

echo [4] 检查 Rust...
rustc --version 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ Rust: 已安装
    rustc --version
) else (
    echo ❌ Rust: 未安装
    echo 💡 访问 https://rustup.rs/ 安装
)
echo.

echo [5] 检查项目依赖...
if exist "package.json" (
    echo ✅ 找到 package.json
    echo 检查 node_modules...
    if exist "node_modules" (
        echo ✅ node_modules: 已存在
    ) else (
        echo ⚠️ node_modules: 不存在，需要运行 npm install
        echo 💡 运行: npm install
    )
) else (
    echo ❌ 未找到 package.json
)
echo.

echo [6] 检查 Tauri 依赖...
cd src-tauri 2>nul
if exist "Cargo.toml" (
    echo ✅ 找到 Cargo.toml
    cargo --version 2>nul
    if %ERRORLEVEL% EQU 0 (
        echo ✅ Cargo: 已安装
    ) else (
        echo ❌ Cargo: 未安装
        echo 💡 Rust 安装后应该包含 Cargo
    )
) else (
    echo ❌ 未找到 Cargo.toml
)
cd ..
echo.

echo [7] 快速功能测试...
echo 测试 npm 命令...
powershell -Command "npm --version" >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo ✅ npm 命令正常工作
) else (
    echo ❌ npm 命令有问题
    echo 💡 尝试使用 npm.cmd 而不是 npm
)
echo.

echo ========================================
echo 环境验证完成！
echo ========================================
echo.

echo 🚀 快速开始指南:
echo 1. 如果所有检查都通过，可以运行:
echo    npm install
echo    npm run tauri dev
echo.
echo 2. 如果缺少组件，请按照上述提示安装
echo.
echo 3. 如果 npm 仍然有问题，使用 npm.cmd 代替 npm
echo.

pause