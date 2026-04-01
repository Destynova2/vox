use anyhow::{Context, Result};
use sherpa_onnx::{
    OfflineRecognizer, OfflineRecognizerConfig, OfflineWhisperModelConfig,
};
use std::path::Path;

pub struct Whisper {
    recognizer: OfflineRecognizer,
}

impl Whisper {
    pub fn new(
        encoder: &Path,
        decoder: &Path,
        tokens: &Path,
        language: &str,
    ) -> Result<Self> {
        let mut config = OfflineRecognizerConfig::default();
        config.model_config.whisper = OfflineWhisperModelConfig {
            encoder: Some(encoder.to_str().context("encoder path not UTF-8")?.into()),
            decoder: Some(decoder.to_str().context("decoder path not UTF-8")?.into()),
            language: Some(language.into()),
            task: Some("transcribe".into()),
            tail_paddings: -1,
            ..Default::default()
        };
        config.model_config.tokens = Some(tokens.to_str().context("tokens path not UTF-8")?.into());
        config.model_config.num_threads = 4;
        config.model_config.debug = false;

        let recognizer =
            OfflineRecognizer::create(&config).context("failed to create whisper recognizer")?;

        Ok(Self { recognizer })
    }

    pub fn transcribe(&self, samples: &[f32]) -> Result<String> {
        let stream = self.recognizer.create_stream();
        stream.accept_waveform(16000, samples);
        self.recognizer.decode(&stream);
        let result = stream.get_result().context("no result from whisper")?;
        Ok(result.text.trim().to_string())
    }
}
