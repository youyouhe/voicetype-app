# Windows CUDA 安装要求清单

## 必需软件下载链接

### 1. NVIDIA 显卡驱动
- **下载地址**: https://www.nvidia.com/drivers/
- **版本要求**: 470.x 或更高
- **验证命令**: `nvidia-smi`

### 2. Visual Studio Build Tools
- **下载地址**: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
- **必需组件**:
  - ✅ C++ build tools
  - ✅ Windows 10/11 SDK
  - ✅ CMake tools
- **替代方案**: Visual Studio Community 2022 (完整版)

### 3. CUDA Toolkit
- **下载地址**: https://developer.nvidia.com/cuda-downloads
- **推荐版本**:
  - CUDA 12.0+ (最新)
  - CUDA 11.8 (稳定)
- **验证命令**: `nvcc --version`

## 可选但推荐

### 4. Git
- **下载地址**: https://git-scm.com/download/win

### 5. Rust (如果要编译)
- **下载地址**: https://rustup.rs/

## 一键安装脚本

保存为 `install_cuda.bat` 并以管理员身份运行：

```batch
@echo off
echo 开始 CUDA 环境安装...
echo.

echo 1. 检查管理员权限...
net session >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ 请以管理员身份运行此脚本
    pause
    exit /b 1
)

echo 2. 下载 NVIDIA 驱动...
echo 请访问 https://www.nvidia.com/drivers/ 手动下载并安装
pause

echo 3. 下载 CUDA Toolkit...
echo 请访问 https://developer.nvidia.com/cuda-downloads 手动下载并安装
pause

echo 4. 安装完成后，运行验证脚本...
echo 运行: verify_cuda.bat
pause
```

## 安装后验证

1. **运行验证脚本**:
   ```cmd
   verify_cuda.bat
   ```

2. **手动验证关键组件**:
   ```cmd
   # 检查 NVIDIA 驱动
   nvidia-smi

   # 检查 CUDA 编译器
   nvcc --version

   # 检查编译器
   cl
   ```

## 常见问题解决

### 问题1: "cl 不是内部或外部命令"
**解决方案**: 安装 Visual Studio Build Tools，确保包含 C++ 工具

### 问题2: "nvidia-smi 不是内部或外部命令"
**解决方案**: 安装 NVIDIA 显卡驱动

### 问题3: "nvcc 不是内部或外部命令"
**解决方案**: 安装 CUDA Toolkit 并添加到系统 PATH

### 问题4: CUDA 编译失败
**解决方案**:
1. 确认 Visual Studio 版本兼容
2. 检查 CUDA Toolkit 版本
3. 以管理员身份运行编译命令

## 环境变量设置

如果自动设置失败，手动添加：

```cmd
# CUDA 安装路径 (根据实际安装位置调整)
set CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.0

# 添加到 PATH
set PATH=%CUDA_PATH%\bin;%PATH%
set PATH=%CUDA_PATH%\libnvvp;%PATH%
```

## 性能优化设置

```cmd
# 设置 GPU 性能模式
nvidia-smi -pm 1

# 查看当前 GPU 状态
nvidia-smi
```

---

*安装完成后，您就可以编译和运行支持 CUDA 的 EchoType 了！*