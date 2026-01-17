use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig, SampleFormat, Host};
use hound::{WavWriter, WavSpec};
use std::io::Cursor;
use std::path::PathBuf;
use crate::voice_assistant::VoiceError;

pub struct AudioRecorder {
    recording: bool,
    sample_rate: u32,
    min_duration_secs: f64,
    record_start_time: Option<std::time::Instant>,
    audio_data: Vec<f32>,
    stream: Option<Stream>,
    save_wav_files: bool,
    _host: Host,
    recording_audio_data: Option<std::sync::Arc<std::sync::Mutex<Vec<f32>>>>,
    stream_initialized: bool,  // 🔥 新增：跟踪stream是否已初始化
}

impl AudioRecorder {
    pub fn new() -> Result<Self, VoiceError> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| VoiceError::Audio("No default input device found".to_string()))?;

        // 获取硬件支持的实际配置，不要强制使用特定采样率
        let config = device.default_input_config()
            .map_err(|e| VoiceError::Audio(format!("Failed to get input config: {}", e)))?;

        let sample_rate = config.sample_rate();
        println!("AudioRecorder initialized: device={:?}, hardware_sample_rate={:?}", device.name(), sample_rate);

        Ok(Self {
            recording: false,
            sample_rate: sample_rate.0,
            min_duration_secs: 1.0,
            record_start_time: None,
            audio_data: Vec::new(),
            stream: None,
            save_wav_files: true, // Default to true
            _host: host,
            recording_audio_data: None,
            stream_initialized: false,  // 🔥 初始化为false
        })
    }

    pub fn start_recording(&mut self) -> Result<(), VoiceError> {
        if self.recording {
            return Ok(());
        }

        // 🔥 如果stream已经初始化，直接开始录音，不重新创建stream
        if self.stream_initialized {
            println!("🎙️ Resuming recording (stream already active)");
            self.recording = true;
            self.record_start_time = Some(std::time::Instant::now());

            // 清空之前的音频数据
            if let Some(ref audio_data_arc) = self.recording_audio_data {
                if let Ok(mut buffer) = audio_data_arc.lock() {
                    buffer.clear();
                }
            }

            println!("Recording started");
            return Ok(());
        }

        // 首次启动，需要创建stream
        println!("🎙️ First time - initializing audio stream...");

        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| VoiceError::Audio("No default input device".to_string()))?;

        let config = device.default_input_config()
            .map_err(|e| VoiceError::Audio(format!("Failed to get input config: {}", e)))?;

        let hardware_channels = config.channels();
        // 使用硬件支持的通道数，不强制改变
        let channels = hardware_channels;
        // 使用硬件的实际采样率
        let sample_rate = config.sample_rate();

        let stream_config = StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Fixed(512), // 使用固定缓冲区大小
        };

        println!("🎙️ Stream Config: channels={}, sample_rate={:?}, sample_format={:?}",
            channels, sample_rate, config.sample_format());
        println!("🔧 Hardware supports {} channels, requesting {} channels", hardware_channels, channels);

        if hardware_channels > 1 {
            println!("📝 Note: Will convert {} hardware channels to mono for speech recognition", hardware_channels);
        }

        println!("Starting recording on device: {:?}, config: {:?}", device.name(), config);

        let audio_data = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let audio_data_clone = audio_data.clone();

        let hardware_channels = hardware_channels; // 用于闭包的副本

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // 在这里进行多声道到单声道的转换
                    let samples: Vec<f32> = if hardware_channels == 1 {
                        data.to_vec() // 已经是单声道
                    } else {
                        // 多声道转单声道：取左声道（最适合语音识别）
                        data.chunks(hardware_channels as usize)
                            .map(|chunk| chunk[0]) // 取左声道
                            .collect()
                    };
                    if let Ok(mut buffer) = audio_data_clone.lock() {
                        buffer.extend_from_slice(&samples);
                    }
                },
                |err| eprintln!("Error in input stream: {}", err),
                None,
            ).map_err(|e| VoiceError::Audio(format!("Failed to build f32 stream: {}", e)))?,

            SampleFormat::I16 => {
                let hardware_channels = hardware_channels; // 再次复制
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let samples: Vec<f32> = if hardware_channels == 1 {
                            data.iter()
                                .map(|&sample| f32::from(sample) / i16::MAX as f32)
                                .collect()
                        } else {
                            // 多声道转单声道：取左声道
                            data.chunks(hardware_channels as usize)
                                .map(|chunk| f32::from(chunk[0]) / i16::MAX as f32)
                                .collect()
                        };
                        if let Ok(mut buffer) = audio_data_clone.lock() {
                            buffer.extend_from_slice(&samples);
                        }
                    },
                    |err| eprintln!("Error in input stream: {}", err),
                    None,
                ).map_err(|e| VoiceError::Audio(format!("Failed to build i16 stream: {}", e)))?
            },

            SampleFormat::U16 => {
                let hardware_channels = hardware_channels; // 再次复制
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let samples: Vec<f32> = if hardware_channels == 1 {
                            data.iter()
                                .map(|&sample| (f32::from(sample) - u16::MAX as f32) / u16::MAX as f32)
                                .collect()
                        } else {
                            // 多声道转单声道：取左声道
                            data.chunks(hardware_channels as usize)
                                .map(|chunk| (f32::from(chunk[0]) - u16::MAX as f32) / u16::MAX as f32)
                                .collect()
                        };
                        if let Ok(mut buffer) = audio_data_clone.lock() {
                            buffer.extend_from_slice(&samples);
                        }
                    },
                    |err| eprintln!("Error in input stream: {}", err),
                    None,
                ).map_err(|e| VoiceError::Audio(format!("Failed to build u16 stream: {}", e)))?
            },

            _ => return Err(VoiceError::Audio("Unsupported sample format".to_string())),
        };

        self.audio_data = Vec::new();
        if let Ok(mut buffer) = audio_data.lock() {
            buffer.clear();
        }

        // Store the Arc to the audio data so we can retrieve it later
        self.recording_audio_data = Some(audio_data.clone());

        stream.play().map_err(|e| VoiceError::Audio(format!("Failed to play stream: {}", e)))?;

        self.stream = Some(stream);
        self.stream_initialized = true;  // 🔥 标记stream已初始化
        self.recording = true;
        self.record_start_time = Some(std::time::Instant::now());

        println!("Recording started (stream initialized)");
        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<String, VoiceError> {
        if !self.recording {
            return Err(VoiceError::Audio("Not recording".to_string()));
        }

        println!("Stopping recording...");
        self.recording = false;

        // 🔥 优化：不要销毁stream，保持它打开以便下次复用
        // if let Some(stream) = self.stream.take() {
        //     drop(stream);
        // }

        let duration = if let Some(start_time) = self.record_start_time {
            start_time.elapsed().as_secs_f64()
        } else {
            0.0
        };

        if duration < self.min_duration_secs {
            println!("Recording too short: {:.2}s < {:.2}s", duration, self.min_duration_secs);
            return Err(VoiceError::TooShort);
        }

        // Get the actual audio data for saving
        // 🔥 优化：不要take()，而是clone()，保留Arc以便下次使用
        let (recorded_samples, audio_samples) = if let Some(ref audio_data_arc) = self.recording_audio_data {
            if let Ok(buffer) = audio_data_arc.lock() {
                let sample_count = buffer.len();
                let samples = buffer.clone();
                (sample_count, samples)
            } else {
                (0, Vec::new())
            }
        } else {
            (0, Vec::new())
        };

        println!("Recording duration: {:.2}s, samples: {}", duration, recorded_samples);

        // Save audio to file
        let file_path = self.save_audio_to_file(&audio_samples)?;
        self.audio_data.clear();

        println!("Audio saved to: {}", file_path);
        Ok(file_path)
    }

    fn audio_to_wav(&self, samples: &[f32]) -> Result<Vec<u8>, VoiceError> {
        // 🎯 保存为单声道，适合语音识别
        let spec = WavSpec {
            channels: 1, // 单声道，适合语音识别
            sample_rate: self.sample_rate,
            bits_per_sample: 16, // 16位有符号整数 (s16le)
            sample_format: hound::SampleFormat::Int,
        };
        
        println!("🎵 WAV Spec: channels={}, sample_rate={}, bits_per_sample={}", 
            spec.channels, spec.sample_rate, spec.bits_per_sample);

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut cursor, spec)
                .map_err(|e| VoiceError::Audio(format!("Failed to create WAV writer: {}", e)))?;

            for &sample in samples {
                let sample_i16 = (sample * i16::MAX as f32) as i16;
                writer.write_sample(sample_i16)
                    .map_err(|e| VoiceError::Audio(format!("Failed to write sample: {}", e)))?;
            }
        }
        
        println!("💾 WAV file created: {} samples, {} bytes", samples.len(), cursor.get_ref().len());

        let wav_bytes = cursor.into_inner();
        Ok(wav_bytes)
    }

    fn save_audio_to_file(&self, samples: &[f32]) -> Result<String, VoiceError> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Create audio directory if it doesn't exist
        let audio_dir = self.get_audio_directory()?;
        
        // Generate unique filename with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| VoiceError::Audio(format!("Failed to get timestamp: {}", e)))?
            .as_secs();
        
        let filename = format!("recording_{}.wav", timestamp);
        let file_path = audio_dir.join(filename);
        
        // Convert to WAV and save to file
        let wav_bytes = self.audio_to_wav(samples)?;
        
        std::fs::write(&file_path, wav_bytes)
            .map_err(|e| VoiceError::Audio(format!("Failed to write audio file: {}", e)))?;
        
        println!("✅ Audio saved successfully: {}", file_path.display());
        
        // 验证保存的WAV文件格式
        if let Err(e) = self.verify_wav_file_format(&file_path.to_string_lossy()) {
            println!("⚠️ Failed to verify WAV file: {}", e);
        }
        
        Ok(file_path.to_string_lossy().to_string())
    }
    
    fn get_audio_directory(&self) -> Result<PathBuf, VoiceError> {
        let mut audio_dir = std::env::current_dir()
            .map_err(|e| VoiceError::Audio(format!("Failed to get current directory: {}", e)))?;

        audio_dir.push(".tauri-data");
        audio_dir.push("audio");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&audio_dir)
            .map_err(|e| VoiceError::Audio(format!("Failed to create audio directory: {}", e)))?;

        println!("📁 Audio directory: {}", audio_dir.display());
        Ok(audio_dir)
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn set_save_wav_files(&mut self, save_wav_files: bool) {
        self.save_wav_files = save_wav_files;
    }

    pub fn stop_recording_with_option(&mut self, save_to_file: bool) -> Result<String, VoiceError> {
        if !self.recording {
            return Err(VoiceError::Audio("Not recording".to_string()));
        }

        println!("Stopping recording...");
        self.recording = false;

        // 🔥 优化：不要销毁stream，保持它打开以便下次复用
        // if let Some(stream) = self.stream.take() {
        //     drop(stream);
        // }

        let duration = if let Some(start_time) = self.record_start_time {
            start_time.elapsed().as_secs_f64()
        } else {
            0.0
        };

        if duration < self.min_duration_secs {
            println!("Recording too short: {:.2}s < {:.2}s", duration, self.min_duration_secs);
            return Err(VoiceError::TooShort);
        }

        println!("Recording duration: {:.2}s, samples: {}", duration, self.audio_data.len());

        // Get the actual audio data from recording buffer
        // 🔥 优化：不要take()，而是clone()，保留Arc以便下次使用
        let (recorded_samples, audio_samples) = if let Some(ref audio_data_arc) = self.recording_audio_data {
            if let Ok(buffer) = audio_data_arc.lock() {
                let sample_count = buffer.len();
                let samples = buffer.clone();
                (sample_count, samples)
            } else {
                (0, Vec::new())
            }
        } else {
            (0, Vec::new())
        };

        println!("Recording duration: {:.2}s, samples: {}", duration, recorded_samples);

        if save_to_file {
            // Save audio to file
            let file_path = self.save_audio_to_file(&audio_samples)?;
            println!("✅ Audio saved to: {}", file_path);
            Ok(file_path)
        } else {
            // Don't save file, but provide audio data for processing
            println!("⏭️ Skipping file save (Save WAV Files = false)");
            // Return a placeholder path to indicate successful recording without file save
            Ok(format!("memory://audio_data_{}_samples", recorded_samples))
        }
    }

    pub fn get_audio_data(&self) -> Vec<f32> {
        // Get audio data from the recording buffer
        if let Some(audio_data_arc) = &self.recording_audio_data {
            if let Ok(buffer) = audio_data_arc.lock() {
                return buffer.clone();
            }
        }
        // Fallback to internal audio_data
        self.audio_data.clone()
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    // ========== Streaming support methods ==========

    /// 获取已录制的音频块（用于流式处理）
    /// 这个方法会克隆当前缓冲区的音频数据，用于传递给流式处理器
    pub fn get_recorded_audio_chunk(&self) -> Vec<f32> {
        if let Some(audio_data_arc) = &self.recording_audio_data {
            if let Ok(buffer) = audio_data_arc.lock() {
                return buffer.clone();
            }
        }
        self.audio_data.clone()
    }

    /// 清空录制缓冲区（流式块处理完成后调用）
    /// 注意：这会清空所有已录制但未处理的音频数据
    pub fn clear_recording_buffer(&mut self) {
        if let Some(audio_data_arc) = &self.recording_audio_data {
            if let Ok(mut buffer) = audio_data_arc.lock() {
                buffer.clear();
            }
        }
        self.audio_data.clear();
    }

    /// 获取当前缓冲区的音频样本数量
    pub fn get_buffer_length(&self) -> usize {
        if let Some(audio_data_arc) = &self.recording_audio_data {
            if let Ok(buffer) = audio_data_arc.lock() {
                return buffer.len();
            }
        }
        self.audio_data.len()
    }

    /// 验证WAV文件格式 - 用于调试
    pub fn verify_wav_file_format(&self, file_path: &str) -> Result<(), VoiceError> {
        use std::fs::File;
        use std::io::BufReader;
        
        let file = File::open(file_path)
            .map_err(|e| VoiceError::Audio(format!("Failed to open WAV file: {}", e)))?;
        let mut reader = BufReader::new(file);
        
        // 使用hound读取WAV头信息
        let wav_reader = hound::WavReader::new(&mut reader)
            .map_err(|e| VoiceError::Audio(format!("Failed to read WAV header: {}", e)))?;
        
        let spec = wav_reader.spec();
        let duration = wav_reader.duration();
        
        println!("🔍 WAV File Verification:");
        println!("  📁 Path: {}", file_path);
        println!("  🎵 Channels: {}", spec.channels);
        println!("  🔢 Sample Rate: {}", spec.sample_rate);
        println!("  🔢 Bits per sample: {}", spec.bits_per_sample);
        println!("  🔢 Sample format: {:?}", spec.sample_format);
        println!("  ⏱️ Duration: {} samples ({:.2} seconds)", 
            duration, duration as f64 / spec.sample_rate as f64);
        println!("  📊 Expected format: (44100 Hz, 2ch, s16le) - Your simulation file");
        println!("  📊 Original system: (44100 Hz, 1ch, s16le)");
        println!("  📊 Actual format:   ({} Hz, {}ch, s{:?})", 
            spec.sample_rate, spec.channels, spec.bits_per_sample);
        
        // 验证是否符合你的模拟文件格式
        let correct_sample_rate = spec.sample_rate == 44100;
        let correct_channels = spec.channels == 2; // 你的模拟文件使用2通道
        let correct_bits = spec.bits_per_sample == 16;
        
        if correct_sample_rate && correct_channels && correct_bits {
            println!("  ✅ Format matches your simulation file (44100 Hz, 2ch, s16le)");
        } else {
            println!("  ⚠️ Format mismatch detected!");
            if spec.channels == 1 {
                println!("  💡 Note: Original system used mono (1ch), now using stereo (2ch)");
            }
        }
        
        Ok(())
    }
}

impl Drop for AudioRecorder {
    fn drop(&mut self) {
        if self.recording {
            let _ = self.stop_recording();
        }
    }
}