use std::path::Path;
use crate::voice_assistant::VoiceError;

// Temporary VAD processor using energy-based detection
// TODO: Replace with whisper-rs VAD API when available
pub struct WhisperVadProcessor {
    sample_rate: u32,
    threshold: f32,
    window_size: usize,
}

impl WhisperVadProcessor {
    pub fn new(_model_path: &str) -> Result<Self, VoiceError> {
        // For now, ignore model_path and use energy-based VAD
        println!("⚠️  Using energy-based VAD (whisper-rs VAD API not available)");

        Ok(Self {
            sample_rate: 16000,
            threshold: 0.01f32,
            window_size: 1024,
        })
    }

    /// Detect speech segments in audio data using energy-based detection
    pub fn detect_speech_segments(&self, audio_data: &[f32]) -> Result<Vec<VadSegment>, VoiceError> {
        if audio_data.is_empty() {
            return Ok(Vec::new());
        }

        let mut segments = Vec::new();
        let mut in_speech = false;
        let mut speech_start = 0usize;

        let samples_per_ms = self.sample_rate as usize / 1000;

        for (i, window) in audio_data.chunks(self.window_size).enumerate() {
            let energy = window.iter().map(|&x| x * x).sum::<f32>() / window.len() as f32;
            let energy_sqrt = energy.sqrt();

            if energy_sqrt > self.threshold {
                if !in_speech {
                    // Speech starts
                    in_speech = true;
                    speech_start = i * self.window_size;
                }
            } else {
                if in_speech {
                    // Speech ends
                    in_speech = false;
                    let speech_end = i * self.window_size;

                    // Only add segment if it's long enough (> 100ms)
                    let duration_ms = (speech_end - speech_start) / samples_per_ms;
                    if duration_ms > 100 {
                        segments.push(VadSegment {
                            start_ms: (speech_start / samples_per_ms) as u64,
                            end_ms: (speech_end / samples_per_ms) as u64,
                        });
                    }
                }
            }
        }

        // Handle case where audio ends while still in speech
        if in_speech {
            let speech_end = audio_data.len();
            let duration_ms = (speech_end - speech_start) / samples_per_ms;
            if duration_ms > 100 {
                segments.push(VadSegment {
                    start_ms: (speech_start / samples_per_ms) as u64,
                    end_ms: (speech_end / samples_per_ms) as u64,
                });
            }
        }

        Ok(segments)
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