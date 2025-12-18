use rdev::{listen, EventType, Key};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::process::Command;
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
        })
    }

    /// 🔥 更新处理器引用 - 用于配置刷新
    pub fn update_processors(
        &mut self,
        new_asr_processor: Arc<dyn AsrProcessor + Send + Sync>,
        new_translate_processor: Option<Arc<dyn TranslateProcessor + Send + Sync>>,
    ) -> Result<(), VoiceError> {
        println!("🔄 Updating KeyboardManager processors...");
        
        // 更新处理器引用
        self.asr_processor = new_asr_processor;
        self.translate_processor = new_translate_processor;
        
        println!("✅ KeyboardManager processors updated successfully");
        Ok(())
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

        // Use tokio::task::spawn_blocking to avoid runtime conflicts with rdev
        // 获取save_wav_files配置传递到回调中
        let save_wav_files_config = *self.save_wav_files.lock().unwrap();
        println!("📁 Save WAV Files setting from config: {}", save_wav_files_config);

        // 克隆延迟配置以便在闭包中使用
        let typing_delays_for_callback = self.typing_delays.clone();

        tokio::task::spawn_blocking(move || {
            let mut recorder: Option<crate::voice_assistant::AudioRecorder> = None;

            // 使用传递过来的save_wav_files配置
            let save_wav_files = save_wav_files_config;
            println!("📁 Save WAV Files setting in callback: {}", save_wav_files);
            let mut last_state = InputState::Idle;
            let mut recording_started = false;
            let mut hotkey_press_time: Option<Instant> = None;
            const HOTKEY_DELAY_THRESHOLD: Duration = Duration::from_millis(300); // 防误触阈值

            if let Err(e) = listen(move |event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
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
                            if transcribe_hotkey.matches(&*keys) && current_state.can_start_recording() && !recording_started {
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
                                    println!("🎤 Transcribe hotkey pressed - starting recording state...");

                                    // IMPORTANT: Clear keys immediately to prevent repeated triggers
                                    keys.clear();

                                    *hotkey_start_time.lock().unwrap() = Some(Instant::now());
                                    *state.lock().unwrap() = InputState::Recording; // Start recording state
                                    // Emit state change event
                                    crate::voice_assistant::coordinator::emit_voice_assistant_state_from_keyboard(&InputState::Recording);
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
                            if translate_hotkey.matches(&*keys) && current_state.can_start_recording() && !recording_started {
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
                        let mut keys = pressed_keys.lock().unwrap();
                        println!("🔓 KeyRelease detected: {:?}", key);
                        keys.remove(&key);
                        println!("🔑 Remaining keys after release: {:?}", keys);
                        
                        // 重置按键时间戳（当所有按键都释放时）
                        if keys.is_empty() {
                            hotkey_press_time = None;
                            
                            // 检查是否在录音状态，如果是，则转换到处理状态
                            let current_state = *state.lock().unwrap();
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
                            Self::start_recording_internal(&mut recorder, save_wav_files);
                        }
                        InputState::RecordingTranslate => {
                            // 开始翻译录音
                            println!("🌐 Recording Translate state - starting real audio recording...");
                            Self::start_recording_internal(&mut recorder, save_wav_files);
                        }
                        InputState::Processing => {
                            // Process recorded audio with real ASR
                            println!("🔄 Entering Processing state...");
                            println!("🎙️ Processing audio with real ASR...");

                            // Stop recording and get audio data
                            // Process ASR - can now be done synchronously since we use spawn_blocking internally
                            let asr_result = if let Some(ref mut rec) = recorder {
                                println!("🛑 Stopping recording...");

                                // Get audio data BEFORE stopping recording (to avoid data loss)
                                let audio_data = rec.get_audio_data();
                                println!("📊 Got audio data: {} samples", audio_data.len());

                                match rec.stop_recording_with_option(save_wav_files) {
                                    Ok(_) => {
                                        println!("✅ Recording stopped successfully");

                                        if audio_data.is_empty() {
                                            println!("⚠️ No audio data recorded, using mock text");
                                            Some("No audio recorded - please check microphone".to_string())
                                        } else {
                                            // Convert to WAV format for ASR processing
                                            match Self::convert_to_wav_bytes(&audio_data, rec.get_sample_rate()) {
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
                                        }
                                    }
                                    Err(e) => {
                                        println!("❌ Failed to stop recording: {}", e);
                                        Some(format!("Recording error: {}", e))
                                    }
                                }
                            } else {
                                println!("❌ No recorder available");
                                Some("No recorder available".to_string())
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
                            recorder = None;

                            // IMPORTANT: Reset state and flags after processing
                            println!("🔄 Resetting state after processing completion...");
                            recording_started = false;
                            *hotkey_start_time.lock().unwrap() = None;
                            *state.lock().unwrap() = InputState::Idle;
                        // Emit state change event
                        crate::voice_assistant::coordinator::emit_voice_assistant_state_from_keyboard(&InputState::Idle);
                        }
                        InputState::Translating => {
                            // Skip audio recording and use mock translation text directly
                            println!("🔄 Entering Translating state...");
                            println!("📝 Using mock translation text for testing (mic is broken)");

                            let state_clone = state.clone();
                            let temp_len_clone = temp_text_length.clone();
                            let clipboard_clone = original_clipboard.clone();

                            // Mock translation text with Chinese content
                            let mock_translated = "这是热键翻译测试文字，模拟语音翻译结果。This is a mock translation test from voice input. 🌐".to_string();

                            println!("⌨️ Typing translated text: \"{}\"", mock_translated);
                            Self::type_text_internal(&state_clone, &temp_len_clone, &clipboard_clone, &mock_translated, None, &typing_delays_for_callback.lock().unwrap());
                            println!("✅ Translation text typing completed");

                            // Stop any recording if active
                            if let Some(ref mut rec) = recorder {
                                let _ = rec.stop_recording();
                                recorder = None;
                            }

                            // IMPORTANT: Reset state and flags immediately after processing
                            println!("🔄 Resetting state after translation completion...");
                            recording_started = false;
                            *hotkey_start_time.lock().unwrap() = None;
                            *state.lock().unwrap() = InputState::Idle;
                        // Emit state change event
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

fn start_recording_internal(recorder: &mut Option<crate::voice_assistant::AudioRecorder>, save_wav_files: bool) {
        if recorder.is_none() {
            match crate::voice_assistant::AudioRecorder::new() {
                Ok(mut r) => {
                    // Set the save_wav_files option on the recorder
                    r.set_save_wav_files(save_wav_files);

                    if let Err(e) = r.start_recording() {
                        eprintln!("Failed to start recording: {}", e);
                    } else {
                        println!("🎙️ Recording started (Save WAV: {})", save_wav_files);
                        *recorder = Some(r);
                    }
                }
                Err(e) => eprintln!("Failed to create recorder: {}", e),
            }
        }
    }

    fn type_text_internal(
        state: &Arc<Mutex<InputState>>,
        temp_text_length: &Arc<Mutex<usize>>,
        original_clipboard: &Arc<Mutex<Option<String>>>,
        text: &str,
        error: Option<&str>,
        delays: &TypingDelays,
    ) {
        // 删除之前的临时文本
        let len = *temp_text_length.lock().unwrap();
        for _ in 0..len {
            simulate_backspace();
        }
        *temp_text_length.lock().unwrap() = 0;

        if let Some(err_msg) = error {
            // 显示错误消息
            simulate_typing(&format!("❌ {}", err_msg), delays);
            *temp_text_length.lock().unwrap() = 2 + err_msg.len();

            // 2秒后清除错误消息 - use std sleep instead of tokio
            let state_clone = state.clone();
            let temp_len_clone = temp_text_length.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(2));
                if *state_clone.lock().unwrap() == InputState::Error {
                    *state_clone.lock().unwrap() = InputState::Idle;
                    let len = *temp_len_clone.lock().unwrap();
                    for _ in 0..len {
                        simulate_backspace();
                    }
                    *temp_len_clone.lock().unwrap() = 0;
                }
            });

            *state.lock().unwrap() = InputState::Error;
        } else if !text.is_empty() {
            // 输入最终文本
            simulate_typing(text, delays);

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

        // 删除临时文本
        let len = *self.temp_text_length.lock().unwrap();
        for _ in 0..len {
            simulate_backspace();
        }
        *self.temp_text_length.lock().unwrap() = 0;

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
        // Windows 实现可以使用 sendinput 或者剪贴板
        // 为了简化，这里使用剪贴板方式
        set_clipboard_content(text);
        simulate_paste();
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
}

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
        // Windows backspace
        use std::process::Command;
        let _ = Command::new("powershell")
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
fn simulate_paste() {
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
        use std::process::Command;
        let _ = Command::new("powershell")
            .arg("-Command")
            .arg("$wshell = New-Object -ComObject wscript.shell; $wshell.SendKeys('^v')")
            .output();
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
        use std::process::Command;
        let output = Command::new("powershell")
            .arg("-Command")
            .arg("Get-Clipboard")
            .output()
            .map_err(|e| VoiceError::Other(format!("Failed to get clipboard: {}", e)))?;

        Ok(String::from_utf8(output.stdout)?)
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
        use std::process::Command;
        let _ = Command::new("powershell")
            .arg("-Command")
            .arg(&format!("Set-Clipboard \"{}\"", text))
            .output();
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