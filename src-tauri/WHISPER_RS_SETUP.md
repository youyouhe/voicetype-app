# WhisperRS 集成设置指南

## 概述

本项目已成功集成了 `whisper-rs`，这是一个基于 whisper.cpp 的高性能 Rust 绑定，提供完全本地化的语音识别功能。

## 依赖安装

在编译之前，请确保已安装以下依赖：

### Ubuntu/Debian
```bash
sudo apt update
sudo apt install -y clang cmake build-essential pkg-config
```

### macOS
```bash
brew install cmake
```

### Windows
需要安装 Visual Studio C++ Build Tools 和 CMake。

## 环境配置

### 1. 下载 Whisper 模型

从以下地址下载预训练的 whisper.cpp 模型：

- [官方模型下载](https://huggingface.co/ggerganov/whisper.cpp/tree/main)
- 推荐模型：`ggml-base.bin` 或 `ggml-small.bin`

### 2. 设置模型路径

设置环境变量指向你的模型文件：

```bash
export WHISPER_MODEL_PATH="/path/to/your/ggml-base.bin"
```

或者在 `.env` 文件中添加：
```
WHISPER_MODEL_PATH=./models/ggml-base.bin
```

## 使用方法

### 1. 配置选择

在应用的 ASR 配置中选择 `whisper-rs` 处理器类型：

```json
{
  "asr_processor": "whisper-rs",
  "model_path": "./models/ggml-base.bin"
}
```

### 2. 功能特性

- ✅ **完全本地化**：无需网络连接
- ✅ **高性能**：基于 whisper.cpp 优化
- ✅ **多语言支持**：自动语言检测
- ✅ **实时转录**：支持实时音频处理
- ✅ **隐私保护**：数据不离开本地设备

### 3. 支持的模型

- `ggml-tiny.bin` - 最小，速度最快
- `ggml-base.bin` - 平衡速度和准确性（推荐）
- `ggml-small.bin` - 更好的准确性
- `ggml-medium.bin` - 高准确性
- `ggml-large-v3.bin` - 最高准确性

## 性能优化

### 1. CPU 优化

- 使用多线程：设置 `RAYON_NUM_THREADS` 环境变量
- 内存限制：调整模型的量化级别

### 2. GPU 加速

编译时启用特定 feature：

```toml
[dependencies]
whisper-rs = { version = "0.11", features = ["cuda"] }  # NVIDIA GPU
# whisper-rs = { version = "0.11", features = ["metal"] }  # Apple Silicon
# whisper-rs = { version = "0.11", features = ["vulkan"] }  # Vulkan
```

## 故障排除

### 1. 模型加载失败

```
Error: Whisper model file not found: ./models/ggml-base.bin
```

**解决方案**：确保 `WHISPER_MODEL_PATH` 环境变量设置正确，或模型文件存在于指定路径。

### 2. 编译错误

```
Error: Unable to find libclang
```

**解决方案**：安装 LLVM/Clang 开发工具：

```bash
# Ubuntu/Debian
sudo apt install clang libclang-dev

# macOS
brew install llvm
```

### 3. 性能问题

- 确保使用合适的模型大小
- 检查系统资源使用情况
- 考虑启用 GPU 加速

## 代码示例

```rust
use your_project::voice_assistant::asr::whisper_rs::WhisperRSProcessor;

// 创建处理器
let processor = WhisperRSProcessor::with_model_path("./models/ggml-base.bin")?;

// 处理音频
let result = processor.process_audio(audio_buffer, Mode::Transcriptions, "")?;

println!("转录结果: {}", result);
```

## 集成状态

- ✅ 基本集成完成
- ✅ ASR trait 实现完成
- ✅ 配置系统更新完成
- ✅ 编译通过，无错误和警告
- 🔄 测试和优化进行中