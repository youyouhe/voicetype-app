# CUDA 构建指南

## 概述

本指南将帮助您构建支持 CUDA GPU 加速的 EchoType 应用程序。当前发布的版本使用 CPU 后端，要启用 CUDA 需要重新编译。

## 系统要求

### 硬件要求
- **NVIDIA GPU**: 支持 CUDA 的显卡
- **显存**: 至少 4GB（推荐 8GB+）
- **支持系列**:
  - GeForce RTX 20系列及以上
  - Quadro RTX 系列及以上
  - Tesla V100/A100/H100

### 软件要求
- **Windows 10/11** (64位)
- **NVIDIA 驱动**: 470.x 或更高版本
- **CUDA Toolkit**: 11.8 或 12.x（可选）
- **Visual Studio 2019/2022**（包含 C++ 构建工具）
- **Git**

## 安装步骤

### 1. 安装 NVIDIA 显卡驱动

```cmd
# 访问 https://www.nvidia.com/drivers/
# 下载并安装适合您显卡的最新驱动
```

验证安装：
```cmd
nvidia-smi
```

### 2. 安装 CUDA Toolkit（可选但推荐）

```cmd
# 访问 https://developer.nvidia.com/cuda-downloads
# 下载并安装 CUDA 11.8 或 12.x
```

### 3. 安装构建工具

确保已安装 Rust 和必要的构建工具：

```cmd
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Visual Studio Build Tools
# 或安装完整的 Visual Studio Community
```

### 4. 构建支持 CUDA 的版本

#### 方法一：仅 CUDA 支持

```cmd
cd src-tauri
cargo build --release --features cuda
```

#### 方法二：完整的 GPU 支持

```cmd
cd src-tauri
cargo build --release --features gpu
```

#### 方法三：手动配置特性

编辑 `Cargo.toml` 文件：

```toml
[dependencies]
whisper-rs = { version = "0.13", features = ["cuda"] }
```

然后构建：
```cmd
cd src-tauri
cargo build --release
```

### 5. 构建完整应用

```cmd
# 返回项目根目录
cd ..

# 构建前端
cd src
npm run build
cd ..

# 构建 Tauri 应用
npm run tauri build
```

## 验证 CUDA 支持

构建完成后，运行应用程序并查看启动日志：

```
🔍 Starting comprehensive GPU backend detection...
   📋 Checking CUDA support (NVIDIA GPUs)...
🚀 NVIDIA driver detected
💾 NVIDIA GPU Info:
NVIDIA GeForce RTX 3080, 10240 MiB, Driver Version: 531.68
✅ Sufficient GPU memory detected for CUDA acceleration
🎯 CUDA installation found at: C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.0
✅ CUDA runtime libraries found
✅ CUDA backend detected - Highest performance option
```

如果看到类似输出，说明 CUDA 支持已成功启用。

## 故障排除

### 常见编译错误

#### 1. "CUDA toolkit not found"

**解决方案**：
- 安装 CUDA Toolkit
- 确保 CUDA 安装路径在系统 PATH 中
- 重启命令行或系统

#### 2. "NVIDIA driver too old"

**解决方案**：
```cmd
# 更新到最新的 NVIDIA 驱动
# 访问 https://www.nvidia.com/drivers/
```

#### 3. "CMake configuration failed"

**解决方案**：
- 安装 Visual Studio Build Tools
- 确保 CMake 已安装
- 检查 Visual Studio 组件是否完整

#### 4. "Link error with CUDA libraries"

**解决方案**：
```cmd
# 检查 CUDA 环境变量
echo %CUDA_PATH%
echo %PATH%

# 如果未设置，手动添加：
set CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.0
set PATH=%CUDA_PATH%\bin;%PATH%
```

### 运行时问题

#### 1. "CUDA initialization failed"

**可能原因**：
- GPU 不支持 CUDA
- 显存不足
- 驱动程序问题

**解决方案**：
```cmd
# 检查 GPU 状态
nvidia-smi

# 尝试使用更小的模型
# 关闭其他 GPU 应用
```

#### 2. "Fallback to CPU"

**解决方案**：
- 确认 CUDA 驱动正确安装
- 检查 CUDA Toolkit 版本兼容性
- 查看应用程序日志了解具体原因

## 性能优化

### 1. 模型选择

| 模型 | 显存要求 | 推荐用途 |
|------|----------|----------|
| tiny | 1GB | 快速测试，低精度 |
| base | 2GB | 日常使用，平衡速度和精度 |
| small | 4GB | 较高精度要求 |
| medium | 8GB | 高精度应用 |
| large | 12GB | 专业应用 |

### 2. 系统优化

```cmd
# 设置 GPU 性能模式
nvidia-smi -pm 1

# 设置最大功耗限制（如果需要）
nvidia-smi -pl 250
```

### 3. 环境变量优化

```cmd
# 设置 CUDA 设备
set CUDA_VISIBLE_DEVICES=0

# 优化 GPU 内存使用
set CUDA_LAUNCH_BLOCKING=1
```

## 构建变体

### CPU Only 版本（默认）

```cmd
cargo build --release
```

### 仅 CUDA 版本

```cmd
cargo build --release --features cuda
```

### 多 GPU 后端版本

```cmd
cargo build --release --features "cuda,vulkan,metal"
```

## 技术细节

### 特性配置

```toml
[features]
default = []
cuda = ["whisper-rs/cuda"]
vulkan = ["whisper-rs/vulkan"]
metal = ["whisper-rs/metal"]
gpu = ["cuda", "vulkan"]
```

### 环境变量

| 变量 | 说明 | 示例 |
|------|------|------|
| `CUDA_VISIBLE_DEVICES` | 指定使用的 GPU | `0` |
| `CUDA_PATH` | CUDA 安装路径 | `C:\CUDA\v12.0` |
| `WHISPER_MODEL_PATH` | 模型文件路径 | `./models/ggml-base.bin` |

## 发布版本

要构建包含 CUDA 支持的发布版本：

1. 按上述步骤安装依赖
2. 使用 CUDA 特性构建
3. 创建安装包：
   ```cmd
   npm run tauri build
   ```
4. 在 `src-tauri/target/release/` 目录找到生成的可执行文件

## 支持

如果遇到问题：

1. 检查 NVIDIA 官方文档
2. 查看 CUDA 安装指南
3. 检查 whisper-rs 项目问题
4. 联系 EchoType 技术支持

---

*注意：CUDA 功能需要重新编译，预编译版本仅支持 CPU 后端。*