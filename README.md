# vox

Push-to-talk dictation. Press ² to record, press again to transcribe and type at cursor. Local Whisper, no cloud.

## How it works

```
² (start) → record audio → ² (stop) → Whisper transcribes → text typed at cursor
```

- **STT**: Whisper large-v3-turbo int8 via sherpa-onnx (~8% WER French, ~2s latency)
- **Audio**: cpal (16kHz mono, ALSA/PipeWire)
- **Typing**: uinput virtual keyboard (AZERTY, works in any Wayland app including terminals)
- **Hotkey**: evdev (all keyboards, auto udev setup)

## Install

```bash
curl -L https://github.com/Destynova2/vox/releases/latest/download/vox-linux-x86_64 -o ~/.local/bin/vox
chmod +x ~/.local/bin/vox
```

On first run, vox will:
1. Download Whisper turbo int8 models (~1 GB) to `~/.local/share/whisper-onnx/`
2. Install udev rules for `/dev/input` and `/dev/uinput` via pkexec (one-time)

## Usage

```bash
vox                # French (default)
vox -l en          # English
vox -m small       # use a different model (small, medium, turbo-int8...)
vox --debug-keys   # show key codes
```

## Build from source

```bash
make build    # auto-detects linuxbrew
make install  # copies to ~/.local/bin/vox
```

Requires `alsa-lib-devel` and a C++ compiler.

## License

MIT
