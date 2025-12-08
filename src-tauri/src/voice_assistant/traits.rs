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
    
    fn get_processor_type(&self) -> Option<&str>;
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
    Recording,
    RecordingTranslate,
    Processing,
    Translating,
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
}