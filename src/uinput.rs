use anyhow::{Context, Result};
use evdev::uinput::VirtualDeviceBuilder;
use evdev::{AttributeSet, EventType, InputEvent, Key, uinput::VirtualDevice};
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
    if get_or_create().is_err() {
        return;
    }
    let _ = emit_key(Key::KEY_BACKSPACE, true);
    let _ = emit_key(Key::KEY_BACKSPACE, false);
}

/// Type text character by character via uinput (AZERTY layout).
pub fn type_text(text: &str) -> Result<()> {
    get_or_create()?;

    for ch in text.chars() {
        if let Some((key, shift)) = char_to_key(ch) {
            if shift {
                emit_key(Key::KEY_LEFTSHIFT, true)?;
            }
            emit_key(key, true)?;
            emit_key(key, false)?;
            if shift {
                emit_key(Key::KEY_LEFTSHIFT, false)?;
            }
        }
        // Unknown chars are silently skipped
    }

    Ok(())
}

/// Map a character to (evdev Key, needs_shift) for French AZERTY layout.
fn char_to_key(ch: char) -> Option<(Key, bool)> {
    let (key, shift) = match ch {
        // Space & newline
        ' ' => (Key::KEY_SPACE, false),
        '\n' => (Key::KEY_ENTER, false),
        '\t' => (Key::KEY_TAB, false),

        // Lowercase letters (AZERTY positions)
        'a' => (Key::KEY_Q, false),
        'b' => (Key::KEY_B, false),
        'c' => (Key::KEY_C, false),
        'd' => (Key::KEY_D, false),
        'e' => (Key::KEY_E, false),
        'f' => (Key::KEY_F, false),
        'g' => (Key::KEY_G, false),
        'h' => (Key::KEY_H, false),
        'i' => (Key::KEY_I, false),
        'j' => (Key::KEY_J, false),
        'k' => (Key::KEY_K, false),
        'l' => (Key::KEY_L, false),
        'm' => (Key::KEY_SEMICOLON, false),
        'n' => (Key::KEY_N, false),
        'o' => (Key::KEY_O, false),
        'p' => (Key::KEY_P, false),
        'q' => (Key::KEY_A, false),
        'r' => (Key::KEY_R, false),
        's' => (Key::KEY_S, false),
        't' => (Key::KEY_T, false),
        'u' => (Key::KEY_U, false),
        'v' => (Key::KEY_V, false),
        'w' => (Key::KEY_Z, false),
        'x' => (Key::KEY_X, false),
        'y' => (Key::KEY_Y, false),
        'z' => (Key::KEY_W, false),

        // Uppercase letters
        'A' => (Key::KEY_Q, true),
        'B' => (Key::KEY_B, true),
        'C' => (Key::KEY_C, true),
        'D' => (Key::KEY_D, true),
        'E' => (Key::KEY_E, true),
        'F' => (Key::KEY_F, true),
        'G' => (Key::KEY_G, true),
        'H' => (Key::KEY_H, true),
        'I' => (Key::KEY_I, true),
        'J' => (Key::KEY_J, true),
        'K' => (Key::KEY_K, true),
        'L' => (Key::KEY_L, true),
        'M' => (Key::KEY_SEMICOLON, true),
        'N' => (Key::KEY_N, true),
        'O' => (Key::KEY_O, true),
        'P' => (Key::KEY_P, true),
        'Q' => (Key::KEY_A, true),
        'R' => (Key::KEY_R, true),
        'S' => (Key::KEY_S, true),
        'T' => (Key::KEY_T, true),
        'U' => (Key::KEY_U, true),
        'V' => (Key::KEY_V, true),
        'W' => (Key::KEY_Z, true),
        'X' => (Key::KEY_X, true),
        'Y' => (Key::KEY_Y, true),
        'Z' => (Key::KEY_W, true),

        // Punctuation (AZERTY)
        ',' => (Key::KEY_M, false),
        ';' => (Key::KEY_COMMA, false),
        ':' => (Key::KEY_DOT, false),
        '!' => (Key::KEY_SLASH, false),
        '?' => (Key::KEY_M, true),
        '.' => (Key::KEY_COMMA, true),
        '/' => (Key::KEY_DOT, true),
        '§' => (Key::KEY_SLASH, true),

        // Number row (unshifted = symbols, shifted = digits on AZERTY)
        '&' => (Key::KEY_1, false),
        '1' => (Key::KEY_1, true),
        'é' => (Key::KEY_2, false),
        '2' => (Key::KEY_2, true),
        '"' => (Key::KEY_3, false),
        '3' => (Key::KEY_3, true),
        '\'' => (Key::KEY_4, false),
        '4' => (Key::KEY_4, true),
        '(' => (Key::KEY_5, false),
        '5' => (Key::KEY_5, true),
        '-' => (Key::KEY_6, false),
        '6' => (Key::KEY_6, true),
        'è' => (Key::KEY_7, false),
        '7' => (Key::KEY_7, true),
        '_' => (Key::KEY_8, false),
        '8' => (Key::KEY_8, true),
        'ç' => (Key::KEY_9, false),
        '9' => (Key::KEY_9, true),
        'à' => (Key::KEY_0, false),
        '0' => (Key::KEY_0, true),
        ')' => (Key::KEY_MINUS, false),
        '°' => (Key::KEY_MINUS, true),
        '=' => (Key::KEY_EQUAL, false),
        '+' => (Key::KEY_EQUAL, true),

        // Other common chars
        'ù' => (Key::KEY_APOSTROPHE, false),
        '%' => (Key::KEY_APOSTROPHE, true),
        '*' => (Key::KEY_BACKSLASH, false),
        'µ' => (Key::KEY_BACKSLASH, true),
        '$' => (Key::KEY_RIGHTBRACE, false),
        '£' => (Key::KEY_RIGHTBRACE, true),
        '<' => (Key::KEY_102ND, false),
        '>' => (Key::KEY_102ND, true),

        _ => return None,
    };
    Some((key, shift))
}
