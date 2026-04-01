use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 16_000;

pub struct Capture {
    _stream: cpal::Stream,
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl Capture {
    pub fn start() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("no input device found")?;

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let buf_clone = buffer.clone();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut buf) = buf_clone.lock() {
                    buf.extend_from_slice(data);
                }
            },
            |err| eprintln!("[audio] error: {err}"),
            None,
        )?;

        stream.play()?;
        Ok(Self {
            _stream: stream,
            buffer,
        })
    }

    pub fn stop(self) -> Vec<f32> {
        // Stream drops here, stopping capture
        drop(self._stream);
        let buf = self.buffer.lock().unwrap();
        buf.clone()
    }
}
