use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ModelSet {
    pub encoder: PathBuf,
    pub decoder: PathBuf,
    pub tokens: PathBuf,
}

pub struct ModelConfig {
    pub name: String,
    pub base_url: String,
    pub encoder_file: String,
    pub decoder_file: String,
    pub tokens_file: String,
}

impl ModelConfig {
    pub fn from_name(name: &str) -> Self {
        // Extract base name (e.g. "small", "turbo", "medium")
        // and detect int8 suffix
        let (base, int8) = if name.ends_with("-int8") {
            (&name[..name.len() - 5], true)
        } else {
            (name, false)
        };

        let suffix = if int8 { ".int8" } else { "" };

        Self {
            name: name.into(),
            base_url: format!("https://huggingface.co/csukuangfj/sherpa-onnx-whisper-{base}/resolve/main"),
            encoder_file: format!("{base}-encoder{suffix}.onnx"),
            decoder_file: format!("{base}-decoder{suffix}.onnx"),
            tokens_file: format!("{base}-tokens.txt"),
        }
    }
}

pub fn model_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".local/share/whisper-onnx")
}

pub fn ensure_models(config: &ModelConfig) -> Result<ModelSet> {
    let dir = model_dir();
    let encoder = dir.join(&config.encoder_file);
    let decoder = dir.join(&config.decoder_file);
    let tokens = dir.join(&config.tokens_file);

    if encoder.exists() && decoder.exists() && tokens.exists() {
        return Ok(ModelSet { encoder, decoder, tokens });
    }

    eprintln!("[vox] models not found, downloading {}...", config.name);
    fs::create_dir_all(&dir)
        .with_context(|| format!("cannot create {}", dir.display()))?;

    for (file, dest) in [
        (&config.encoder_file, &encoder),
        (&config.decoder_file, &decoder),
        (&config.tokens_file, &tokens),
    ] {
        if dest.exists() {
            continue;
        }
        download(&format!("{}/{file}", config.base_url), dest)?;
    }

    eprintln!("[vox] models ready");
    Ok(ModelSet { encoder, decoder, tokens })
}

fn download(url: &str, dest: &Path) -> Result<()> {
    let name = dest.file_name().unwrap().to_str().unwrap();
    eprintln!("[download] {name}...");

    let status = Command::new("curl")
        .args(["-L", "-o"])
        .arg(dest)
        .arg(url)
        .args(["--progress-bar"])
        .status()
        .context("curl not found")?;

    if !status.success() {
        anyhow::bail!("failed to download {name}");
    }

    Ok(())
}
