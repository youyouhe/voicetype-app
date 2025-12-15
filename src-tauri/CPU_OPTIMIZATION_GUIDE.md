# CPU 优化指南

本指南专门针对没有 NVIDIA GPU 的机器，优化 Whisper-rs 的性能。

## 🎯 已完成的优化

### 1. 智能线程管理
- 根据CPU核心数自动调整线程数量
- 高核心数CPU保留2个核心给系统
- 中等核心数CPU保留1个核心给系统
- 低端机器使用所有可用核心

### 2. 模型自动选择
系统会按以下优先级自动选择已下载的模型（从小到大）：
1. `ggml-base.bin` (~74MB) - 最快，适合实时使用
2. `ggml-small.bin` (~244MB) - 平衡性能和准确性
3. `ggml-medium.bin` (~769MB) - 较高准确性
4. `ggml-large-v3-turbo.bin` (~1570MB) - 最高准确性

### 3. CPU特定参数优化
- 禁用不必要的音频抑制以提升CPU性能
- 优化内存使用模式
- 启用提示缓存以提升后续识别速度

## 💡 使用建议

### 模型选择建议
- **日常使用**: `ggml-base.bin` - 快速响应
- **重要转录**: `ggml-small.bin` - 平衡选择
- **高质量需求**: `ggml-medium.bin` - 更准确但较慢

### 系统要求
- **最低**: 2核CPU, 4GB RAM
- **推荐**: 4核CPU, 8GB RAM
- **最佳**: 8核CPU, 16GB RAM

### 下载模型
```bash
# 创建模型目录
mkdir -p ~/.local/share/com.martin.flash-input/models

# 下载推荐模型（选择一个）
wget -O ~/.local/share/com.martin.flash-input/models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin

# 或下载更准确的模型
wget -O ~/.local/share/com.martin.flash-input/models/ggml-small.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
```

## 🔧 性能调优

### 监控系统资源
```bash
# 监控CPU和内存使用
htop

# 监控温度
sensors
```

### 优化系统设置
1. **关闭不必要的程序**减少内存竞争
2. **设置高性能模式**（如可用）
3. **确保良好的散热**避免热降频

## ⚡ Vulkan 支持计划

对于支持 Vulkan 的现代 GPU（包括 Intel 集成显卡），我们计划添加 Vulkan 加速支持：

- **Intel HD Graphics 5000+**
- **AMD Radeon GCN+**
- **其他支持 Vulkan 的显卡**

这将显著提升在有适当硬件但无 NVIDIA GPU 的机器上的性能。

## 📊 性能基准

预期性能（基于模型）：

| 模型 | 内存使用 | CPU使用 | 实时因子 (RTF) | 推荐场景 |
|------|----------|---------|----------------|----------|
| base | ~200MB | 中等 | 0.3-0.5x | 实时转录 |
| small | ~400MB | 中高 | 0.5-0.8x | 一般转录 |
| medium | ~800MB | 高 | 0.8-1.2x | 高质量转录 |

*RTF < 1.0 表示处理速度比实时音频快*

## 🚨 故障排除

### 处理中断
如果遇到处理中断：
1. 检查系统内存是否足够
2. 尝试使用更小的模型
3. 确保系统没有过热

### 性能慢
1. 检查是否有其他程序占用CPU
2. 考虑使用更小的模型
3. 重启应用清理内存碎片

## 🔮 未来优化计划

1. **Vulkan 加速** - 支持现代GPU加速
2. **模型量化** - 更小更快的量化模型
3. **流式处理** - 减少内存占用
4. **多语言优化** - 针对中文的特定优化