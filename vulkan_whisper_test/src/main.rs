use anyhow::Result;
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::time::Instant;

fn main() -> Result<()> {
    println!("🚀 Vulkan Whisper推理测试程序");

    // 检查Vulkan是否可用
    println!("🔍 检查Vulkan支持...");
    check_vulkan_support();

    // 模型路径
    let model_path = "/home/martin/.local/share/com.martin.flash-input/models/ggml-small.bin";

    println!("📁 加载模型: {}", model_path);

    // 加载模型（启用Vulkan）
    let ctx = load_whisper_model_with_vulkan(model_path)?;

    // 测试音频文件
    let audio_file = "/home/martin/hello-tauri/money.wav";

    println!("🎵 加载音频文件: {}", audio_file);
    let audio_data = load_wav_file(audio_file)?;
    println!("📊 音频数据: {} 采样点", audio_data.len());

    // 执行推理
    println!("🔥 开始Vulkan推理测试...");
    let start_time = Instant::now();

    let transcription = run_whisper_inference(&ctx, &audio_data)?;

    let duration = start_time.elapsed();
    println!("⏱️  推理完成，耗时: {:?}", duration);
    println!("📝 转录结果: {}", transcription.trim());

    Ok(())
}

fn check_vulkan_support() {
    // 检查Vulkan驱动
    match std::process::Command::new("vulkaninfo").output() {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("Vulkan Instance Version:") {
                    println!("✅ Vulkan运行时支持检测到");
                    // 提取版本信息
                    for line in output_str.lines() {
                        if line.contains("Vulkan Instance Version:") {
                            println!("   {}", line.trim());
                            break;
                        }
                    }
                } else {
                    println!("❌ Vulkan运行时未正确配置");
                }
            } else {
                println!("❌ Vulkan信息查询失败");
            }
        }
        Err(_) => {
            println!("❌ 未找到vulkaninfo命令，请安装vulkan-tools");
        }
    }
}

fn load_whisper_model_with_vulkan(model_path: &str) -> Result<WhisperContext> {
    println!("🔧 初始化Whisper上下文（尝试启用Vulkan）...");

    // 使用默认参数，whisper-rs的vulkan feature会自动启用GPU
    let mut params = WhisperContextParameters::default();

    // 尝试设置GPU相关参数（如果whisper-rs支持）
    // 注意：某些whisper-rs版本可能不支持这些方法
    println!("📋 WhisperContextParameters: {:?}", params);

    let ctx = WhisperContext::new_with_params(model_path, params)
        .map_err(|e| anyhow::anyhow!("加载模型失败: {}", e))?;

    println!("✅ 模型加载成功");

    // 检查是否实际使用了GPU
    check_gpu_usage();

    Ok(ctx)
}

fn check_gpu_usage() {
    println!("🔍 检查GPU使用状态...");

    // 这里我们需要检查whisper是否实际使用了GPU
    // 由于whisper-rs可能不直接暴露这个信息，我们依赖运行时日志

    println!("💡 如果Vulkan正常启用，您应该看到类似以下日志:");
    println!("   - whisper_init_with_params_no_state: use gpu = 1");
    println!("   - whisper_backend_init_gpu: GPU found");
    println!("   - GPU相关的内存分配信息");
}

fn load_wav_file(file_path: &str) -> Result<Vec<f32>> {
    println!("🎵 解析WAV文件...");

    let reader = hound::WavReader::open(file_path)
        .map_err(|e| anyhow::anyhow!("无法打开WAV文件: {}", e))?;

    let spec = reader.spec();
    println!("📊 音频格式: {}Hz, {}通道, {}位",
             spec.sample_rate,
             spec.channels,
             spec.bits_per_sample);

    // 转换为32位浮点数，单声道
    let samples: Vec<f32> = reader.into_samples::<i16>()
        .filter_map(|s| s.ok())
        .map(|s| s as f32 / 32768.0)  // 归一化到[-1.0, 1.0]
        .collect();

    // 如果是立体声，转换为单声道
    let mono_samples = if spec.channels == 2 {
        println!("🔄 转换立体声到单声道...");
        samples.chunks_exact(2)
            .map(|pair| (pair[0] + pair[1]) / 2.0)
            .collect()
    } else {
        samples
    };

    println!("✅ 音频数据加载完成: {} 采样点", mono_samples.len());

    Ok(mono_samples)
}

fn run_whisper_inference(ctx: &WhisperContext, audio_data: &[f32]) -> Result<String> {
    println!("🧠 执行Whisper推理...");

    // 创建推理参数
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    // 配置参数
    params.set_language(None);  // 自动检测语言
    params.set_translate(false);  // 不翻译，直接转录
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // 创建状态
    let mut state = ctx.create_state()
        .map_err(|e| anyhow::anyhow!("创建Whisper状态失败: {}", e))?;

    println!("🔥 开始处理音频数据...");

    // 执行推理
    state.full(params, audio_data)
        .map_err(|e| anyhow::anyhow!("Whisper推理失败: {}", e))?;

    println!("✅ 推理完成，提取结果...");

    // 获取分段数量
    let num_segments = state.full_n_segments()
        .map_err(|e| anyhow::anyhow!("获取分段数量失败: {}", e))?;

    println!("📊 检测到 {} 个语音段", num_segments);

    // 提取转录文本
    let mut result = String::new();
    for i in 0..num_segments {
        let segment_text = state.full_get_segment_text(i)
            .map_err(|e| anyhow::anyhow!("获取分段{}文本失败: {}", i, e))?;

        let start_timestamp = state.full_get_segment_t0(i)
            .map_err(|e| anyhow::anyhow!("获取分段{}开始时间失败: {}", i, e))?;

        let end_timestamp = state.full_get_segment_t1(i)
            .map_err(|e| anyhow::anyhow!("获取分段{}结束时间失败: {}", i, e))?;

        println!("   [{}-{}s] {}", start_timestamp, end_timestamp, segment_text.trim());
        result.push_str(segment_text);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulkan_detection() {
        check_vulkan_support();
    }
}