#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Local-only crash log. Never sends data anywhere; user paths and any
/// token-shaped strings are redacted before the line is written.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown".into());
        let safe = redact(&message);
        vexora_lib::log::write("panic", &format!("{location} — {safe}"));
        previous(info);
    }));
}

fn redact(message: &str) -> String {
    let mut out: String = message.chars().take(500).collect();
    if let Ok(home) = std::env::var("USERPROFILE") {
        out = out.replace(&home, "%USERPROFILE%");
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        out = out.replace(&local, "%LOCALAPPDATA%");
    }
    // Strip GitHub-token-shaped strings if one ever reaches a panic message.
    let mut result = String::with_capacity(out.len());
    let mut rest = out.as_str();
    while let Some(idx) = rest.find("ghp_") {
        result.push_str(&rest[..idx]);
        rest = &rest[idx + 4..];
        let skip = rest.find(char::is_whitespace).unwrap_or(rest.len());
        result.push_str("[REDACTED]");
        rest = &rest[skip..];
    }
    result.push_str(rest);
    result
}

fn main() {
    install_panic_hook();
    // Silent background mode used by the Task Scheduler.
    if std::env::args().any(|a| a == "--autoclean") {
        vexora_lib::run_autoclean();
        return;
    }

    // Move old Abu Maryam Cleaner settings/history to the VEXORA profile.
    vexora_lib::util::migrate_old_data();

    // Upgrade to administrator when the user allowed it, then make sure only
    // one VEXORA window is running.
    vexora_lib::elevate_if_needed();
    vexora_lib::util::ensure_single_instance();

    vexora_lib::run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crash_log_redacts_tokens() {
        let s = redact("failed for ghp_ABCDEFG1234567890 on C:\\Users\\me\\file.txt");
        assert!(s.contains("[REDACTED]"));
        assert!(!s.contains("ghp_"));
    }
}
