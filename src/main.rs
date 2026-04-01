mod audio;
mod keys;
mod stt;
mod tray;
mod uinput;

use anyhow::{Context, Result};
use clap::Parser;
use evdev::Key;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "vox", about = "Push-to-talk dictation — toggle key, Whisper ONNX, type at cursor")]
struct Cli {
    /// Whisper ONNX encoder model path
    #[arg(long, default_value = "~/.local/share/whisper-onnx/small-encoder.onnx")]
    encoder: String,

    /// Whisper ONNX decoder model path
    #[arg(long, default_value = "~/.local/share/whisper-onnx/small-decoder.onnx")]
    decoder: String,

    /// Tokens file path
    #[arg(long, default_value = "~/.local/share/whisper-onnx/small-tokens.txt")]
    tokens: String,

    /// Language for transcription
    #[arg(short, long, default_value = "fr")]
    language: String,

    /// Debug mode: print all key events
    #[arg(long)]
    debug_keys: bool,
}

fn expand_path(p: &str) -> PathBuf {
    if p.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(p.replacen("~", &home, 1));
        }
    }
    PathBuf::from(p)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.debug_keys {
        return keys::debug_keys();
    }

    eprintln!("[vox] loading whisper ONNX...");
    let whisper = stt::Whisper::new(
        &expand_path(&cli.encoder),
        &expand_path(&cli.decoder),
        &expand_path(&cli.tokens),
        &cli.language,
    )
    .context("failed to load whisper")?;

    eprintln!("[vox] ready — press ² to start/stop dictation");

    ctrlc::set_handler(|| {
        tray::set_idle();
        std::process::exit(0);
    })?;

    let recording = Arc::new(AtomicBool::new(false));
    let recording_key = recording.clone();

    let (tx, rx) = std::sync::mpsc::channel::<Action>();
    let tx = Arc::new(std::sync::Mutex::new(tx));

    // Key listener thread
    std::thread::spawn(move || {
        if let Err(e) = keys::listen_toggle(Key::KEY_GRAVE, move |pressed| {
            if pressed {
                uinput::send_backspace();

                let was_recording = recording_key.fetch_xor(true, Ordering::SeqCst);
                let action = if !was_recording { Action::Start } else { Action::Stop };
                let _ = tx.lock().unwrap().send(action);
            }
        }) {
            eprintln!("[keys] error: {e}");
            std::process::exit(1);
        }
    });

    let mut capture: Option<audio::Capture> = None;

    loop {
        match rx.recv() {
            Ok(Action::Start) => {
                eprintln!("\x1b[91m● REC\x1b[0m");
                tray::set_recording();
                match audio::Capture::start() {
                    Ok(c) => capture = Some(c),
                    Err(e) => {
                        eprintln!("[rec] failed: {e}");
                        tray::set_idle();
                    }
                }
            }
            Ok(Action::Stop) => {
                eprintln!("\x1b[90m■ STOP\x1b[0m");
                if let Some(cap) = capture.take() {
                    tray::set_processing();

                    let samples = cap.stop();
                    let duration_ms = samples.len() as f32 / 16.0;
                    eprintln!("[rec] {duration_ms:.0}ms captured");

                    if samples.len() < 4800 {
                        eprintln!("[stt] too short, skipping");
                        tray::set_idle();
                        continue;
                    }

                    eprintln!("[stt] transcribing...");
                    match whisper.transcribe(&samples) {
                        Ok(text) if !text.is_empty() => {
                            eprintln!("\x1b[92m> {text}\x1b[0m");
                            if let Err(e) = uinput::type_text(&text) {
                                eprintln!("[type] error: {e}");
                            }
                        }
                        Ok(_) => eprintln!("[stt] (empty)"),
                        Err(e) => eprintln!("[stt] error: {e}"),
                    }

                    tray::set_idle();
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}

enum Action {
    Start,
    Stop,
}
