use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, OfflineWhisperModelConfig, Wave};

fn main() {
    let wav_path = std::env::args().nth(1).unwrap_or("/tmp/test-voice-16k.wav".into());

    let home = std::env::var("HOME").unwrap();
    let base = format!("{home}/.local/share/whisper-onnx");

    let mut config = OfflineRecognizerConfig::default();
    config.model_config.whisper = OfflineWhisperModelConfig {
        encoder: Some(format!("{base}/turbo-encoder.int8.onnx")),
        decoder: Some(format!("{base}/turbo-decoder.int8.onnx")),
        language: Some("fr".into()),
        task: Some("transcribe".into()),
        tail_paddings: -1,
        ..Default::default()
    };
    config.model_config.tokens = Some(format!("{base}/turbo-tokens.txt"));
    config.model_config.num_threads = 4;
    config.model_config.debug = false;

    eprintln!("Loading whisper model...");
    let recognizer = OfflineRecognizer::create(&config).expect("failed to create recognizer");

    eprintln!("Reading {wav_path}...");
    let wave = Wave::read(&wav_path).expect("failed to read wav");
    eprintln!("Audio: {} samples, {}Hz, {:.1}s", wave.samples().len(), wave.sample_rate(), wave.samples().len() as f32 / wave.sample_rate() as f32);

    let stream = recognizer.create_stream();
    stream.accept_waveform(wave.sample_rate(), wave.samples());

    eprintln!("Transcribing...");
    recognizer.decode(&stream);

    let result = stream.get_result().expect("no result");
    println!("Result: \"{}\"", result.text.trim());
}
