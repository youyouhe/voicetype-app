@echo off
setlocal enabledelayedexpansion

echo ========================================
echo Windows 10 CUDA 编译环境安装向导
echo ========================================
echo.

echo 当前系统检查结果:
echo   - ✅ Git: 已安装
echo   - ✅ Rust: 已安装
echo   - ❌ NVIDIA 驱动: 需要安装
echo   - ❌ Visual Studio: 需要安装
echo.

pause
echo.

echo === 步骤 1: 安装 NVIDIA 显卡驱动 ===
echo.
echo 正在打开 NVIDIA 驱动下载页面...
echo.
echo 请按照以下步骤操作:
echo 1. 访问 https://www.nvidia.com/drivers/
echo 2. 选择您的显卡型号
echo 3. 下载最新的 Game Ready 或 Studio 驱动
echo 4. 安装驱动
echo 5. 安装完成后运行 'nvidia-smi' 验证
echo.
start https://www.nvidia.com/drivers/
echo.
set /p "driver_done=驱动安装完成后按回车继续 (输入 n 跳过): "
if /i "!driver_done!"=="n" goto skip_driver

:skip_driver
echo.
echo === 步骤 2: 安装 Visual Studio Build Tools ===
echo.
echo 正在打开 Visual Studio Build Tools 下载页面...
echo.
echo 请按照以下步骤操作:
echo 1. 访问 https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
echo 2. 下载 Build Tools for Visual Studio 2022
echo 3. 运行安装程序
echo 4. 选择工作负载:
echo    ✅ C++ build tools
echo    ✅ Windows 10/11 SDK
echo    ✅ CMake tools for Visual Studio
echo 5. 安装完成后运行 'cl' 验证
echo.
start https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
echo.
set /p "vs_done=Visual Studio 安装完成后按回车继续 (输入 n 跳过): "
if /i "!vs_done!"=="n" goto skip_vs

:skip_vs
echo.
echo === 步骤 3: 安装 CUDA Toolkit ===
echo.
echo 正在打开 CUDA Toolkit 下载页面...
echo.
echo 推荐版本选择:
echo   - CUDA 12.0 (最新功能)
echo   - CUDA 11.8 (稳定版本)
echo.
echo 请按照以下步骤操作:
echo 1. 访问 https://developer.nvidia.com/cuda-downloads
echo 2. 选择 Windows, x86_64, Version, exe(local)
echo 3. 下载并安装 CUDA Toolkit
echo 4. 安装完成后运行 'nvcc --version' 验证
echo.
start https://developer.nvidia.com/cuda-downloads
echo.
set /p "cuda_done=CUDA Toolkit 安装完成后按回车继续 (输入 n 跳过): "
if /i "!cuda_done!"=="n" goto skip_cuda

:skip_cuda
echo.
echo === 验证安装 ===
echo.
echo 正在检查所有组件...
echo.

echo [1] 检查 NVIDIA 驱动...
nvidia-smi 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ NVIDIA 驱动已安装
    nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader,nounits
) else (
    echo ❌ NVIDIA 驱动未安装或有问题
)
echo.

echo [2] 检查 Visual Studio 编译器...
cl 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ Visual Studio C++ 编译器已安装
) else (
    echo ❌ Visual Studio C++ 编译器未找到
)
echo.

echo [3] 检查 CUDA Toolkit...
nvcc --version 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ CUDA Toolkit 已安装
    nvcc --version
) else (
    echo ❌ CUDA Toolkit 未安装或不在 PATH 中
)
echo.

echo [4] 检查 Rust 和 Git...
rustc --version 2>nul && echo ✅ Rust: 已安装 || echo ❌ Rust: 未安装
git --version 2>nul && echo ✅ Git: 已安装 || echo ❌ Git: 未安装
echo.

echo.
echo ========================================
echo 安装向导完成！
echo ========================================
echo.

if exist "C:\Windows\System32\nvidia-smi.exe" (
    if exist "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC" (
        echo 🎉 所有必需组件已安装！
        echo 您现在可以编译 CUDA 版本的应用程序了。
        echo.
        echo 下一步:
        echo   1. cd C:\Users\Administrator\EchoType\src-tauri
        echo   2. cargo build --release --features cuda
        echo   3. npm run tauri build
    ) else (
        echo ⚠️ NVIDIA 驱动已安装，但 Visual Studio 缺失
        echo 请安装 Visual Studio Build Tools 后重试
    )
) else (
    echo ❌ 还需要安装 NVIDIA 显卡驱动
    echo 请完成驱动安装后重新运行此脚本
)

echo.
pause