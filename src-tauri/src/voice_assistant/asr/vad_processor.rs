use std::path::Path;
use whisper_rs::{WhisperVadContext, WhisperVadContextParams, WhisperVadParams, WhisperVadSegment};
use crate::voice_assistant::VoiceError;

pub struct WhisperVadProcessor {
    vad_ctx: WhisperVadContext,
}

impl WhisperVadProcessor {
    pub fn new(model_path: &str) -> Result<Self, VoiceError> {
        if !Path::new(model_path).exists() {
            return Err(VoiceError::Other(format!(
                "VAD model file not found: {}",
                model_path
            )));
        }

        let mut vad_ctx_params = WhisperVadContextParams::default();
        vad_ctx_params.set_n_threads(1);
        vad_ctx_params.set_use_gpu(false);

        let vad_ctx = WhisperVadContext::new(model_path, vad_ctx_params)
            .map_err(|e| VoiceError::Other(format!("Failed to load VAD model: {}", e)))?;

        Ok(Self { vad_ctx })
    }

    /// Detect speech segments in audio data
    pub fn detect_speech_segments(&mut self, audio_data: &[f32]) -> Result<Vec<VadSegment>, VoiceError> {
        if audio_data.is_empty() {
            return Ok(Vec::new());
        }

        let vad_params = WhisperVadParams::new();
        let segments = self.vad_ctx
            .segments_from_samples(vad_params, audio_data)
            .map_err(|e| VoiceError::Other(format!("VAD processing failed: {}", e)))?;

        // Convert to our internal format
        let result = segments.into_iter()
            .map(|WhisperVadSegment { start, end }| VadSegment {
                start_ms: (start * 10.0) as u64, // Convert from centiseconds to milliseconds
                end_ms: (end * 10.0) as u64,
            })
            .collect();

        Ok(result)
    }

    /// Extract only the speech portions from audio
    pub fn extract_speech_audio(&self, audio_data: &[f32], segments: &[VadSegment]) -> Vec<f32> {
        if segments.is_empty() {
            return Vec::new();
        }

        // Calculate total length of speech segments
        let total_samples: usize = segments.iter()
            .map(|seg| {
                let start_sample = (seg.start_ms as f32 * 16.0) as usize; // 16kHz
                let end_sample = (seg.end_ms as f32 * 16.0) as usize;
                end_sample.saturating_sub(start_sample)
            })
            .sum();

        let mut speech_audio = Vec::with_capacity(total_samples);

        for segment in segments {
            let start_sample = (segment.start_ms as f32 * 16.0) as usize;
            let end_sample = (segment.end_ms as f32 * 16.0) as usize;

            // Ensure we don't go out of bounds
            let start = start_sample.min(audio_data.len());
            let end = end_sample.min(audio_data.len());

            if start < end {
                speech_audio.extend_from_slice(&audio_data[start..end]);
            }
        }

        speech_audio
    }
}

#[derive(Debug, Clone)]
pub struct VadSegment {
    pub start_ms: u64,
    pub end_ms: u64,
}

impl VadSegment {
    pub fn duration_ms(&self) -> u64 {
        self.end_ms.saturating_sub(self.start_ms)
    }

    pub fn duration_seconds(&self) -> f32 {
        self.duration_ms() as f32 / 1000.0
    }
}

pub fn filter_silence(
    audio_data: &[f32],
    threshold: f32,
    window_size: usize,
) -> Vec<f32> {
    let mut filtered_audio = Vec::new();
    let mut in_speech = false;

    for chunk in audio_data.chunks(window_size) {
        let rms = (chunk.iter().map(|&x| x * x).sum::<f32>() / chunk.len() as f32).sqrt();

        if rms > threshold {
            if !in_speech {
                // Start of speech segment
                in_speech = true;
            }
            filtered_audio.extend_from_slice(chunk);
        } else if in_speech {
            // End of speech segment, add a small buffer
            in_speech = false;
            // Add a small buffer of silence to avoid cutting off words
            let buffer_size = (16000.0 * 0.2) as usize; // 200ms buffer at 16kHz
            let buffer_end = (chunk.len() + buffer_size).min(audio_data.len());
            filtered_audio.extend_from_slice(&audio_data[chunk.len()..buffer_end]);
        }
    }

    filtered_audio
}

// Factory function to create a VAD processor with common settings
pub fn create_vad_processor() -> Result<WhisperVadProcessor, VoiceError> {
    // Try to find a VAD model in common locations
    let possible_paths = vec![
        // Check if VAD model is in the same directory as whisper models
        std::env::var("WHISPER_MODEL_PATH")
            .ok()
            .and_then(|p| Path::new(&p).parent().map(|dir| dir.join("ggml-vad.bin")))
            .and_then(|p| p.to_str().map(|s| s.to_string())),

        // Check app data directory
        Some("./models/ggml-vad.bin".to_string()),
        Some("./ggml-vad.bin".to_string()),
    ];

    for model_path in possible_paths.into_iter().flatten() {
        if Path::new(&model_path).exists() {
            println!("🎯 Using VAD model: {}", model_path);
            return WhisperVadProcessor::new(&model_path);
        }
    }

    Err(VoiceError::Other(
        "VAD model not found. Please download ggml-vad.bin to your models directory.".to_string()
    ))
}