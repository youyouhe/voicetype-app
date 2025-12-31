use std::io::Cursor;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Transcriptions,
    Translations,
}

#[derive(Debug, Error)]
pub enum VoiceError {
    #[error("Audio error: {0}")]
    Audio(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Too short recording")]
    TooShort,
    #[error("Other: {0}")]
    Other(String),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl From<String> for VoiceError {
    fn from(s: String) -> Self {
        VoiceError::Other(s)
    }
}

impl From<&str> for VoiceError {
    fn from(s: &str) -> Self {
        VoiceError::Other(s.to_string())
    }
}

pub trait AsrProcessor {
    fn process_audio(
        &self,
        audio_buffer: Cursor<Vec<u8>>,
        mode: Mode,
        prompt: &str,
    ) -> Result<String, VoiceError>;

    /// 启动流式会话 (用于实时边说边转录)
    fn start_streaming_session(&self, mode: Mode) -> Result<Box<dyn StreamingAsrSession>, VoiceError> {
        Err(VoiceError::Other("Streaming not supported by this processor".to_string()))
    }

    fn get_processor_type(&self) -> Option<&str>;

    /// 显式卸载模型并释放GPU内存
    fn unload(&mut self) {
        // 默认实现：什么都不做
    }
}

/// 流式段落结果
#[derive(Debug, Clone)]
pub struct StreamingSegment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub is_final: bool,     // 此段落是否已最终确定（不会再修改）
    pub should_type: bool,  // 是否应立即打字输出
}

/// 流式 ASR 会话 trait
pub trait StreamingAsrSession: Send {
    /// 处理音频块，返回检测到的语音段落
    fn process_audio_chunk(&mut self, audio_samples: &[f32], sample_rate: u32) -> Result<Vec<StreamingSegment>, VoiceError>;

    /// 结束会话（处理剩余音频）
    fn finalize(&mut self) -> Result<String, VoiceError>;

    /// 获取当前上下文 tokens（用于连续性）
    fn get_context_tokens(&self) -> Vec<i32>;
}

pub trait TranslateProcessor {
    fn translate(&self, text: &str) -> Result<String, VoiceError>;
}

pub trait KeyboardManagerTrait {
    fn start_listening(&mut self);
    fn type_text(&mut self, text: &str, error: Option<&str>);
    fn reset_state(&mut self);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputState {
    Idle,
    Recording,           // F4 按下 - 批处理模式
    RecordingTranslate,  // Shift+F4 按下 - 批处理模式
    Processing,          // ASR 运行中 - 批处理模式
    Translating,         // 翻译运行中 - 批处理模式

    // 流式状态 (Streaming states)
    Streaming,           // F4 按下 - 流式模式（边说边转录）
    StreamingPaused,     // 流式暂停（用户静默时）
    StreamingFinalizing, // 热键释放后的收尾处理

    Error,
    Warning,
}

impl InputState {
    pub fn is_recording(&self) -> bool {
        matches!(self, Self::Recording | Self::RecordingTranslate)
    }
    pub fn can_start_recording(&self) -> bool {
        !self.is_recording()
    }

    // 流式状态检测
    pub fn is_streaming(&self) -> bool {
        matches!(self, Self::Streaming | Self::StreamingPaused | Self::StreamingFinalizing)
    }
    pub fn can_start_streaming(&self) -> bool {
        !self.is_streaming() && !self.is_recording()
    }
}