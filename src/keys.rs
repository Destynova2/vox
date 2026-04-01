use anyhow::{Context, Result};
use evdev::{Device, EventType, InputEventKind, Key};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const UDEV_RULES: &str = r#"SUBSYSTEM=="input", KERNEL=="event*", TAG+="uaccess"
KERNEL=="uinput", SUBSYSTEM=="misc", TAG+="uaccess""#;
const UDEV_RULE_PATH: &str = "/etc/udev/rules.d/71-voice-type-input.rules";

fn find_keyboards() -> Result<Vec<PathBuf>> {
    let input_dir = fs::read_dir("/dev/input")
        .context("cannot read /dev/input")?;

    let mut keyboards = Vec::new();
    for entry in input_dir.flatten() {
        let path = entry.path();
        if !path.to_str().map_or(false, |s| s.contains("event")) {
            continue;
        }
        if let Ok(dev) = Device::open(&path) {
            if let Some(keys) = dev.supported_keys() {
                if keys.contains(Key::KEY_A) {
                    let name = dev.name().unwrap_or("unknown").to_string();
                    eprintln!("[keys] found keyboard: {} ({})", name, path.display());
                    keyboards.push(path);
                }
            }
        }
    }

    if keyboards.is_empty() {
        anyhow::bail!("no keyboard found");
    }
    Ok(keyboards)
}

fn install_udev_rule() -> Result<()> {
    if fs::metadata(UDEV_RULE_PATH).is_ok() {
        return Ok(());
    }

    eprintln!("[keys] installing udev rule for input access...");

    let status = Command::new("pkexec")
        .args(["bash", "-c", &format!(
            "echo '{}' > {} && udevadm control --reload-rules && udevadm trigger",
            UDEV_RULES, UDEV_RULE_PATH
        )])
        .status()
        .context("pkexec not found")?;

    if !status.success() {
        anyhow::bail!("failed to install udev rule (auth cancelled?)");
    }

    eprintln!("[keys] udev rule installed — retrying...");
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(())
}

fn get_keyboards() -> Result<Vec<PathBuf>> {
    match find_keyboards() {
        Ok(k) if !k.is_empty() => Ok(k),
        _ => {
            install_udev_rule()?;
            find_keyboards()
        }
    }
}

/// Debug mode: print all key events from all keyboards.
pub fn debug_keys() -> Result<()> {
    let keyboards = get_keyboards()?;

    eprintln!("[keys] DEBUG — press keys to see their codes (Ctrl+C to quit)");

    // Listen on all keyboards in parallel
    let mut handles = Vec::new();
    for path in keyboards {
        handles.push(std::thread::spawn(move || {
            let mut dev = match Device::open(&path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[keys] cannot open {}: {e}", path.display());
                    return;
                }
            };
            let name = dev.name().unwrap_or("?").to_string();
            loop {
                match dev.fetch_events() {
                    Ok(events) => {
                        for ev in events {
                            if ev.event_type() == EventType::KEY {
                                let action = match ev.value() {
                                    0 => "UP",
                                    1 => "DOWN",
                                    2 => "REPEAT",
                                    _ => "?",
                                };
                                eprintln!("[{}] {:?} {} (code={})", name, ev.kind(), action, ev.code());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[keys] error on {}: {e}", name);
                        break;
                    }
                }
            }
        }));
    }

    for h in handles {
        let _ = h.join();
    }
    Ok(())
}

/// Listen for a toggle key on all keyboards.
/// Calls `on_press(true)` on key-down only.
pub fn listen_toggle(key: Key, on_press: impl Fn(bool) + Send + Sync + 'static) -> Result<()> {
    let keyboards = get_keyboards()?;
    let on_press = std::sync::Arc::new(on_press);

    let mut handles = Vec::new();
    for path in keyboards {
        let key = key;
        let on_press = on_press.clone();
        handles.push(std::thread::spawn(move || {
            let mut dev = match Device::open(&path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[keys] cannot open {}: {e}", path.display());
                    return;
                }
            };
            loop {
                match dev.fetch_events() {
                    Ok(events) => {
                        for ev in events {
                            if ev.event_type() == EventType::KEY {
                                if let InputEventKind::Key(k) = ev.kind() {
                                    if k == key && ev.value() == 1 {
                                        on_press(true);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[keys] error on {}: {e}", path.display());
                        break;
                    }
                }
            }
        }));
    }

    for h in handles {
        let _ = h.join();
    }
    Ok(())
}
