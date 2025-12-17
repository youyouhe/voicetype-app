// Temporary stub for WhisperVadProcessor when whisper-rs is disabled
use crate::voice_assistant::VoiceError;

pub struct WhisperVadProcessor {
    // Disabled for Windows migration
}

pub struct VadSegment {
    pub start: u64,
    pub end: u64,
}

impl WhisperVadProcessor {
    pub fn new(_model_path: &str) -> Result<Self, VoiceError> {
        Err(VoiceError::Other("WhisperVad disabled for Windows migration".to_string()))
    }

    pub fn process(&mut self, _audio_data: &[f32], _sample_rate: u32) -> Result<Vec<VadSegment>, VoiceError> {
        Err(VoiceError::Other("WhisperVad disabled for Windows migration".to_string()))
    }
}