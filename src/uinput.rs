use anyhow::{Context, Result};
use evdev::uinput::VirtualDeviceBuilder;
use evdev::{AttributeSet, EventType, InputEvent, Key, uinput::VirtualDevice};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

static VDEV: Mutex<Option<VirtualDevice>> = Mutex::new(None);

fn get_or_create() -> Result<()> {
    let mut guard = VDEV.lock().unwrap();
    if guard.is_some() {
        return Ok(());
    }

    let mut keys = AttributeSet::new();
    for code in 1..=248 {
        keys.insert(Key::new(code));
    }

    let vdev = VirtualDeviceBuilder::new()
        .context("cannot open /dev/uinput")?
        .name("vox-keyboard")
        .with_keys(&keys)
        .context("keys setup")?
        .build()
        .context("cannot create virtual device")?;

    std::thread::sleep(Duration::from_millis(300));

    *guard = Some(vdev);
    Ok(())
}

fn emit_key(key: Key, press: bool) -> Result<()> {
    let mut guard = VDEV.lock().unwrap();
    let vdev = guard.as_mut().context("no virtual device")?;
    let ev = InputEvent::new(EventType::KEY, key.code(), if press { 1 } else { 0 });
    let syn = InputEvent::new(EventType::SYNCHRONIZATION, 0, 0);
    vdev.emit(&[ev, syn])?;
    std::thread::sleep(Duration::from_millis(5));
    Ok(())
}

/// Send a backspace to erase the ² character that leaked through.
pub fn send_backspace() {
    if let Err(e) = get_or_create() {
        eprintln!("[uinput] backspace failed: {e}");
        return;
    }
    if let Err(e) = emit_key(Key::KEY_BACKSPACE, true)
        .and_then(|_| emit_key(Key::KEY_BACKSPACE, false))
    {
        eprintln!("[uinput] backspace failed: {e}");
    }
}

/// Type text character by character via uinput, using the detected keyboard layout.
pub fn type_text(text: &str, layout: &Layout) -> Result<()> {
    get_or_create()?;

    for ch in text.chars() {
        if let Some((key, shift)) = layout.char_to_key(ch) {
            if shift {
                emit_key(Key::KEY_LEFTSHIFT, true)?;
            }
            emit_key(key, true)?;
            emit_key(key, false)?;
            if shift {
                emit_key(Key::KEY_LEFTSHIFT, false)?;
            }
        }
    }

    Ok(())
}

// --- Keyboard layout detection & mapping ---

pub struct Layout {
    map: HashMap<char, (Key, bool)>,
    pub name: String,
}

impl Layout {
    pub fn detect() -> Self {
        let name = detect_layout_name();
        eprintln!("[layout] detected: {name}");
        match name.as_str() {
            "us" | "gb" | "en" => Self { map: qwerty_map(), name },
            "de" | "at" | "ch" => Self { map: qwertz_map(), name },
            _ => Self { map: azerty_map(), name }, // fr and default
        }
    }

    pub fn char_to_key(&self, ch: char) -> Option<(Key, bool)> {
        self.map.get(&ch).copied()
    }
}

fn detect_layout_name() -> String {
    // 1. GNOME/Wayland: dconf
    if let Ok(out) = std::process::Command::new("dconf")
        .args(["read", "/org/gnome/desktop/input-sources/sources"])
        .output()
    {
        let s = String::from_utf8_lossy(&out.stdout);
        // Format: [('xkb', 'fr')]
        if let Some(start) = s.rfind("'") {
            let before = &s[..start];
            if let Some(begin) = before.rfind("'") {
                let layout = &before[begin + 1..];
                if !layout.is_empty() {
                    return layout.to_string();
                }
            }
        }
    }

    // 2. localectl
    if let Ok(out) = std::process::Command::new("localectl").arg("status").output() {
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines() {
            if line.contains("X11 Layout:") {
                if let Some(layout) = line.split(':').nth(1) {
                    let l = layout.trim();
                    if !l.is_empty() {
                        return l.to_string();
                    }
                }
            }
        }
    }

    "fr".to_string()
}

fn common_map() -> HashMap<char, (Key, bool)> {
    let mut m = HashMap::new();
    m.insert(' ', (Key::KEY_SPACE, false));
    m.insert('\n', (Key::KEY_ENTER, false));
    m.insert('\t', (Key::KEY_TAB, false));
    m
}

fn azerty_map() -> HashMap<char, (Key, bool)> {
    let mut m = common_map();

    // Letters
    for (ch, key) in [
        ('a', Key::KEY_Q), ('b', Key::KEY_B), ('c', Key::KEY_C), ('d', Key::KEY_D),
        ('e', Key::KEY_E), ('f', Key::KEY_F), ('g', Key::KEY_G), ('h', Key::KEY_H),
        ('i', Key::KEY_I), ('j', Key::KEY_J), ('k', Key::KEY_K), ('l', Key::KEY_L),
        ('m', Key::KEY_SEMICOLON), ('n', Key::KEY_N), ('o', Key::KEY_O), ('p', Key::KEY_P),
        ('q', Key::KEY_A), ('r', Key::KEY_R), ('s', Key::KEY_S), ('t', Key::KEY_T),
        ('u', Key::KEY_U), ('v', Key::KEY_V), ('w', Key::KEY_Z), ('x', Key::KEY_X),
        ('y', Key::KEY_Y), ('z', Key::KEY_W),
    ] {
        m.insert(ch, (key, false));
        m.insert(ch.to_ascii_uppercase(), (key, true));
    }

    // Punctuation
    for (ch, key, shift) in [
        (',', Key::KEY_M, false), (';', Key::KEY_COMMA, false), (':', Key::KEY_DOT, false),
        ('!', Key::KEY_SLASH, false), ('?', Key::KEY_M, true), ('.', Key::KEY_COMMA, true),
        ('/', Key::KEY_DOT, true),
    ] {
        m.insert(ch, (key, shift));
    }

    // Number row
    for (ch_normal, ch_shift, key) in [
        ('&', '1', Key::KEY_1), ('é', '2', Key::KEY_2), ('"', '3', Key::KEY_3),
        ('\'', '4', Key::KEY_4), ('(', '5', Key::KEY_5), ('-', '6', Key::KEY_6),
        ('è', '7', Key::KEY_7), ('_', '8', Key::KEY_8), ('ç', '9', Key::KEY_9),
        ('à', '0', Key::KEY_0),
    ] {
        m.insert(ch_normal, (key, false));
        m.insert(ch_shift, (key, true));
    }

    // Extra
    for (ch, key, shift) in [
        (')', Key::KEY_MINUS, false), ('°', Key::KEY_MINUS, true),
        ('=', Key::KEY_EQUAL, false), ('+', Key::KEY_EQUAL, true),
        ('ù', Key::KEY_APOSTROPHE, false), ('%', Key::KEY_APOSTROPHE, true),
        ('*', Key::KEY_BACKSLASH, false), ('$', Key::KEY_RIGHTBRACE, false),
        ('£', Key::KEY_RIGHTBRACE, true), ('<', Key::KEY_102ND, false),
        ('>', Key::KEY_102ND, true),
    ] {
        m.insert(ch, (key, shift));
    }

    m
}

fn qwerty_map() -> HashMap<char, (Key, bool)> {
    let mut m = common_map();

    // Letters (same physical position = same keycode on QWERTY)
    for (ch, key) in [
        ('a', Key::KEY_A), ('b', Key::KEY_B), ('c', Key::KEY_C), ('d', Key::KEY_D),
        ('e', Key::KEY_E), ('f', Key::KEY_F), ('g', Key::KEY_G), ('h', Key::KEY_H),
        ('i', Key::KEY_I), ('j', Key::KEY_J), ('k', Key::KEY_K), ('l', Key::KEY_L),
        ('m', Key::KEY_M), ('n', Key::KEY_N), ('o', Key::KEY_O), ('p', Key::KEY_P),
        ('q', Key::KEY_Q), ('r', Key::KEY_R), ('s', Key::KEY_S), ('t', Key::KEY_T),
        ('u', Key::KEY_U), ('v', Key::KEY_V), ('w', Key::KEY_W), ('x', Key::KEY_X),
        ('y', Key::KEY_Y), ('z', Key::KEY_Z),
    ] {
        m.insert(ch, (key, false));
        m.insert(ch.to_ascii_uppercase(), (key, true));
    }

    // Number row
    for (ch, key) in [
        ('1', Key::KEY_1), ('2', Key::KEY_2), ('3', Key::KEY_3), ('4', Key::KEY_4),
        ('5', Key::KEY_5), ('6', Key::KEY_6), ('7', Key::KEY_7), ('8', Key::KEY_8),
        ('9', Key::KEY_9), ('0', Key::KEY_0),
    ] {
        m.insert(ch, (key, false));
    }

    // Punctuation
    for (ch, key, shift) in [
        ('.', Key::KEY_DOT, false), (',', Key::KEY_COMMA, false),
        (';', Key::KEY_SEMICOLON, false), (':', Key::KEY_SEMICOLON, true),
        ('/', Key::KEY_SLASH, false), ('?', Key::KEY_SLASH, true),
        ('\'', Key::KEY_APOSTROPHE, false), ('"', Key::KEY_APOSTROPHE, true),
        ('-', Key::KEY_MINUS, false), ('_', Key::KEY_MINUS, true),
        ('=', Key::KEY_EQUAL, false), ('+', Key::KEY_EQUAL, true),
        ('!', Key::KEY_1, true), ('@', Key::KEY_2, true),
        ('#', Key::KEY_3, true), ('$', Key::KEY_4, true),
        ('%', Key::KEY_5, true), ('&', Key::KEY_7, true),
        ('*', Key::KEY_8, true), ('(', Key::KEY_9, true),
        (')', Key::KEY_0, true), ('[', Key::KEY_LEFTBRACE, false),
        (']', Key::KEY_RIGHTBRACE, false), ('<', Key::KEY_COMMA, true),
        ('>', Key::KEY_DOT, true),
    ] {
        m.insert(ch, (key, shift));
    }

    m
}

fn qwertz_map() -> HashMap<char, (Key, bool)> {
    let mut m = common_map();

    // Letters (QWERTZ: Y and Z swapped vs QWERTY)
    for (ch, key) in [
        ('a', Key::KEY_A), ('b', Key::KEY_B), ('c', Key::KEY_C), ('d', Key::KEY_D),
        ('e', Key::KEY_E), ('f', Key::KEY_F), ('g', Key::KEY_G), ('h', Key::KEY_H),
        ('i', Key::KEY_I), ('j', Key::KEY_J), ('k', Key::KEY_K), ('l', Key::KEY_L),
        ('m', Key::KEY_M), ('n', Key::KEY_N), ('o', Key::KEY_O), ('p', Key::KEY_P),
        ('q', Key::KEY_Q), ('r', Key::KEY_R), ('s', Key::KEY_S), ('t', Key::KEY_T),
        ('u', Key::KEY_U), ('v', Key::KEY_V), ('w', Key::KEY_W), ('x', Key::KEY_X),
        ('y', Key::KEY_Z), ('z', Key::KEY_Y),
    ] {
        m.insert(ch, (key, false));
        m.insert(ch.to_ascii_uppercase(), (key, true));
    }

    // Number row
    for (ch, key) in [
        ('1', Key::KEY_1), ('2', Key::KEY_2), ('3', Key::KEY_3), ('4', Key::KEY_4),
        ('5', Key::KEY_5), ('6', Key::KEY_6), ('7', Key::KEY_7), ('8', Key::KEY_8),
        ('9', Key::KEY_9), ('0', Key::KEY_0),
    ] {
        m.insert(ch, (key, false));
    }

    // Punctuation
    for (ch, key, shift) in [
        ('.', Key::KEY_DOT, false), (',', Key::KEY_COMMA, false),
        ('-', Key::KEY_SLASH, false), ('_', Key::KEY_SLASH, true),
        (';', Key::KEY_COMMA, true), (':', Key::KEY_DOT, true),
        ('!', Key::KEY_1, true), ('"', Key::KEY_2, true),
        ('\'', Key::KEY_BACKSLASH, true), ('?', Key::KEY_MINUS, true),
        ('+', Key::KEY_RIGHTBRACE, false), ('*', Key::KEY_RIGHTBRACE, true),
        ('(', Key::KEY_8, true), (')', Key::KEY_9, true),
        ('=', Key::KEY_0, true), ('<', Key::KEY_102ND, false),
        ('>', Key::KEY_102ND, true),
    ] {
        m.insert(ch, (key, shift));
    }

    m
}
