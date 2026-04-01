use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

/// Stores the notification ID for replacement
static NOTIFY_ID: AtomicU32 = AtomicU32::new(0);

pub fn set_recording() {
    send_notification("🔴 Enregistrement...", "media-record", "critical");
}

pub fn set_processing() {
    send_notification("⏳ Transcription...", "preferences-system", "normal");
}

pub fn set_idle() {
    // Close the notification
    let id = NOTIFY_ID.load(Ordering::SeqCst);
    if id > 0 {
        let _ = Command::new("gdbus")
            .args([
                "call", "--session",
                "--dest=org.freedesktop.Notifications",
                "--object-path=/org/freedesktop/Notifications",
                "--method=org.freedesktop.Notifications.CloseNotification",
                &id.to_string(),
            ])
            .output();
    }
}

fn send_notification(body: &str, icon: &str, urgency: &str) {
    let id = NOTIFY_ID.load(Ordering::SeqCst);

    let output = Command::new("notify-send")
        .args([
            "--app-name=vox",
            &format!("--urgency={urgency}"),
            &format!("--icon={icon}"),
            "--print-id",
            &format!("--replace-id={id}"),
            "vox",
            body,
        ])
        .output();

    if let Ok(out) = output {
        if let Ok(s) = String::from_utf8(out.stdout) {
            if let Ok(new_id) = s.trim().parse::<u32>() {
                NOTIFY_ID.store(new_id, Ordering::SeqCst);
            }
        }
    }
}
