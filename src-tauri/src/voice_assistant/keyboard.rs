use rdev::{listen, EventType, Key};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::io::Write;
use crate::voice_assistant::{KeyboardManagerTrait, AsrProcessor, TranslateProcessor, InputState, Mode, VoiceError};
use std::process::Command;

pub struct KeyboardManager {
    state: Arc<Mutex<InputState>>,
    asr_processor: Arc<dyn AsrProcessor + Send + Sync>,
    translate_processor: Option<Arc<dyn TranslateProcessor + Send + Sync>>,
    option_pressed: Arc<Mutex<bool>>,
    shift_pressed: Arc<Mutex<bool>>,
    option_press_time: Arc<Mutex<Option<Instant>>>,
    temp_text_length: Arc<Mutex<usize>>,
    original_clipboard: Arc<Mutex<Option<String>>>,
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
            option_pressed: Arc::new(Mutex::new(false)),
            shift_pressed: Arc::new(Mutex::new(false)),
            option_press_time: Arc::new(Mutex::new(None)),
            temp_text_length: Arc::new(Mutex::new(0)),
            original_clipboard: Arc::new(Mutex::new(None)),
        })
    }

    pub fn start_listening(&mut self) {
        let state = self.state.clone();
        let asr_processor = self.asr_processor.clone();
        let translate_processor = self.translate_processor.clone();
        let option_pressed = self.option_pressed.clone();
        let shift_pressed = self.shift_pressed.clone();
        let option_press_time = self.option_press_time.clone();
        let temp_text_length = self.temp_text_length.clone();
        let original_clipboard = self.original_clipboard.clone();

        tokio::spawn(async move {
            let mut recorder = None;
            let mut last_state = InputState::Idle;

            if let Err(e) = listen(move |event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        if key == Key::Alt {
                            let mut opt = option_pressed.lock().unwrap();
                            let mut opt_time = option_press_time.lock().unwrap();
                            *opt = true;
                            *opt_time = Some(Instant::now());

                            // 保存原始剪贴板
                            let mut clipboard = original_clipboard.lock().unwrap();
                            if clipboard.is_none() {
                                if let Ok(content) = get_clipboard_content() {
                                    *clipboard = Some(content);
                                }
                            }
                        } else if key == Key::ShiftLeft || key == Key::ShiftRight {
                            *shift_pressed.lock().unwrap() = true;
                        }
                    }
                    EventType::KeyRelease(key) => {
                        if key == Key::Alt {
                            let opt = *option_pressed.lock().unwrap();
                            let _shift = *shift_pressed.lock().unwrap();
                            let press_time_opt = *option_press_time.lock().unwrap();

                            if opt {
                                *option_pressed.lock().unwrap() = false;
                                *option_press_time.lock().unwrap() = None;

                                if let Some(press_time) = press_time_opt {
                                    let elapsed = press_time.elapsed();
                                    let current_state = *state.lock().unwrap();

                                    if elapsed > Duration::from_millis(300) {
                                        // 触发录音结束
                                        if current_state == InputState::Recording {
                                            *state.lock().unwrap() = InputState::Processing;
                                        } else if current_state == InputState::RecordingTranslate {
                                            *state.lock().unwrap() = InputState::Translating;
                                        }
                                    } else {
                                        // 按键太短，重置状态
                                        if current_state.is_recording() {
                                            *state.lock().unwrap() = InputState::Idle;
                                        }
                                    }
                                }
                            }
                        } else if key == Key::ShiftLeft || key == Key::ShiftRight {
                            *shift_pressed.lock().unwrap() = false;
                        }
                    }
                    _ => {}
                }

                // 状态变化检测
                let current_state = *state.lock().unwrap();
                if current_state != last_state {
                    last_state = current_state;

                    match current_state {
                        InputState::Recording => {
                            // 开始转录录音
                            Self::start_recording_internal(&mut recorder);
                        }
                        InputState::RecordingTranslate => {
                            // 开始翻译录音
                            Self::start_recording_internal(&mut recorder);
                        }
                        InputState::Processing => {
                            // 停止录音并处理转录
                            if let Some(ref mut rec) = recorder {
                                match rec.stop_recording_with_option(true) { // TODO: Get this from config
                                    Ok(audio_file_path) => {
                                        let asr_clone = asr_processor.clone();
                                        let state_clone = state.clone();
                                        let temp_len_clone = temp_text_length.clone();
                                        let clipboard_clone = original_clipboard.clone();

                                        tokio::spawn(async move {
                                            // 使用 ASR 处理器的 process_audio_file 方法
                                            match asr_clone.process_audio_file(&audio_file_path, Mode::Transcriptions, "") {
                                                Ok(text) => {
                                                    Self::type_text_internal(&state_clone, &temp_len_clone, &clipboard_clone, &text, None);
                                                }
                                                Err(e) => {
                                                    Self::type_text_internal(&state_clone, &temp_len_clone, &clipboard_clone, "", Some(&e.to_string()));
                                                }
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        Self::type_text_internal(&state, &temp_text_length, &original_clipboard, "", Some(&e.to_string()));
                                    }
                                }
                                recorder = None;
                            }
                        }
                        InputState::Translating => {
                            // 停止录音并处理翻译
                            if let Some(ref mut rec) = recorder {
                                if let Some(ref translate_proc) = translate_processor {
                                    match rec.stop_recording_with_option(true) { // TODO: Get this from config
                                        Ok(audio_file_path) => {
                                            let asr_clone = asr_processor.clone();
                                            let translate_clone = translate_proc.clone();
                                            let state_clone = state.clone();
                                            let temp_len_clone = temp_text_length.clone();
                                            let clipboard_clone = original_clipboard.clone();

                                            tokio::spawn(async move {
                                                // 先使用 ASR 转录
                                                match asr_clone.process_audio_file(&audio_file_path, Mode::Transcriptions, "") {
                                                    Ok(transcribed) => {
                                                        // 再翻译
                                                        match translate_clone.translate(&transcribed) {
                                                            Ok(translated) => {
                                                                Self::type_text_internal(&state_clone, &temp_len_clone, &clipboard_clone, &translated, None);
                                                            }
                                                            Err(e) => {
                                                                Self::type_text_internal(&state_clone, &temp_len_clone, &clipboard_clone, "", Some(&e.to_string()));
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        Self::type_text_internal(&state_clone, &temp_len_clone, &clipboard_clone, "", Some(&e.to_string()));
                                                    }
                                                }
                                            });
                                        }
                                        Err(e) => {
                                            Self::type_text_internal(&state, &temp_text_length, &original_clipboard, "", Some(&e.to_string()));
                                        }
                                    }
                                }
                            }
                            recorder = None;
                        }
                        _ => {}
                    }
                }

                // 持续检查 Option 按键时间
                if *option_pressed.lock().unwrap() {
                    if let Some(press_time) = *option_press_time.lock().unwrap() {
                        if press_time.elapsed() > Duration::from_millis(300) {
                            let shift = *shift_pressed.lock().unwrap();
                            let current_state = *state.lock().unwrap();

                            if current_state.can_start_recording() {
                                *state.lock().unwrap() = if shift {
                                    InputState::RecordingTranslate
                                } else {
                                    InputState::Recording
                                };
                            }
                        }
                    }
                }

            }) {
                eprintln!("Error listening for keyboard events: {:?}", e);
            }
        });
    }

    fn start_recording_internal(recorder: &mut Option<crate::voice_assistant::AudioRecorder>) {
        if recorder.is_none() {
            match crate::voice_assistant::AudioRecorder::new() {
                Ok(mut r) => {
                    if let Err(e) = r.start_recording() {
                        eprintln!("Failed to start recording: {}", e);
                    } else {
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
    ) {
        // 删除之前的临时文本
        let len = *temp_text_length.lock().unwrap();
        for _ in 0..len {
            simulate_backspace();
        }
        *temp_text_length.lock().unwrap() = 0;

        if let Some(err_msg) = error {
            // 显示错误消息
            simulate_typing(&format!("❌ {}", err_msg));
            *temp_text_length.lock().unwrap() = 2 + err_msg.len();

            // 2秒后清除错误消息
            let state_clone = state.clone();
            let temp_len_clone = temp_text_length.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(2)).await;
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
            simulate_typing(text);

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
        *self.option_pressed.lock().unwrap() = false;
        *self.shift_pressed.lock().unwrap() = false;
        *self.option_press_time.lock().unwrap() = None;

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

fn simulate_typing(text: &str) {
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
        // Linux 实现可以使用 xdotool
        if let Ok(output) = Command::new("xdotool").arg("type").arg(text).output() {
            if !output.status.success() {
                eprintln!("Failed to type text: {:?}", String::from_utf8_lossy(&output.stderr));
            }
        } else {
            eprintln!("xdotool not found");
        }
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
        if let Ok(mut child) = Command::new("xclip").arg("-selection").arg("clipboard").stdin(std::process::Stdio::piped()).spawn() {
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }
}