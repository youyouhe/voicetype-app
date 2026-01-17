use rdev::{listen, EventType, Key};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::process::Command;
use std::thread::{self, JoinHandle};
use crate::voice_assistant::{KeyboardManagerTrait, AsrProcessor, TranslateProcessor, InputState, VoiceError};
use crate::voice_assistant::hotkey_parser::ParsedHotkey;
use std::collections::HashSet;
use crate::database::TypingDelays;

pub struct KeyboardManager {
    state: Arc<Mutex<InputState>>,
    asr_processor: Arc<dyn AsrProcessor + Send + Sync>,
    translate_processor: Option<Arc<dyn TranslateProcessor + Send + Sync>>,
    // 热键配置
    transcribe_hotkey: Arc<Mutex<Option<ParsedHotkey>>>,
    translate_hotkey: Arc<Mutex<Option<ParsedHotkey>>>,
    // 按键状态跟踪
    pressed_keys: Arc<Mutex<HashSet<Key>>>,
    hotkey_start_time: Arc<Mutex<Option<Instant>>>,
    temp_text_length: Arc<Mutex<usize>>,
    original_clipboard: Arc<Mutex<Option<String>>>,
    // WAV文件保存配置
    save_wav_files: Arc<Mutex<bool>>,
    // 延迟配置
    typing_delays: Arc<Mutex<TypingDelays>>,
    // 流式支持字段
    streaming_session: Arc<Mutex<Option<Box<dyn crate::voice_assistant::StreamingAsrSession>>>>,
    streaming_enabled: Arc<Mutex<bool>>,
    streaming_chunk_interval_ms: Arc<Mutex<u64>>,
    #[allow(dead_code)]
    streaming_last_process_time: Arc<Mutex<Option<Instant>>>,
    streaming_thread_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    streaming_stop_signal: Arc<Mutex<bool>>,
}

impl KeyboardManager {
    pub fn new(
        asr_processor: Arc<dyn AsrProcessor + Send + Sync>,
        translate_processor: Option<Arc<dyn TranslateProcessor + Send + Sync>>,
    ) -> Result<Self, VoiceError> {
        Ok(Self {
            state: Arc::new(Mutex::new(InputState::Idle)),
            asr_processor,
            translate_processor,
            transcribe_hotkey: Arc::new(Mutex::new(None)),
            translate_hotkey: Arc::new(Mutex::new(None)),
            pressed_keys: Arc::new(Mutex::new(HashSet::new())),
            hotkey_start_time: Arc::new(Mutex::new(None)),
            temp_text_length: Arc::new(Mutex::new(0)),
            original_clipboard: Arc::new(Mutex::new(None)),
            save_wav_files: Arc::new(Mutex::new(false)), // Default to false
            typing_delays: Arc::new(Mutex::new(TypingDelays::default())),
            // 流式字段初始化
            streaming_session: Arc::new(Mutex::new(None)),
            streaming_enabled: Arc::new(Mutex::new(false)),
            streaming_chunk_interval_ms: Arc::new(Mutex::new(500)),
            streaming_last_process_time: Arc::new(Mutex::new(None)),
            streaming_thread_handle: Arc::new(Mutex::new(None)),
            streaming_stop_signal: Arc::new(Mutex::new(false)),
        })
    }

    /// 🔥 更新处理器引用 - 用于配置刷新
    pub fn update_processors(
        &mut self,
        new_asr_processor: Option<Arc<dyn AsrProcessor + Send + Sync>>,
        new_translate_processor: Option<Arc<dyn TranslateProcessor + Send + Sync>>,
    ) -> Result<(), VoiceError> {
        println!("🔄 Updating KeyboardManager processors...");

        // 更新处理器引用
        if let Some(asr) = new_asr_processor {
            self.asr_processor = asr;
        }
        self.translate_processor = new_translate_processor;

        println!("✅ KeyboardManager processors updated successfully");
        Ok(())
    }

    /// 🔥 清除处理器引用 - 用于停止服务时释放模型内存
    pub fn clear_processors(&mut self) {
        println!("🗑️ KeyboardManager: Clearing processor references to free memory...");
        // 将 ASR 处理器替换为一个空的默认实现
        // 这样可以释放原有的 Arc 引用
        self.asr_processor = Arc::new(DefaultAsrProcessor);
        self.translate_processor = None;
        println!("✅ KeyboardManager: Processor references cleared");
    }

    /// 设置热键配置
    pub fn set_hotkeys(&mut self, transcribe_key: &str, translate_key: &str) -> Result<(), VoiceError> {
        println!("🔧 Setting hotkeys:");
        println!("  - Transcribe: {}", transcribe_key);
        println!("  - Translate: {}", translate_key);

        // 解析热键
        let transcribe_parsed = ParsedHotkey::parse(transcribe_key)
            .map_err(|e| VoiceError::Audio(format!("Failed to parse transcribe hotkey: {}", e)))?;
        
        let translate_parsed = ParsedHotkey::parse(translate_key)
            .map_err(|e| VoiceError::Audio(format!("Failed to parse translate hotkey: {}", e)))?;

        println!("✅ Parsed hotkeys successfully");
        
        *self.transcribe_hotkey.lock().unwrap() = Some(transcribe_parsed);
        *self.translate_hotkey.lock().unwrap() = Some(translate_parsed);
        
        Ok(())
    }

    pub fn start_listening(&mut self) {
        let state = self.state.clone();
        let _asr_processor = self.asr_processor.clone();
        let _translate_processor = self.translate_processor.clone();
        let transcribe_hotkey = self.transcribe_hotkey.clone();
        let translate_hotkey = self.translate_hotkey.clone();
        let pressed_keys = self.pressed_keys.clone();
        let hotkey_start_time = self.hotkey_start_time.clone();
        let temp_text_length = self.temp_text_length.clone();
        let original_clipboard = self.original_clipboard.clone();

        // 克隆流式相关字段
        let streaming_session = self.streaming_session.clone();
        let streaming_enabled = self.streaming_enabled.clone();
        let streaming_chunk_interval_ms = self.streaming_chunk_interval_ms.clone();
        let streaming_stop_signal = self.streaming_stop_signal.clone();
        let streaming_thread_handle = self.streaming_thread_handle.clone();

        // Use tokio::task::spawn_blocking to avoid runtime conflicts with rdev
        // 获取save_wav_files配置传递到回调中
        let save_wav_files_config = *self.save_wav_files.lock().unwrap();
        println!("📁 Save WAV Files setting from config: {}", save_wav_files_config);

        // 克隆延迟配置以便在闭包中使用
        let typing_delays_for_callback = self.typing_delays.clone();

        tokio::task::spawn_blocking(move || {
            // 将 recorder 包装成 Arc<Mutex<>> 以便流式线程访问
            let recorder = Arc::new(Mutex::new(None::<crate::voice_assistant::AudioRecorder>));
            let recorder_for_stream = recorder.clone();

            // 使用传递过来的save_wav_files配置
            let save_wav_files = save_wav_files_config;
            println!("📁 Save WAV Files setting in callback: {}", save_wav_files);
            let mut last_state = InputState::Idle;
            let mut recording_started = false;
            let mut hotkey_press_time: Option<Instant> = None;
            const HOTKEY_DELAY_THRESHOLD: Duration = Duration::from_millis(300); // 防误触阈值

            // 定义流式模式使用的 mode
            let mode = crate::voice_assistant::Mode::Transcriptions;

            if let Err(e) = listen(move |event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        // 🔥 关键优化：在非Idle状态下，提前返回忽略所有按键
                        let current_state = *state.lock().unwrap();
                        if current_state != InputState::Idle {
                            // 不打印日志，完全静默忽略所有按键事件
                            return;
                        }

                        let mut keys = pressed_keys.lock().unwrap();
                        // 只有当按键是新的时候才记录日志和插入
                        let is_new_key = !keys.contains(&key);
                        if is_new_key {
                            println!("⌨️  KeyPress detected: {:?}", key);
                        }
                        keys.insert(key);
                        
                        // 检查是否应该开始录音
                        let transcribe_hotkey_guard = transcribe_hotkey.lock().unwrap();
                        let translate_hotkey_guard = translate_hotkey.lock().unwrap();
                        let current_state = *state.lock().unwrap();
                        
                        // 只在有按键变化时输出详细日志
                        if is_new_key {
                            println!("🔑 Current state: {:?}, Recording started: {}", current_state, recording_started);
                            println!("🔑 Pressed keys: {:?}", keys);
                        }
                        
                        // 检查转录热键
                        if let Some(ref transcribe_hotkey) = *transcribe_hotkey_guard {
                            // 🔥 只在Idle状态下响应热键，避免enigo模拟输入触发死循环
                            if transcribe_hotkey.matches(&*keys) && current_state.can_start_recording() && !recording_started && current_state == InputState::Idle {
                                // 检查按键持续时间（防误触）
                                let current_time = Instant::now();
                                let should_trigger = if let Some(press_time) = hotkey_press_time {
                                    current_time.duration_since(press_time) >= HOTKEY_DELAY_THRESHOLD
                                } else {
                                    // 首次按下，记录时间但不触发
                                    hotkey_press_time = Some(current_time);
                                    false
                                };

                                if should_trigger {
                                    // 🔥 根据streaming_enabled决定使用哪种模式
                                    let is_streaming_enabled = *streaming_enabled.lock().unwrap();

                                    if is_streaming_enabled {
                                        println!("🎯 Transcribe hotkey pressed - starting STREAMING state...");
                                    } else {
                                        println!("🎤 Transcribe hotkey pressed - starting RECORDING state...");
                                    }

                                    // IMPORTANT: Clear keys immediately to prevent repeated triggers
                                    keys.clear();

                                    *hotkey_start_time.lock().unwrap() = Some(Instant::now());

                                    // 🔥 根据streaming配置设置状态
                                    let new_state = if is_streaming_enabled {
                                        InputState::Streaming
                                    } else {
                                        InputState::Recording
                                    };
                                    *state.lock().unwrap() = new_state.clone();

                                    // Emit state change event
                                    crate::voice_assistant::coordinator::emit_voice_assistant_state_from_keyboard(&new_state);
                                    recording_started = true;
                                    hotkey_press_time = None; // 重置按键时间
                                }

                                // 保存原始剪贴板
                                let mut clipboard = original_clipboard.lock().unwrap();
                                if clipboard.is_none() {
                                    if let Ok(content) = get_clipboard_content() {
                                        *clipboard = Some(content);
                                    }
                                }
                            }
                        }
                        
                        // 检查翻译热键
                        if let Some(ref translate_hotkey) = *translate_hotkey_guard {
                            // 🔥 只在Idle状态下响应热键，避免enigo模拟输入触发死循环
                            if translate_hotkey.matches(&*keys) && current_state.can_start_recording() && !recording_started && current_state == InputState::Idle {
                                // 检查按键持续时间（防误触）
                                let current_time = Instant::now();
                                let should_trigger = if let Some(press_time) = hotkey_press_time {
                                    current_time.duration_since(press_time) >= HOTKEY_DELAY_THRESHOLD
                                } else {
                                    // 首次按下，记录时间但不触发
                                    hotkey_press_time = Some(current_time);
                                    false
                                };

                                if should_trigger {
                                    println!("🌐 Translate hotkey pressed - starting recording translate state...");

                                    // IMPORTANT: Clear keys immediately to prevent repeated triggers
                                    keys.clear();

                                    *hotkey_start_time.lock().unwrap() = Some(Instant::now());
                                    *state.lock().unwrap() = InputState::RecordingTranslate; // Start recording translate state
                                    // Emit state change event
                                    crate::voice_assistant::coordinator::emit_voice_assistant_state_from_keyboard(&InputState::RecordingTranslate);
                                    recording_started = true;
                                    hotkey_press_time = None; // 重置按键时间
                                }

                                // 保存原始剪贴板
                                let mut clipboard = original_clipboard.lock().unwrap();
                                if clipboard.is_none() {
                                    if let Ok(content) = get_clipboard_content() {
                                        *clipboard = Some(content);
                                    }
                                }
                            }
                        }
                    }
                    
                    EventType::KeyRelease(key) => {
                        // 🔥 优化：在非录音状态下，提前返回忽略所有按键释放事件
                        let current_state = *state.lock().unwrap();
                        if !matches!(current_state, InputState::Recording | InputState::RecordingTranslate | InputState::Streaming | InputState::Idle) {
                            // 在Processing/Translating等状态下，完全忽略按键释放
                            return;
                        }

                        let mut keys = pressed_keys.lock().unwrap();
                        println!("🔓 KeyRelease detected: {:?}", key);
                        keys.remove(&key);
                        println!("🔑 Remaining keys after release: {:?}", keys);

                        // 重置按键时间戳（当所有按键都释放时）
                        if keys.is_empty() {
                            hotkey_press_time = None;

                            // 检查是否在录音/流式状态，如果是，则转换到处理状态
                            match current_state {
                                InputState::Recording => {
                                    println!("🎤 Transcribe hotkey released - switching to Processing state...");
                                    *state.lock().unwrap() = InputState::Processing;
                                    // Emit state change event
                                    crate::voice_assistant::coordinator::emit_voice_assistant_state_from_keyboard(&InputState::Processing);
                                }
                                InputState::RecordingTranslate => {
                                    println!("🌐 Translate hotkey released - switching to Translating state...");
                                    *state.lock().unwrap() = InputState::Translating;
                                    // Emit state change event
                                    crate::voice_assistant::coordinator::emit_voice_assistant_state_from_keyboard(&InputState::Translating);
                                }
                                // 🔥 Streaming模式：松开F4后继续streaming，处理结果
                                InputState::Streaming => {
                                    println!("🎯 Streaming hotkey released - finalizing streaming...");
                                    *state.lock().unwrap() = InputState::StreamingFinalizing;
                                    // Emit state change event
                                    crate::voice_assistant::coordinator::emit_voice_assistant_state_from_keyboard(&InputState::StreamingFinalizing);
                                }
                                _ => {}
                            }
                        }

                        // For direct processing, state reset happens in the processing handlers
                        // We don't need to reset state here anymore
                    }
                    _ => {}
                }

                // 状态变化检测和处理
                let current_state = *state.lock().unwrap();
                if current_state != last_state {
                    last_state = current_state;

                    match current_state {
                        InputState::Recording => {
                            // 开始转录录音
                            println!("🎤 Recording state - starting real audio recording...");
                            Self::start_recording_internal(&recorder, save_wav_files);
                        }
                        InputState::RecordingTranslate => {
                            // 开始翻译录音
                            println!("🌐 Recording Translate state - starting real audio recording...");
                            Self::start_recording_internal(&recorder, save_wav_files);
                        }
                        InputState::Processing => {
                            // Process recorded audio with real ASR
                            println!("🔄 Entering Processing state...");
                            println!("🎙️ Processing audio with real ASR...");

                            // Stop recording and get audio data
                            // Process ASR - can now be done synchronously since we use spawn_blocking internally
                            let asr_result = {
                                let (audio_data, sample_rate, has_recorder) = if let Some(rec) = recorder.lock().unwrap().as_ref() {
                                    println!("🛑 Stopping recording...");

                                    // Get audio data BEFORE stopping recording (to avoid data loss)
                                    let audio_data = rec.get_audio_data();
                                    let sample_rate = rec.get_sample_rate();
                                    println!("📊 Got audio data: {} samples", audio_data.len());

                                    (audio_data, sample_rate, true)
                                } else {
                                    println!("❌ No recorder available");
                                    (Vec::new(), 0, false)
                                };

                                if has_recorder && !audio_data.is_empty() {
                                    // Stop recording
                                    if let Some(ref mut rec) = *recorder.lock().unwrap() {
                                        let _ = rec.stop_recording_with_option(save_wav_files);
                                    }

                                    // Convert to WAV format for ASR processing
                                    match Self::convert_to_wav_bytes(&audio_data, sample_rate) {
                                        Ok(wav_bytes) => {
                                            println!("🔄 Converting {} audio samples to WAV format ({} bytes)", audio_data.len(), wav_bytes.len());

                                            // Process with ASR - this now uses spawn_blocking internally
                                            use std::io::Cursor;
                                            match _asr_processor.process_audio(Cursor::new(wav_bytes), crate::voice_assistant::Mode::Transcriptions, "") {
                                                Ok(result) => {
                                                    println!("✅ ASR processing successful");
                                                    Some(result)
                                                }
                                                Err(e) => {
                                                    println!("❌ ASR processing failed: {}", e);
                                                    Some(format!("ASR Error: {}", e))
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!("❌ Failed to convert audio to WAV: {}", e);
                                            Some(format!("Audio conversion error: {}", e))
                                        }
                                    }
                                } else if has_recorder && audio_data.is_empty() {
                                    Some("No audio recorded - please check microphone".to_string())
                                } else {
                                    Some("No recorder available".to_string())
                                }
                            };

                            // Use the ASR result
                            if let Some(result_text) = asr_result {
                                println!("⌨️ Typing ASR result: \"{}\"", result_text);
                                
                                // Calculate processing time
                                let processing_time = if let Some(start_time) = hotkey_start_time.lock().unwrap().as_ref() {
                                    Some(start_time.elapsed().as_millis() as i64)
                                } else {
                                    None
                                };
                                
                                // Use tokio runtime to save to database
                                if let Ok(tokio_rt) = tokio::runtime::Runtime::new() {
                                    let result_text_clone = result_text.clone();
                                    let processor_type = _asr_processor.get_processor_type().unwrap_or("unknown").to_string();
                                    tokio_rt.block_on(async move {
                                        crate::voice_assistant::coordinator::save_asr_result_directly(
                                            result_text_clone,
                                            &processor_type,
                                            processing_time,
                                            true,
                                            None
                                        ).await;
                                    });
                                    
                                    println!("✅ Database save operation completed");
                                }
                                
                                Self::type_text_internal(&state, &temp_text_length, &original_clipboard, &result_text, None, &typing_delays_for_callback.lock().unwrap());
                                println!("✅ ASR result typing completed");
                            }

                            // Reset recorder for next use
                            *recorder.lock().unwrap() = None;

                            // IMPORTANT: Reset state and flags after processing
                            println!("🔄 Resetting state after processing completion...");
                            recording_started = false;
                            *hotkey_start_time.lock().unwrap() = None;
                            *state.lock().unwrap() = InputState::Idle;
                        // Emit state change event
                        crate::voice_assistant::coordinator::emit_voice_assistant_state_from_keyboard(&InputState::Idle);
                        }
                        InputState::Translating => {
                            // 🔥 使用whisper.cpp内置的翻译功能
                            println!("🔄 Entering Translating state...");
                            println!("🌐 Using whisper.cpp built-in translation (speech → English text)...");

                            // Get audio data and sample rate BEFORE stopping recording
                            let (audio_data, sample_rate, has_recorder) = if let Some(rec) = recorder.lock().unwrap().as_ref() {
                                println!("🛑 Stopping recording for translation...");

                                let audio_data = rec.get_audio_data();
                                let sample_rate = rec.get_sample_rate();
                                println!("📊 Got audio data: {} samples", audio_data.len());

                                (audio_data, sample_rate, true)
                            } else {
                                (Vec::new(), 0, false)
                            };

                            // Stop recording
                            if has_recorder {
                                if let Some(ref mut rec) = *recorder.lock().unwrap() {
                                    let _ = rec.stop_recording();
                                }
                            }

                            // Convert to WAV bytes
                            let wav_bytes_result = Self::convert_to_wav_bytes(&audio_data, sample_rate);

                            let final_result = match wav_bytes_result {
                                Ok(wav_bytes) => {
                                    let audio_cursor = std::io::Cursor::new(wav_bytes);
                                    println!("🎵 Converted audio to WAV format");

                                    // 🔥 关键：使用 Mode::Translations 让whisper直接翻译成英文
                                    let start = std::time::Instant::now();
                                    let translation = _asr_processor.process_audio(
                                        audio_cursor,
                                        crate::voice_assistant::Mode::Translations,  // 🔥 翻译模式
                                        ""
                                    );
                                    let processing_time = start.elapsed().as_millis() as i64;

                                    match translation {
                                        Ok(translated_text) => {
                                            println!("✅ Whisper translation result: \"{}\"", translated_text);
                                            println!("⏱️ Processing time: {}ms", processing_time);
                                            Some(translated_text)
                                        }
                                        Err(e) => {
                                            println!("❌ Whisper translation error: {}", e);
                                            Some(format!("❌ Translation failed: {}", e))
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("❌ Failed to convert audio to WAV: {}", e);
                                    Some(format!("❌ Audio conversion failed: {}", e))
                                }
                            };

                            if !has_recorder {
                                println!("⚠️ No recorder found, nothing to translate");
                            }

                            // Type the result
                            if let Some(result_text) = final_result {
                                let state_clone = state.clone();
                                let temp_len_clone = temp_text_length.clone();
                                let clipboard_clone = original_clipboard.clone();

                                println!("⌨️ Typing translation result: \"{}\"", result_text);
                                Self::type_text_internal(&state_clone, &temp_len_clone, &clipboard_clone, &result_text, None, &typing_delays_for_callback.lock().unwrap());
                                println!("✅ Translation result typing completed");
                            }

                            // IMPORTANT: Reset state and flags immediately after processing
                            println!("🔄 Resetting state after translation completion...");
                            *recorder.lock().unwrap() = None;
                            recording_started = false;
                            *hotkey_start_time.lock().unwrap() = None;
                            *state.lock().unwrap() = InputState::Idle;
                        // Emit state change event
                        crate::voice_assistant::coordinator::emit_voice_assistant_state_from_keyboard(&InputState::Idle);
                        }
                        // ========== Streaming states ==========
                        InputState::Streaming => {
                            println!("🎯 Streaming state - starting streaming session...");

                            // 启动录音
                            Self::start_recording_internal(&recorder, save_wav_files);

                            // 启动流式会话
                            match Self::start_streaming_session_internal(mode, &_asr_processor) {
                                Ok(session) => {
                                    *streaming_session.lock().unwrap() = Some(session);
                                    // 启动流式处理线程
                                    Self::start_streaming_processor_thread(
                                        &recorder_for_stream,
                                        &_asr_processor,
                                        &streaming_session,
                                        &streaming_enabled,
                                        &streaming_chunk_interval_ms,
                                        &streaming_stop_signal,
                                        &streaming_thread_handle,
                                    );
                                }
                                Err(e) => {
                                    println!("❌ Failed to start streaming session: {}", e);
                                    *state.lock().unwrap() = InputState::Idle;
                                    recording_started = false;
                                    *hotkey_start_time.lock().unwrap() = None;
                                }
                            }
                        }
                        InputState::StreamingFinalizing => {
                            println!("🎯 StreamingFinalizing state - stopping streaming session...");

                            // 停止流式处理线程
                            *streaming_stop_signal.lock().unwrap() = true;

                            // 等待线程结束（带超时）
                            if let Some(handle) = streaming_thread_handle.lock().unwrap().take() {
                                let _ = handle.join();
                            }
                            *streaming_stop_signal.lock().unwrap() = false;

                            // 结束流式会话，获取最终结果
                            match Self::finalize_streaming_session_internal(&streaming_session) {
                                Ok(final_text) => {
                                    if !final_text.is_empty() {
                                        println!("✅ Streaming final result: \"{}\"", final_text);
                                        Self::type_text_internal(&state, &temp_text_length, &original_clipboard, &final_text, None, &typing_delays_for_callback.lock().unwrap());
                                    }
                                }
                                Err(e) => {
                                    println!("❌ Failed to finalize streaming session: {}", e);
                                }
                            }

                            // 停止录音（但保留 recorder 实例以便下次复用）
                            if let Some(ref mut rec) = *recorder.lock().unwrap() {
                                let _ = rec.stop_recording_with_option(save_wav_files);
                            }

                            // 🔥 优化：不清空 recorder，保留 AudioRecorder 实例以便下次复用
                            // 这样可以避免重新初始化麦克风

                            // Reset state
                            recording_started = false;
                            *hotkey_start_time.lock().unwrap() = None;
                            *state.lock().unwrap() = InputState::Idle;
                            crate::voice_assistant::coordinator::emit_voice_assistant_state_from_keyboard(&InputState::Idle);
                        }
                        _ => {}
                    }
                }

            }) {
                eprintln!("Error listening for keyboard events: {:?}", e);
            }
        });
    }

    fn convert_to_wav_bytes(audio_data: &[f32], sample_rate: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use hound::{WavWriter, WavSpec};

    let spec = WavSpec {
        channels: 1,
        sample_rate: sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    let mut writer = WavWriter::new(&mut cursor, spec)?;

    // Convert f32 samples to i16
    for &sample in audio_data {
        let i16_sample = (sample * i16::MAX as f32) as i16;
        writer.write_sample(i16_sample)?;
    }

    writer.finalize()?;
    Ok(cursor.into_inner())
}

fn start_recording_internal(recorder: &Arc<Mutex<Option<crate::voice_assistant::AudioRecorder>>>, save_wav_files: bool) {
        let mut recorder_guard = recorder.lock().unwrap();

        if let Some(ref mut rec) = *recorder_guard {
            // 🔥 优化：复用已存在的 AudioRecorder
            println!("♻️ Reusing existing AudioRecorder");
            rec.set_save_wav_files(save_wav_files);
            if let Err(e) = rec.start_recording() {
                eprintln!("Failed to start recording: {}", e);
            } else {
                println!("🎙️ Recording restarted (Save WAV: {})", save_wav_files);
            }
        } else {
            // 创建新的 AudioRecorder（首次使用）
            match crate::voice_assistant::AudioRecorder::new() {
                Ok(mut r) => {
                    r.set_save_wav_files(save_wav_files);

                    if let Err(e) = r.start_recording() {
                        eprintln!("Failed to start recording: {}", e);
                    } else {
                        println!("🎙️ Recording started (Save WAV: {})", save_wav_files);
                        *recorder_guard = Some(r);
                    }
                }
                Err(e) => eprintln!("Failed to create recorder: {}", e),
            }
        }
    }

    fn type_text_internal(
        state: &Arc<Mutex<InputState>>,
        _temp_text_length: &Arc<Mutex<usize>>,
        original_clipboard: &Arc<Mutex<Option<String>>>,
        text: &str,
        error: Option<&str>,
        _delays: &TypingDelays,
    ) {
        // 🔥 禁用temp_text_length机制，避免模拟退格触发rdev死循环
        // 剪贴板输入已经可靠，不需要删除临时文本
        println!("⌨️ Skipping temp_text_length cleanup (using clipboard input)");

        if let Some(err_msg) = error {
            // 显示错误消息
            simulate_typing(&format!("❌ {}", err_msg), _delays);

            // 2秒后清除错误消息 - use std sleep instead of tokio
            let state_clone = state.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(2));
                if *state_clone.lock().unwrap() == InputState::Error {
                    *state_clone.lock().unwrap() = InputState::Idle;
                    // 🔥 不再删除临时文本，避免死循环
                }
            });

            *state.lock().unwrap() = InputState::Error;
        } else if !text.is_empty() {
            // 输入最终文本
            simulate_typing(text, _delays);

            // 恢复剪贴板
            let mut clipboard = original_clipboard.lock().unwrap();
            if let Some(content) = clipboard.take() {
                set_clipboard_content(&content);
            }
        }

        *state.lock().unwrap() = InputState::Idle;
    }

    pub fn reset_state(&mut self) {
        *self.state.lock().unwrap() = InputState::Idle;
        *self.temp_text_length.lock().unwrap() = 0;
        self.pressed_keys.lock().unwrap().clear();
        *self.hotkey_start_time.lock().unwrap() = None;

        // 🔥 不再删除临时文本，避免enigo模拟退格触发rdev死循环
        println!("🔄 State reset (skipping temp_text cleanup)");

        // 恢复剪贴板
        let mut clipboard = self.original_clipboard.lock().unwrap();
        if let Some(content) = clipboard.take() {
            set_clipboard_content(&content);
        }
    }

    // 可配置热键方法
    pub fn set_transcribe_hotkey(&self, hotkey_str: &str) -> Result<(), VoiceError> {
        let _parsed_hotkey = crate::voice_assistant::hotkey_parser::ParsedHotkey::parse(hotkey_str)
            .map_err(|e| VoiceError::Other(e))?;
        // 由于我们使用简单的版本，暂时只打印日志
        println!("Setting transcribe hotkey: {}", hotkey_str);
        Ok(())
    }

    pub fn set_translate_hotkey(&self, hotkey_str: &str) -> Result<(), VoiceError> {
        let _parsed_hotkey = crate::voice_assistant::hotkey_parser::ParsedHotkey::parse(hotkey_str)
            .map_err(|e| VoiceError::Other(e))?;
        // 由于我们使用简单的版本，暂时只打印日志
        println!("Setting translate hotkey: {}", hotkey_str);
        Ok(())
    }

    pub fn set_trigger_delay_ms(&self, delay_ms: i64) {
        println!("Setting trigger delay: {}ms", delay_ms);
    }

    pub fn set_anti_mistouch_enabled(&self, enabled: bool) {
        println!("Setting anti-mistouch: {}", enabled);
    }

    /// 设置WAV文件保存开关
    pub fn set_save_wav_files(&self, save_wav_files: bool) {
        let mut setting = self.save_wav_files.lock().unwrap();
        *setting = save_wav_files;
        println!("🔧 Save WAV Files setting updated to: {}", save_wav_files);
    }

    /// 设置延迟配置
    pub fn set_typing_delays(&self, typing_delays: TypingDelays) {
        let mut delays = self.typing_delays.lock().unwrap();
        *delays = typing_delays;
        println!("🔧 Typing delays updated:");
        println!("  - clipboard_update_ms: {}ms", delays.clipboard_update_ms);
        println!("  - keyboard_events_settle_ms: {}ms", delays.keyboard_events_settle_ms);
        println!("  - typing_complete_ms: {}ms", delays.typing_complete_ms);
        println!("  - character_interval_ms: {}ms", delays.character_interval_ms);
        println!("  - short_operation_ms: {}ms", delays.short_operation_ms);
    }
}

impl KeyboardManagerTrait for KeyboardManager {
    fn start_listening(&mut self) {
        self.start_listening();
    }

    fn type_text(&mut self, _text: &str, _error: Option<&str>) {
        // This is handled internally by the state machine
    }

    fn reset_state(&mut self) {
        self.reset_state();
    }
}

fn simulate_typing(text: &str, _delays: &TypingDelays) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&format!(
                "tell application \"System Events\" to keystroke \"{}\"",
                text.replace("\"", "\\\"").replace("\n", "\\n")
            ))
            .output();

        if let Err(e) = output {
            eprintln!("Failed to type text: {}", e);
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 🔥 Windows平台：使用逐字符模拟键盘输入
        println!("⌨️ Using keyboard simulation for character-by-character input...");
        println!("✅ Text to type: \"{}\"", text);

        type_text_by_keypress(text);

        println!("✅ Keyboard simulation completed");
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 使用剪贴板粘贴方法，更可靠支持中文
        println!("📋 Using clipboard paste method for Linux...");

        // 保存当前剪贴板内容
        let current_clipboard = match get_clipboard_content() {
            Ok(content) => Some(content),
            Err(_) => {
                eprintln!("Warning: Failed to get current clipboard content");
                None
            }
        };

        // 将文本设置到剪贴板
        set_clipboard_content(text);

        // 等待剪贴板更新
        std::thread::sleep(std::time::Duration::from_millis(delays.clipboard_update_ms as u64));

        // For xterm and other terminals, direct typing is more reliable than clipboard paste
        println!("🔧 Using direct typing for terminal compatibility...");

        // Method 1: Try direct typing first (most reliable for terminals)
        if let Ok(_) = type_text_direct(text, delays) {
            println!("✅ Direct typing successful");
        } else {
            println!("🔧 Direct typing failed, trying clipboard methods...");

            // Method 2: Try Ctrl+Shift+V for terminal paste as fallback
            std::thread::sleep(std::time::Duration::from_millis(delays.short_operation_ms as u64));
            if let Ok(output) = Command::new("xdotool")
                .args(&["key", "Ctrl+Shift+V"])
                .output()
            {
                if output.status.success() {
                    println!("✅ Ctrl+Shift+V paste successful");
                } else {
                    eprintln!("Ctrl+Shift+V failed: {:?}", String::from_utf8_lossy(&output.stderr));

                    // Method 3: Try middle-click paste
                    std::thread::sleep(std::time::Duration::from_millis(delays.short_operation_ms as u64));
                    if let Ok(output2) = Command::new("xdotool")
                        .args(&["click", "2"])
                        .output()
                    {
                        if output2.status.success() {
                            println!("✅ Middle-click paste successful");
                        } else {
                            eprintln!("All paste methods failed");
                        }
                    }
                }
            } else {
                eprintln!("xdotool not found");
            }
        }

        // 等待粘贴完成
        std::thread::sleep(std::time::Duration::from_millis(delays.short_operation_ms as u64));

        // 恢复原始剪贴板内容
        if let Some(original) = current_clipboard {
            set_clipboard_content(&original);
        }

        println!("✅ Clipboard paste completed");
    }
}

/// Windows: 逐字符模拟键盘输入（支持Unicode）
#[cfg(target_os = "windows")]
fn type_text_by_keypress(text: &str) {
    // 🔥 统一使用 Unicode 输入方式，完全绕过输入法和虚拟键码映射
    // 这样可以避免 ctfmon.exe 错误和输入法干扰
    println!("⌨️ Using Unicode input for all characters to bypass IME...");
    println!("✅ Text to type: \"{}\"", text);

    for ch in text.chars() {
        type_unicode_char(ch);
    }

    println!("✅ Unicode input completed");
}

/// Windows: 输入Unicode字符（支持中文等）
#[cfg(target_os = "windows")]
fn type_unicode_char(ch: char) {
    unsafe {
        use winapi::um::winuser::{SendInput, INPUT, KEYBDINPUT, INPUT_KEYBOARD, KEYEVENTF_UNICODE, KEYEVENTF_KEYUP};
        use winapi::shared::minwindef::WORD;
        use std::mem;

        let code_point = ch as u32;

        // 按下 - Unicode扫描码
        let mut key_down: INPUT = mem::zeroed();
        key_down.type_ = INPUT_KEYBOARD;
        *key_down.u.ki_mut() = KEYBDINPUT {
            wVk: 0,
            wScan: code_point as WORD,
            dwFlags: KEYEVENTF_UNICODE,
            time: 0,
            dwExtraInfo: 0,
        };

        // 释放
        let mut key_up: INPUT = mem::zeroed();
        key_up.type_ = INPUT_KEYBOARD;
        *key_up.u.ki_mut() = KEYBDINPUT {
            wVk: 0,
            wScan: code_point as WORD,
            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };

        let mut inputs = [key_down, key_up];
        let size = mem::size_of::<INPUT>() as i32;
        let count = inputs.len() as u32;

        SendInput(count, inputs.as_mut_ptr() as *mut INPUT, size);
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
}

// Fallback function: type text directly using xdotool
#[allow(dead_code)]
fn type_text_direct(text: &str, delays: &TypingDelays) -> Result<(), VoiceError> {
    println!("🔧 Direct typing text: \"{}\"", text);

    // For xterm compatibility, set text to BOTH clipboard and primary selection - DISABLED PRIMARY
    println!("🔧 Setting text to clipboard only (PRIMARY selection disabled)...");
    
    // DEBUG: Show current clipboard content before setting
    println!("🔍 DEBUG: Checking current clipboard content...");
    if let Ok(clipboard_content) = get_clipboard_content() {
        println!("📋 Current clipboard content: \"{}\"", clipboard_content);
    } else {
        println!("📋 Current clipboard content: <Failed to read>");
    }
    
    // Set text to standard clipboard (Ctrl+C/Ctrl+V)
    set_clipboard_content(text);
    println!("📋 Text set to standard clipboard");
    
    // DEBUG: Verify clipboard content after setting
    if let Ok(clipboard_content) = get_clipboard_content() {
        println!("📋 Verification - Standard clipboard now contains: \"{}\"", clipboard_content);
        if clipboard_content == text {
            println!("✅ Standard clipboard verification SUCCESS");
        } else {
            println!("❌ Standard clipboard verification FAILED");
        }
    } else {
        println!("❌ Failed to verify standard clipboard content");
    }
    
    // PRIMARY selection code completely disabled
    /*
    println!("🔍 DETAILED PRIMARY DEBUG: Starting PRIMARY selection setup...");
    
    // Step 1: Check if xclip is available
    if let Ok(which_output) = Command::new("which").arg("xclip").output() {
        if which_output.status.success() {
            println!("✅ xclip found for PRIMARY selection");
            
            // Step 2: FIRST - Clear PRIMARY selection completely
            println!("🧹 CLEARING PRIMARY selection before testing...");
            if let Ok(_clear_result) = Command::new("echo").arg("-n").arg("").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn() {
                // This creates an empty string to clear PRIMARY
                if let Ok(mut clear_child) = Command::new("xclip")
                    .args(&["-selection", "primary"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(stdin) = clear_child.stdin.as_mut() {
                        if let Ok(_) = stdin.write_all(b"") {
                            let _ = clear_child.wait();
                            println!("✅ PRIMARY selection cleared");
                        }
                    }
                }
            }
            
            // Step 3: Wait a moment for clearing to take effect
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // Step 4: Immediately check current PRIMARY selection content
            println!("🔍 Checking current PRIMARY selection content...");
            if let Ok(current_primary) = Command::new("xclip")
                .args(&["-selection", "primary", "-o"])
                .output()
            {
                if current_primary.status.success() {
                    let current_text = String::from_utf8_lossy(&current_primary.stdout);
                    let trimmed_text = current_text.trim_end_matches('\n');
                    if !trimmed_text.is_empty() {
                        println!("📋 CURRENT PRIMARY SELECTION: \"{}\"", trimmed_text);
                        println!("📏 Length: {} characters", trimmed_text.len());
                    } else {
                        println!("📋 CURRENT PRIMARY SELECTION: <empty>");
                    }
                } else {
                    println!("❌ Failed to read PRIMARY selection: {}", String::from_utf8_lossy(&current_primary.stderr));
                }
            } else {
                println!("❌ Failed to execute xclip -selection primary -o command");
            }
            
            // Step 6: Check current PRIMARY selection content BEFORE our setting
            if let Ok(current_primary) = Command::new("xclip")
                .args(&["-selection", "primary", "-o"])
                .output()
            {
                let current_text = String::from_utf8_lossy(&current_primary.stdout);
                println!("📋 PRIMARY content BEFORE setting our mock text: \"{}\"", current_text.trim_end_matches('\n'));
            }
            
            // Step 3: Set new content to PRIMARY selection
            println!("🔧 Setting PRIMARY selection with text: \"{}\"", text);
            if let Ok(mut child) = Command::new("xclip")
                .args(&["-selection", "primary"])
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(stdin) = child.stdin.as_mut() {
                    let bytes_written = match stdin.write_all(text.as_bytes()) {
                        Ok(_) => {
                            println!("✅ Bytes written to xclip stdin: {} bytes", text.as_bytes().len());
                            "SUCCESS"
                        }
                        Err(e) => {
                            println!("❌ Failed to write to xclip stdin: {}", e);
                            "FAILED"
                        }
                    };
                    
                    // Step 4: Wait for xclip to complete
                    println!("⏳ Waiting for xclip process to complete...");
                    match child.wait() {
                        Ok(status) => {
                            if status.success() {
                                println!("✅ xclip process completed successfully");
                            } else {
                                println!("❌ xclip process failed with status: {}", status);
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to wait for xclip: {}", e);
                        }
                    }
                    
                    if bytes_written == "SUCCESS" {
                        println!("📋 Text set to PRIMARY selection for middle-click paste");
                        
                        // Step 5: Wait for X11 synchronization
                        println!("⏱️ Waiting 500ms for X11 PRIMARY selection synchronization...");
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        
                        // Step 6: Verify PRIMARY selection content AFTER setting
                        println!("🔍 Verifying PRIMARY selection content AFTER setting...");
                        if let Ok(primary_output) = Command::new("xclip")
                            .args(&["-selection", "primary", "-o"])
                            .output()
                        {
                            if primary_output.status.success() {
                                let primary_stdout = String::from_utf8_lossy(&primary_output.stdout);
                                let primary_text = primary_stdout.trim_end_matches('\n');
                                println!("📋 VERIFIED - PRIMARY selection now contains: \"{}\"", primary_text);
                                println!("📏 Length: {} characters", primary_text.len());
                                
                                if primary_text == text {
                                    println!("✅ PRIMARY selection verification COMPLETE SUCCESS");
                                } else {
                                    println!("❌ PRIMARY selection verification FAILED - Content mismatch");
                                    println!("📋 Expected: \"{}\"", text);
                                    println!("📋 Got:      \"{}\"", primary_text);
                                    
                                    // Show character-by-character comparison
                                    println!("🔍 Character comparison:");
                                    let expected_chars: Vec<char> = text.chars().collect();
                                    let actual_chars: Vec<char> = primary_text.chars().collect();
                                    for (i, (exp, act)) in expected_chars.iter().zip(actual_chars.iter()).enumerate() {
                                        if exp == act {
                                            println!("  [{}] '{}' = '{}' ✅", i, exp, act);
                                        } else {
                                            println!("  [{}] '{}' = '{}' ❌", i, exp, act);
                                        }
                                    }
                                }
                            } else {
                                println!("❌ xclip -o command failed: {:?}", String::from_utf8_lossy(&primary_output.stderr));
                            }
                        } else {
                            println!("❌ Failed to execute xclip -o command");
                        }
                    }
                } else {
                    println!("❌ Failed to get stdin handle for xclip");
                }
            } else {
                println!("❌ Failed to spawn xclip process for PRIMARY selection");
            }
        } else {
            println!("❌ xclip not found: {}", String::from_utf8_lossy(&which_output.stderr));
        }
    }
    
    // Also try to set primary selection using xclip if available
    if let Ok(_) = Command::new("xclip")
        .args(&["-selection", "primary"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        // This is a fallback, continue with other methods
    }
    */
    
    // Wait for clipboard to update
    std::thread::sleep(std::time::Duration::from_millis(delays.clipboard_update_ms as u64));
    
    // Use xdotool type command for direct text input
    println!("🔧 Using xdotool type command for direct text input...");

    if let Ok(_) = Command::new("which").arg("xdotool").output() {
        println!("✅ xdotool found for direct typing");

        // Add delay to ensure keyboard events are fully processed
        println!("⏱️ Waiting {}ms for keyboard events to settle...", delays.keyboard_events_settle_ms);
        std::thread::sleep(std::time::Duration::from_millis(delays.keyboard_events_settle_ms as u64));

        // Use xdotool type to input text directly with slower typing speed for Chinese characters
        match Command::new("xdotool")
            .args(&["type", "--delay", &delays.character_interval_ms.to_string(), text])  // delay between characters for better Chinese input
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    println!("✅ Direct text input successful via xdotool");
                    println!("📝 Text typed: \"{}\"", text);
                } else {
                    println!("❌ xdotool type command failed: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                println!("❌ Failed to execute xdotool type: {}", e);
            }
        }

        // Add a small delay to ensure typing completes
        std::thread::sleep(std::time::Duration::from_millis(delays.typing_complete_ms as u64));

    } else {
        println!("❌ xdotool not found, cannot use direct typing");
    }

    println!("🔧 Text input complete");
    return Ok(());
}

#[allow(dead_code)]
fn simulate_backspace() {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to key code 51 using command down")
            .output();
    }

    #[cfg(target_os = "windows")]
    {
        // 🔥 使用PowerShell的SendKeys模拟退格键（不会被rdev捕获）
        use std::process::Command;
        let _ = Command::new("powershell")
            .arg("-WindowStyle")
            .arg("Hidden")
            .arg("-Command")
            .arg("$wshell = New-Object -ComObject wscript.shell; $wshell.SendKeys('{BACKSPACE}')")
            .output();
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(_) = Command::new("xdotool").arg("key").arg("BackSpace").output() {
            // Success
        }
    }
}

#[allow(dead_code)]
fn simulate_paste(_text: &str) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to keystroke \"v\" using command down")
            .output();
    }

    #[cfg(target_os = "windows")]
    {
        println!("🔧 simulate_paste: Starting Windows paste implementation");

        // Detect if foreground window is a terminal
        let is_terminal = unsafe {
            use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW};

            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                false
            } else {
                let mut buffer = [16u16; 512];
                let len = GetWindowTextW(hwnd, buffer.as_mut_ptr(), 512);
                if len > 0 {
                    let title = String::from_utf16_lossy(&buffer[..len as usize]).to_lowercase();
                    println!("🔧 simulate_paste: Foreground window title: \"{}\"", title);
                    // Check for common terminal names
                    title.contains("windows terminal")
                        || title.contains("powershell")
                        || title.contains("command prompt")
                        || title.contains("cmd")
                        || title.contains("ubuntu")
                        || title.contains("wsl")
                        || title.contains("terminal")
                        || title.contains("console")
                } else {
                    false
                }
            }
        };

        if is_terminal {
            println!("🔧 simulate_paste: Terminal detected, using Ctrl+Shift+V");
            // Try Ctrl+Shift+V for terminals (Windows Terminal, some modern terminals)
            unsafe {
                use winapi::um::winuser::{SendInput, INPUT, KEYBDINPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_CONTROL, VK_SHIFT};
                use std::mem;

                const VK_V: u16 = 0x56;

                // Press Ctrl
                let mut ctrl_down: INPUT = mem::zeroed();
                ctrl_down.type_ = INPUT_KEYBOARD;
                *ctrl_down.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_CONTROL as u16,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                };

                // Press Shift
                let mut shift_down: INPUT = mem::zeroed();
                shift_down.type_ = INPUT_KEYBOARD;
                *shift_down.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_SHIFT as u16,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                };

                // Press V
                let mut v_down: INPUT = mem::zeroed();
                v_down.type_ = INPUT_KEYBOARD;
                *v_down.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_V,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                };

                // Release V
                let mut v_up: INPUT = mem::zeroed();
                v_up.type_ = INPUT_KEYBOARD;
                *v_up.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_V,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };

                // Release Shift
                let mut shift_up: INPUT = mem::zeroed();
                shift_up.type_ = INPUT_KEYBOARD;
                *shift_up.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_SHIFT as u16,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };

                // Release Ctrl
                let mut ctrl_up: INPUT = mem::zeroed();
                ctrl_up.type_ = INPUT_KEYBOARD;
                *ctrl_up.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_CONTROL as u16,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };

                let mut inputs = [ctrl_down, shift_down, v_down, v_up, shift_up, ctrl_up];
                let size = mem::size_of::<INPUT>() as i32;
                let count = inputs.len() as u32;

                println!("🔧 simulate_paste: Sending Ctrl+Shift+V ({}) inputs", count);
                let result = SendInput(count, inputs.as_mut_ptr(), size);
                println!("🔧 simulate_paste: SendInput returned {} (expected {})", result, count);
            }
        } else {
            println!("🔧 simulate_paste: Non-terminal detected, using Ctrl+V");
            // Standard Ctrl+V for GUI applications
            unsafe {
                use winapi::um::winuser::{SendInput, INPUT, KEYBDINPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_CONTROL};
                use std::mem;

                const VK_V: u16 = 0x56;

                // Press Ctrl
                let mut ctrl_down: INPUT = mem::zeroed();
                ctrl_down.type_ = INPUT_KEYBOARD;
                *ctrl_down.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_CONTROL as u16,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                };

                // Press V
                let mut v_down: INPUT = mem::zeroed();
                v_down.type_ = INPUT_KEYBOARD;
                *v_down.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_V,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                };

                // Release V
                let mut v_up: INPUT = mem::zeroed();
                v_up.type_ = INPUT_KEYBOARD;
                *v_up.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_V,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };

                // Release Ctrl
                let mut ctrl_up: INPUT = mem::zeroed();
                ctrl_up.type_ = INPUT_KEYBOARD;
                *ctrl_up.u.ki_mut() = KEYBDINPUT {
                    wVk: VK_CONTROL as u16,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };

                let mut inputs = [ctrl_down, v_down, v_up, ctrl_up];
                let size = mem::size_of::<INPUT>() as i32;
                let count = inputs.len() as u32;

                println!("🔧 simulate_paste: Sending Ctrl+V ({} inputs)", count);
                let result = SendInput(count, inputs.as_mut_ptr(), size);
                println!("🔧 simulate_paste: SendInput returned {} (expected {})", result, count);
            }
        }

        // Wait for paste to complete
        println!("🔧 simulate_paste: Waiting for paste to complete");
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!("🔧 simulate_paste: Paste operation completed");
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(_) = Command::new("xdotool").arg("key").arg("ctrl+v").output() {
            // Success
        }
    }
}

fn get_clipboard_content() -> Result<String, VoiceError> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("pbpaste").output()
            .map_err(|e| VoiceError::Other(format!("Failed to get clipboard: {}", e)))?;

        Ok(String::from_utf8(output.stdout)?)
    }

    #[cfg(target_os = "windows")]
    {
        // Use Windows API directly instead of PowerShell
        unsafe {
            use winapi::um::winuser::{OpenClipboard, CloseClipboard, GetClipboardData, CF_UNICODETEXT};
            use winapi::um::winnt::WCHAR;

            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return Err(VoiceError::Other("Failed to open clipboard".to_string()));
            }

            let clipboard_data = GetClipboardData(CF_UNICODETEXT);
            if clipboard_data.is_null() {
                CloseClipboard();
                return Err(VoiceError::Other("Failed to get clipboard data".to_string()));
            }

            let text_ptr = clipboard_data as *const WCHAR;
            let mut len = 0;
            while *text_ptr.offset(len) != 0 {
                len += 1;
            }

            let slice = std::slice::from_raw_parts(text_ptr, len as usize);
            let text = String::from_utf16(slice)
                .map_err(|e| VoiceError::Other(format!("Failed to parse clipboard text: {}", e)))?;

            CloseClipboard();
            Ok(text)
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("xclip").arg("-selection").arg("clipboard").arg("-o").output() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(VoiceError::Other("xclip not found".to_string()))
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(VoiceError::Other("Platform not supported".to_string()))
    }
}

fn set_clipboard_content(text: &str) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("pbcopy")
            .write_all(text.as_bytes());
    }

    #[cfg(target_os = "windows")]
    {
        // Use Windows API directly instead of PowerShell
        unsafe {
            use winapi::um::winuser::{OpenClipboard, CloseClipboard, EmptyClipboard, SetClipboardData, CF_UNICODETEXT};
            use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
            use winapi::um::winnt::WCHAR;

            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return;
            }

            EmptyClipboard();

            // Convert string to UTF-16
            let utf16_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let byte_len = utf16_text.len() * std::mem::size_of::<WCHAR>();

            // Allocate global memory
            let handle = GlobalAlloc(GMEM_MOVEABLE, byte_len);
            if handle.is_null() {
                CloseClipboard();
                return;
            }

            // Lock and copy data
            let ptr = GlobalLock(handle);
            if ptr.is_null() {
                GlobalUnlock(handle);
                CloseClipboard();
                return;
            }

            std::ptr::copy_nonoverlapping(utf16_text.as_ptr(), ptr as *mut WCHAR, utf16_text.len());
            GlobalUnlock(handle);

            // Set clipboard data
            SetClipboardData(CF_UNICODETEXT, handle);
            CloseClipboard();
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        // Try multiple clipboard methods
        let mut success = false;
        
        // Method 1: Try xclip (most common)
        if let Ok(output) = Command::new("which").arg("xclip").output() {
            if output.status.success() {
                if let Ok(mut child) = Command::new("xclip")
                    .args(&["-selection", "clipboard"])
                    .stdin(std::process::Stdio::piped())
                    .spawn() 
                {
                    if let Some(stdin) = child.stdin.as_mut() {
                        if let Ok(_) = stdin.write_all(text.as_bytes()) {
                            let _ = child.wait();
                            success = true;
                            println!("✅ Text set to clipboard via xclip");
                        }
                    }
                }
            }
        }
        
        // Method 2: Try xsel if xclip fails
        if !success {
            if let Ok(output) = Command::new("which").arg("xsel").output() {
                if output.status.success() {
                    if let Ok(mut child) = Command::new("xsel")
                        .args(&["--clipboard", "--input"])
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                    {
                        if let Some(stdin) = child.stdin.as_mut() {
                            if let Ok(_) = stdin.write_all(text.as_bytes()) {
                                let _ = child.wait();
                                success = true;
                                println!("✅ Text set to clipboard via xsel");
                            }
                        }
                    }
                }
            }
        }
        
        // Method 3: Try wl-copy (Wayland)
        if !success {
            if let Ok(output) = Command::new("which").arg("wl-copy").output() {
                if output.status.success() {
                    if let Ok(_) = Command::new("wl-copy")
                        .arg(text)
                        .output()
                    {
                        success = true;
                        println!("✅ Text set to clipboard via wl-copy");
                    }
                }
            }
        }
        
        if !success {
            eprintln!("❌ Warning: No clipboard utility found (xclip, xsel, wl-copy)");
            eprintln!("💡 Install one of: sudo apt install xclip");
            eprintln!("📝 Falling back to echo command for basic output");

            // As a last resort, just print to stdout so user can see it
            println!("📋 Text to copy manually: {}", text);
        }
    }
}

impl KeyboardManager {
    // ========== Streaming support methods ==========

    /// 启动流式会话（静态版本，用于回调）
    fn start_streaming_session_internal(
        mode: crate::voice_assistant::Mode,
        asr_processor: &Arc<dyn AsrProcessor + Send + Sync>,
    ) -> Result<Box<dyn crate::voice_assistant::StreamingAsrSession>, VoiceError> {
        match asr_processor.start_streaming_session(mode) {
            Ok(session) => {
                println!("✅ Streaming session started");
                Ok(session)
            }
            Err(e) => {
                eprintln!("❌ Failed to start streaming session: {}", e);
                Err(e)
            }
        }
    }

    /// 结束流式会话（静态版本，用于回调）
    fn finalize_streaming_session_internal(
        streaming_session: &Arc<Mutex<Option<Box<dyn crate::voice_assistant::StreamingAsrSession>>>>,
    ) -> Result<String, VoiceError> {
        if let Some(mut session) = streaming_session.lock().unwrap().take() {
            let final_text = session.finalize()?;
            println!("✅ Streaming session finalized: \"{}\"", final_text);
            Ok(final_text)
        } else {
            Ok(String::new())
        }
    }

    /// 启动流式处理线程
    fn start_streaming_processor_thread(
        recorder: &Arc<Mutex<Option<crate::voice_assistant::AudioRecorder>>>,
        _asr_processor: &Arc<dyn AsrProcessor + Send + Sync>,
        streaming_session: &Arc<Mutex<Option<Box<dyn crate::voice_assistant::StreamingAsrSession>>>>,
        streaming_enabled: &Arc<Mutex<bool>>,
        streaming_chunk_interval_ms: &Arc<Mutex<u64>>,
        streaming_stop_signal: &Arc<Mutex<bool>>,
        streaming_thread_handle: &Arc<Mutex<Option<JoinHandle<()>>>>,
    ) {
        // 检查是否已启用流式模式
        if !*streaming_enabled.lock().unwrap() {
            println!("⚠️ Streaming mode is disabled, skipping streaming processor");
            return;
        }

        let recorder_clone = recorder.clone();
        let streaming_session_clone = streaming_session.clone();
        let streaming_enabled_clone = streaming_enabled.clone();
        let streaming_stop_signal_clone = streaming_stop_signal.clone();
        let streaming_chunk_interval_ms_clone = streaming_chunk_interval_ms.clone();

        // 启动流式处理线程
        let handle = thread::spawn(move || {
            println!("🔄 Streaming processor thread started");

            // 🔥 优化：跟踪上次处理到的位置，只获取新音频
            let mut last_processed_length = 0usize;

            loop {
                // 检查停止信号
                if *streaming_stop_signal_clone.lock().unwrap() {
                    println!("🛑 Streaming processor thread received stop signal");
                    break;
                }

                // 检查流式模式是否仍然启用
                if !*streaming_enabled_clone.lock().unwrap() {
                    break;
                }

                // 检查是否有流式会话
                let has_session = streaming_session_clone.lock().unwrap().is_some();
                if !has_session {
                    break;
                }

                // 等待处理间隔
                let interval_ms = *streaming_chunk_interval_ms_clone.lock().unwrap();
                thread::sleep(Duration::from_millis(interval_ms));

                // 获取录音数据并处理
                if let Some(ref rec) = *recorder_clone.lock().unwrap() {
                    let all_audio_samples = rec.get_audio_data();
                    let sample_rate = rec.get_sample_rate();

                    if all_audio_samples.is_empty() {
                        continue;
                    }

                    // 🔥 优化：只获取新增加的音频样本
                    let current_length = all_audio_samples.len();
                    if current_length <= last_processed_length {
                        // 没有新音频，跳过
                        continue;
                    }

                    let new_audio_samples = &all_audio_samples[last_processed_length..];
                    let new_sample_count = new_audio_samples.len();

                    println!("🎵 Streaming: Got {} new audio samples ({}Hz), total: {}",
                        new_sample_count, sample_rate, current_length);

                    // 更新已处理位置
                    last_processed_length = current_length;

                    // 处理音频块
                    if let Some(session) = streaming_session_clone.lock().unwrap().as_mut() {
                        match session.process_audio_chunk(new_audio_samples, sample_rate) {
                            Ok(segments) => {
                                println!("🎯 Streaming: Got {} segments", segments.len());
                                for segment in segments {
                                    if segment.is_final && segment.should_type {
                                        println!("🎯 Streaming result: \"{}\"", segment.text);
                                        // 🔥 实时输入流式结果到目标窗口
                                        type_text_incremental(&segment.text);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ Streaming processing error: {}", e);
                            }
                        }
                    }
                }
            }

            println!("✅ Streaming processor thread stopped");
        });

        *streaming_thread_handle.lock().unwrap() = Some(handle);
    }

    /// 设置是否启用流式模式
    pub fn set_streaming_enabled(&self, enabled: bool) {
        *self.streaming_enabled.lock().unwrap() = enabled;
        println!("🔄 Streaming mode: {}", if enabled { "ENABLED" } else { "DISABLED" });
    }

    /// 设置流式处理间隔（毫秒）
    pub fn set_streaming_chunk_interval(&self, interval_ms: u64) {
        *self.streaming_chunk_interval_ms.lock().unwrap() = interval_ms;
    }
}

/// 增量打字（追加，不删除现有内容）
pub fn type_text_incremental(text: &str) {
        println!("🎯 Streaming text: \"{}\"", text);

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            use std::thread;
            use std::time::Duration;

            // 方法1: 使用 xdotool type（直接输入，无需剪贴板）
            if let Ok(_) = Command::new("which").arg("xdotool").output() {
                if let Ok(_) = Command::new("xdotool")
                    .arg("type")
                    .arg(text)
                    .output()
                {
                    println!("✅ Text typed via xdotool type");
                    return;
                }
            }

            // 方法2: 使用 ydotool (Wayland)
            if let Ok(_) = Command::new("which").arg("ydotool").output() {
                if let Ok(_) = Command::new("ydotool")
                    .arg("type")
                    .arg(text)
                    .output()
                {
                    println!("✅ Text typed via ydotool");
                    return;
                }
            }

            // 方法3: 回退到剪贴板
            println!("⚠️ Falling back to clipboard method");
            set_clipboard_content(text);
            thread::sleep(Duration::from_millis(100));
            simulate_ctrl_v();
        }

        #[cfg(target_os = "windows")]
        {
            // 🔥 Windows: 使用逐字符Unicode输入（与正常转录相同的方式）
            println!("⌨️ Windows streaming typing using Unicode input...");
            for ch in text.chars() {
                type_unicode_char(ch);
            }
            println!("✅ Streaming text typed successfully");
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: 使用 osascript
            // TODO: 实现 macOS 平台的增量打字
            println!("⚠️ macOS streaming typing not yet implemented");
        }
}

/// 占位符 ASR 处理器，用于释放实际处理器时使用
struct DefaultAsrProcessor;

// 实现Send和Sync，因为ASR处理器需要在线程间共享
unsafe impl Send for DefaultAsrProcessor {}
unsafe impl Sync for DefaultAsrProcessor {}

impl AsrProcessor for DefaultAsrProcessor {
    fn process_audio(
        &self,
        _audio_buffer: std::io::Cursor<Vec<u8>>,
        _mode: crate::voice_assistant::Mode,
        _prompt: &str,
    ) -> Result<String, VoiceError> {
        Err(VoiceError::Other("ASR processor not available".to_string()))
    }

    fn get_processor_type(&self) -> Option<&str> {
        Some("default-placeholder")
    }
}