// 在浏览器控制台中运行此脚本来检查配置存储情况
console.log("=== 配置调试信息 ===");

// 检查 localStorage 中的配置
const localStorageConfig = localStorage.getItem('asr_config');
if (localStorageConfig) {
  const parsed = JSON.parse(localStorageConfig);
  console.log("📦 localStorage 中的 ASR 配置:");
  console.log({
    service_provider: parsed.service_provider,
    local_endpoint: parsed.local_endpoint,
    local_api_key: parsed.local_api_key ? parsed.local_api_key.substring(0, 20) + '...' : 'undefined',
    cloud_endpoint: parsed.cloud_endpoint,
    cloud_api_key: parsed.cloud_api_key ? parsed.cloud_api_key.substring(0, 10) + '...' : 'undefined',
    updated_at: parsed.updated_at
  });
} else {
  console.log("❌ localStorage 中没有 ASR 配置");
}

// 检查环境
if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
  console.log("🚀 当前运行在 Tauri 环境");
} else {
  console.log("🌐 当前运行在浏览器环境");
  console.log("💡 提示: 配置只保存在 localStorage 中，没有保存到 SQLite 数据库");
  console.log("💡 要保存到 SQLite 数据库，请运行 'npm run tauri dev'");
}

console.log("=== 调试结束 ===");