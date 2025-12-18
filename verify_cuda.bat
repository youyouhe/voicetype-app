@echo off
echo ========================================
echo CUDA 环境验证工具
echo ========================================
echo.

echo [1] 检查 NVIDIA 显卡驱动...
nvidia-smi
if %ERRORLEVEL% EQU 0 (
    echo ✅ NVIDIA 驱动正常
) else (
    echo ❌ NVIDIA 驱动未安装或有问题
    echo 💡 请访问 https://www.nvidia.com/drivers/ 安装驱动
)
echo.

echo [2] 检查 CUDA Toolkit...
nvcc --version 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ CUDA Toolkit 已安装
    nvcc --version
) else (
    echo ❌ CUDA Toolkit 未安装或不在 PATH 中
    echo 💡 请访问 https://developer.nvidia.com/cuda-downloads 安装 CUDA Toolkit
)
echo.

echo [3] 检查 Visual Studio 编译器...
cl 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ Visual Studio 编译器可用
) else (
    echo ❌ Visual Studio 编译器未找到
    echo 💡 请安装 Visual Studio Community 2022 或 Build Tools
)
echo.

echo [4] 检查 CUDA 运行时库...
where cudart64_*.dll >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo ✅ CUDA 运行时库已找到
    where cudart64_*.dll
) else (
    echo ❌ CUDA 运行时库未找到
    echo 💡 CUDA 运行时库通常位于：
    echo    - C:\Windows\System32\
    echo    - C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\vx.x\bin\
)
echo.

echo [5] 检查 GPU 内存...
nvidia-smi --query-gpu=name,memory.total --format=csv,noheader,nounits 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ GPU 信息获取成功
    for /f "tokens=2" %%a in ('nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits') do (
        set gpu_memory=%%a
    )
    echo 显存大小: %gpu_memory% MB
    if %gpu_memory% GEQ 4096 (
        echo ✅ 显存充足，适合 GPU 加速
    ) else (
        echo ⚠️ 显存较小，建议使用更小的模型
    )
)
echo.

echo [6] 检查环境变量...
echo CUDA_PATH: %CUDA_PATH%
echo PATH 包含 CUDA: %PATH%
echo.

echo ========================================
echo 验证完成！
echo ========================================
pause