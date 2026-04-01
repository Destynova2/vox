# Roadmap

## Done
- [x] Toggle key (²) start/stop recording
- [x] Audio capture via cpal (16kHz mono)
- [x] Whisper ONNX transcription via sherpa-onnx (model small)
- [x] Type text at cursor via uinput virtual AZERTY keyboard
- [x] Backspace auto-erase of ² key leak
- [x] Multi-keyboard support (evdev)
- [x] Auto udev rule install via pkexec (input + uinput)
- [x] Debug key mode (--debug-keys)
- [x] GNOME desktop notifications (record/processing/idle)

## Next
- [ ] macOS support (CGEventTap pour hotkey, CGEventPost pour typing, pbcopy pour clipboard)
- [ ] Configurable hotkey (pas que ²)
- [ ] Support multi-layout clavier (BÉPO, QWERTY, etc.)
- [ ] System tray icon (ksni/StatusNotifierItem)
