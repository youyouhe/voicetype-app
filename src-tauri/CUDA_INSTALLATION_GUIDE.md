# CUDA 安装和使用指南

## 概述

EchoType 支持 NVIDIA GPU 加速，可显著提升语音识别速度。本指南将帮助您安装和配置 CUDA。

## 系统要求

### 硬件要求
- **NVIDIA GPU**: 支持 CUDA 的 NVIDIA 显卡
- **显存**: 建议至少 4GB VRAM（8GB+ 为佳）
- **GPU 系列**: GeForce RTX 20系列及以上，或 Quadro RTX 系列及以上

### 软件要求
- **操作系统**: Windows 10/11 (64位)
- **驱动程序**: NVIDIA 显卡驱动 470.x 或更高版本

## 安装步骤

### 1. 安装 NVIDIA 显卡驱动

这是最基础的步骤，即使不安装完整的 CUDA Toolkit 也需要：

1. 访问 [NVIDIA 官网](https://www.nvidia.com/drivers/)
2. 选择您的显卡型号和操作系统
3. 下载并安装最新的 Game Ready 或 Studio 驱动

验证安装：
```cmd
nvidia-smi
```

如果看到 GPU 信息，说明驱动安装成功。

### 2. 安装 CUDA Toolkit（可选但推荐）

CUDA Toolkit 提供完整的 CUDA 开发和运行环境：

1. 访问 [CUDA Toolkit 下载页面](https://developer.nvidia.com/cuda-downloads)
2. 选择您的操作系统和配置
3. 下载并安装 CUDA 12.x 或 11.x 版本

推荐版本：
- **CUDA 12.0+**: 最新功能，更好的性能
- **CUDA 11.8**: 稳定版本，兼容性好

### 3. 验证 CUDA 安装

安装完成后，验证 CUDA 是否正确安装：

```cmd
nvcc --version
```

如果显示 CUDA 编译器版本信息，说明安装成功。

## EchoType 中的 CUDA 使用

### 自动检测

EchoType 会自动检测您的系统中的 CUDA 支持：

1. **NVIDIA 驱动检测**: 检查 `nvidia-smi.exe` 是否存在
2. **CUDA Toolkit 检测**: 检查标准安装路径
3. **运行时库检测**: 检查系统 PATH 中的 CUDA 库
4. **GPU 内存检查**: 确保有足够的显存用于模型加速

### 手动配置

如果自动检测失败，您可以通过环境变量手动指定：

```cmd
set CUDA_VISIBLE_DEVICES=0
set WHISPER_MODEL_PATH=C:\path\to\your\model.bin
```

### 性能优化建议

1. **使用合适的模型**:
   - 小模型（base, small）: 4GB+ VRAM
   - 大模型（medium, large）: 8GB+ VRAM

2. **GPU 设置**:
   - 单 GPU 系统: 使用默认设置
   - 多 GPU 系统: 设置 `CUDA_VISIBLE_DEVICES` 指定使用的 GPU

3. **内存管理**:
   - 关闭其他 GPU 密集型应用
   - 监控 GPU 内存使用情况

## 故障排除

### 常见问题

#### 1. "NVIDIA driver not found"
**解决方案**:
```cmd
# 重新安装显卡驱动
# 访问 https://www.nvidia.com/drivers/
```

#### 2. "CUDA runtime libraries missing"
**解决方案**:
```cmd
# 检查 CUDA 安装
where cudart64_120.dll

# 如果找不到，重新安装 CUDA Toolkit
# 或将 CUDA bin 目录添加到 PATH
```

#### 3. "Insufficient GPU memory"
**解决方案**:
- 使用较小的模型文件
- 关闭其他 GPU 应用程序
- 重启系统释放 GPU 内存

#### 4. "GPU backend initialization failed"
**解决方案**:
```cmd
# 检查 GPU 是否支持 CUDA
nvidia-smi

# 更新显卡驱动
# 检查 CUDA 版本兼容性
```

### 日志分析

EchoType 启动时会显示详细的 GPU 检测信息：

```
🔍 Starting comprehensive GPU backend detection...
   📋 Checking CUDA support (NVIDIA GPUs)...
🚀 NVIDIA driver detected
💾 NVIDIA GPU Info:
NVIDIA GeForce RTX 3080, 5120 MiB, Driver Version: 531.68
✅ Sufficient GPU memory detected for CUDA acceleration
🎯 CUDA installation found at: C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.0
✅ CUDA runtime libraries found
✅ CUDA backend detected - Highest performance option
```

## 性能对比

使用 CUDA 可以显著提升性能：

| 模型 | CPU 处理时间 | GPU 处理时间 | 加速比 |
|------|-------------|-------------|--------|
| base | ~2-3 秒 | ~0.2-0.5 秒 | 5-10x |
| small | ~5-8 秒 | ~0.3-0.7 秒 | 8-15x |
| medium | ~10-15 秒 | ~0.5-1.0 秒 | 15-20x |
| large | ~20-30 秒 | ~1.0-2.0 秒 | 15-30x |

*实际性能取决于您的具体硬件配置*

## 技术支持

如果遇到问题：

1. 检查 NVIDIA 官方文档
2. 确认硬件和软件兼容性
3. 查看应用程序日志
4. 访问 EchoType 社区寻求帮助

---

*注意：CUDA 是 NVIDIA 的专有技术，仅支持 NVIDIA 显卡。对于 AMD 显卡，应用程序会自动使用 Vulkan 或 CPU 后端。*