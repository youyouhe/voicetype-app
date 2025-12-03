#!/bin/bash
# Rust Crates 添加脚本 (Tauri Voice Assistant 移植)
# 用法: chmod +x add_crates.sh && ./add_crates.sh
# 注意: 在 /home/martin/hello-tauri 运行。网络问题？用VPN/代理或手动编辑 Cargo.toml。

cd src-tauri || { echo "Error: No src-tauri dir"; exit 1; }

echo "=== 添加 crates (忽略网络失败，手动重试) ==="

cargo add cpal --features=async-api || echo "cpal failed, retry later"
cargo add rdev || echo "rdev failed"
cargo add reqwest --features="json default-tls" || echo "reqwest failed"
cargo add tracing tracing-appender tracing-subscriber || echo "tracing failed"
cargo add hound || echo "hound failed"
cargo add dotenvy || echo "dotenvy failed"
cargo add tokio --features=full || echo "tokio failed"

echo "=== 检查 Cargo.toml ==="
cargo check

echo "=== 完成！手动验证 src-tauri/Cargo.toml ==="
cat Cargo.toml | grep -A 20 "\[dependencies\]"
