# vox

Push-to-talk dictation tool. Press a key to record, press again to transcribe and type at cursor. Runs locally with Whisper ONNX, no cloud, no latency.

## How it works

```
² (start) → record audio → ² (stop) → Whisper transcribes → text typed at cursor
```

- **Audio**: cpal (16kHz mono, any ALSA/PipeWire device)
- **STT**: Whisper small via sherpa-onnx (ONNX Runtime, ~2-3s for short utterances)
- **Typing**: uinput virtual keyboard (AZERTY layout, works in any Wayland app including terminals)
- **Hotkey**: evdev (listens on all keyboards, auto udev rule install)

## Requirements

- Linux (Wayland)
- PipeWire or ALSA audio input
- Whisper ONNX models in `~/.local/share/whisper-onnx/`:
  - `small-encoder.onnx`
  - `small-decoder.onnx`
  - `small-tokens.txt`

Download models:
```bash
mkdir -p ~/.local/share/whisper-onnx && cd ~/.local/share/whisper-onnx
curl -LO https://huggingface.co/csukuangfj/sherpa-onnx-whisper-small/resolve/main/small-encoder.onnx
curl -LO https://huggingface.co/csukuangfj/sherpa-onnx-whisper-small/resolve/main/small-decoder.onnx
curl -LO https://huggingface.co/csukuangfj/sherpa-onnx-whisper-small/resolve/main/small-tokens.txt
```

## Build

```bash
./build.sh
```

## Usage

```bash
./target/release/vox              # start (default: French, ² key)
./target/release/vox -l en        # English
./target/release/vox --debug-keys # show key codes to find your hotkey
```

On first run, if `/dev/input` or `/dev/uinput` are not accessible, vox will prompt via `pkexec` to install a udev rule.

## License

MIT
