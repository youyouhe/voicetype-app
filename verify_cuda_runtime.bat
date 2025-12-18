@echo off
echo ========================================
echo CUDA 运行时环境验证工具
echo ========================================
echo.

echo [1] 检查 NVIDIA 显卡驱动...
nvidia-smi 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ NVIDIA 驱动正常

    echo.
    echo 💾 GPU 详细信息:
    nvidia-smi --query-gpu=name,driver_version,cuda_version,memory.total --format=csv

    echo.
    echo 🚀 CUDA 兼容性检查:
    for /f "tokens=4" %%a in ('nvidia-smi --query-gpu=cuda_version --format=csv,noheader,nounits') do (
        set driver_cuda=%%a
    )
    echo 驱动支持的CUDA版本: %driver_cuda%

    if %driver_cuda% GEQ 12.0 (
        echo ✅ 支持 CUDA 12.x
    ) else if %driver_cuda% GEQ 11.8 (
        echo ✅ 支持 CUDA 11.8
    ) else if %driver_cuda% GEQ 11.0 (
        echo ⚠️ 支持 CUDA 11.x，建议升级驱动
    ) else (
        echo ❌ CUDA 版本过低，需要升级驱动
    )
) else (
    echo ❌ NVIDIA 驱动未安装
    echo 💡 这是 CUDA 运行的绝对要求
    echo 💡 请访问 https://www.nvidia.com/drivers/ 安装驱动
)
echo.

echo [2] 检查 CUDA 运行时库...
echo 查找 cudart64_*.dll...
where cudart64_*.dll >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo ✅ CUDA 运行时库已找到
    where cudart64_*.dll
) else (
    echo ❌ CUDA 运行时库未找到
    echo 💡 解决方案:
    echo   1. 安装 CUDA Toolkit
    echo   2. 或复制必要的DLL到应用程序目录
    echo   3. 或将CUDA bin目录添加到PATH
)

echo.
echo 查找 cublas64_*.dll...
where cublas64_*.dll >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo ✅ CUDA BLAS库已找到
) else (
    echo ⚠️ CUDA BLAS库未找到，可能影响性能
)
echo.

echo [3] 检查应用程序目录...
echo 当前目录: %CD%
if exist "hello-tauri.exe" (
    echo ✅ 找到应用程序
) else (
    echo ❌ 未找到应用程序
    echo 💡 请确保在正确的目录中运行此脚本
)
echo.

echo [4] 检查 Visual C++ 运行时...
reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo ✅ Visual C++ 2015+ 运行时已安装
) else (
    echo ⚠️ Visual C++ 运行时可能缺失
    echo 💡 大多数系统已包含，如有问题请安装 VC++ Redistributable
)
echo.

echo [5] 测试 CUDA 应用启动...
echo 尝试启动应用程序...
timeout /t 2 /nobreak >nul
start /B hello-tauri.exe >nul 2>&1
timeout /t 3 /nobreak >nul
tasklist /FI "IMAGENAME eq hello-tauri.exe" 2>NUL | find /I "hello-tauri.exe" >NUL
if %ERRORLEVEL% EQU 0 (
    echo ✅ 应用程序启动成功
    taskkill /F /IM hello-tauri.exe >nul 2>&1
) else (
    echo ❌ 应用程序启动失败
    echo 💡 检查日志文件或错误信息
)
echo.

echo ========================================
echo 运行时验证完成！
echo ========================================
echo.
echo 📋 运行要求总结:
echo   - NVIDIA 驱动 (必需)
echo   - CUDA 运行时库 (必需)
echo   - Visual C++ 运行时 (推荐)
echo.
echo 🚀 如果缺少组件，可以:
echo   1. 安装完整的 CUDA Toolkit
echo   2. 或只安装必要的运行时库
echo   3. 或使用便携式 CUDA 版本
echo.
pause