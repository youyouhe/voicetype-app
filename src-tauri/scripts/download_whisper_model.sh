#!/bin/bash

# Whisper 模型下载脚本
# 用于 EchoType 项目的本地 ASR 功能

set -e

MODEL_DIR="./models"
BASE_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main"

# 可用的模型列表
declare -A MODELS=(
    ["tiny"]="ggml-tiny.bin"
    ["base"]="ggml-base.bin"
    ["small"]="ggml-small.bin"
    ["medium"]="ggml-medium.bin"
    ["large-v3"]="ggml-large-v3.bin"
)

# 模型大小信息（MB）
declare -A MODEL_SIZES=(
    ["tiny"]="39"
    ["base"]="142"
    ["small"]="466"
    ["medium"]="1.5GB"
    ["large-v3"]="2.9GB"
)

# 创建模型目录
mkdir -p "$MODEL_DIR"

echo "🎤 Whisper 模型下载脚本"
echo "========================="
echo ""

# 显示可用模型
echo "可用模型："
for model in "${!MODELS[@]}"; do
    echo "  - $model (${MODEL_SIZES[$model]}): ${MODELS[$model]}"
done
echo ""

# 检查参数
if [ $# -eq 0 ]; then
    echo "用法: $0 <模型名称>"
    echo "示例: $0 base"
    echo ""
    echo "推荐模型："
    echo "  - tiny   : 最快，但准确性较低 (39MB)"
    echo "  - base   : 平衡速度和准确性 (142MB) - 推荐"
    echo "  - small  : 更好的准确性 (466MB)"
    echo "  - medium : 高准确性 (1.5GB)"
    echo "  - large-v3 : 最高准确性 (2.9GB)"
    exit 1
fi

MODEL_NAME=$1
MODEL_FILE="${MODELS[$MODEL_NAME]}"

# 检查模型是否存在
if [ -z "$MODEL_FILE" ]; then
    echo "❌ 错误: 未知模型 '$MODEL_NAME'"
    echo "可用模型: ${!MODELS[*]}"
    exit 1
fi

MODEL_PATH="$MODEL_DIR/$MODEL_FILE"

# 检查文件是否已存在
if [ -f "$MODEL_PATH" ]; then
    echo "⚠️  模型文件已存在: $MODEL_PATH"
    read -p "是否重新下载? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "✅ 使用现有模型文件"
        echo "📍 模型路径: $MODEL_PATH"
        echo ""
        echo "设置环境变量:"
        echo "export WHISPER_MODEL_PATH=\"$MODEL_PATH\""
        exit 0
    fi
    rm "$MODEL_PATH"
fi

echo "📥 下载模型: $MODEL_NAME (${MODEL_SIZES[$MODEL_NAME]})"
echo "🌐 下载地址: $BASE_URL/$MODEL_FILE"
echo "💾 保存路径: $MODEL_PATH"
echo ""

# 使用 curl 下载模型
if command -v curl &> /dev/null; then
    curl -L --progress-bar "$BASE_URL/$MODEL_FILE" -o "$MODEL_PATH"
elif command -v wget &> /dev/null; then
    wget --progress=bar:force "$BASE_URL/$MODEL_FILE" -O "$MODEL_PATH"
else
    echo "❌ 错误: 需要 curl 或 wget 来下载模型"
    exit 1
fi

# 检查下载是否成功
if [ $? -eq 0 ] && [ -f "$MODEL_PATH" ]; then
    echo ""
    echo "✅ 模型下载成功!"
    echo "📍 模型路径: $MODEL_PATH"
    echo "📏 文件大小: $(du -h "$MODEL_PATH" | cut -f1)"
    echo ""
    echo "🔧 设置环境变量:"
    echo "export WHISPER_MODEL_PATH=\"$MODEL_PATH\""
    echo ""
    echo "💡 提示: 将上述环境变量添加到你的 shell 配置文件中 (~/.bashrc, ~/.zshrc 等)"
else
    echo "❌ 模型下载失败!"
    rm -f "$MODEL_PATH"  # 删除可能损坏的文件
    exit 1
fi