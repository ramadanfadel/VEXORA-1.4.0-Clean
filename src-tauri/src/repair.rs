//! System file repair: DISM + SFC, streamed to the UI.

use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

pub static REPAIR_CANCEL: AtomicBool = AtomicBool::new(false);
pub static REPAIR_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RepairProgress {
    pub stage: String,
    pub percent: i32,
    pub line: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RepairResult {
    pub ok: bool,
    pub cancelled: bool,
    pub needs_admin: bool,
    pub summary: String,
    pub log: String,
}

pub fn run(app: &tauri::AppHandle) -> RepairResult {
    use tauri::Emitter;
    REPAIR_CANCEL.store(false, Ordering::Relaxed);
    REPAIR_RUNNING.store(true, Ordering::Relaxed);

    let mut result = RepairResult {
        ok: true,
        cancelled: false,
        needs_admin: false,
        summary: String::new(),
        log: String::new(),
    };

    if !crate::util::is_admin() {
        result.needs_admin = true;
        result.ok = false;
        result.summary = "requiresAdmin".into();
        REPAIR_RUNNING.store(false, Ordering::Relaxed);
        return result;
    }

    let stages: [(&str, &str, &[&str]); 3] = [
        ("dismCheck", "dism", &["/Online", "/Cleanup-Image", "/CheckHealth", "/English"]),
        ("dismRestore", "dism", &["/Online", "/Cleanup-Image", "/RestoreHealth", "/English"]),
        ("sfc", "sfc", &["/scannow"]),
    ];

    for (stage, prog, args) in stages.iter() {
        if REPAIR_CANCEL.load(Ordering::Relaxed) {
            result.cancelled = true;
            result.ok = false;
            break;
        }
        let _ = app.emit(
            "repair:progress",
            RepairProgress {
                stage: stage.to_string(),
                percent: 0,
                line: format!("Starting {stage}..."),
            },
        );

        let mut cmd = Command::new(prog);
        for a in *args {
            cmd.arg(a);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                result.ok = false;
                result.log.push_str(&format!("{stage}: failed to start: {e}\n"));
                continue;
            }
        };

        let stdout = child.stdout.take();
        if let Some(out) = stdout {
            let reader = BufReader::new(out);
            for line in reader.lines() {
                if REPAIR_CANCEL.load(Ordering::Relaxed) {
                    let _ = child.kill();
                    result.cancelled = true;
                    break;
                }
                let line = match line {
                    Ok(l) => l,
                    Err(_) => continue,
                };
                // parse percent
                let mut pct = -1i32;
                if let Some(idx) = line.find('%') {
                    // walk back digits
                    let bytes = line.as_bytes();
                    let mut i = idx as isize - 1;
                    let mut num = String::new();
                    while i >= 0 {
                        let c = bytes[i as usize] as char;
                        if c.is_ascii_digit() {
                            num.insert(0, c);
                            i -= 1;
                        } else if c == ' ' || c == '.' {
                            i -= 1;
                            if c == '.' {
                                // stop at decimal point? keep integer part already collected
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if !num.is_empty() {
                        pct = num.parse().unwrap_or(-1);
                        if pct > 100 {
                            pct = 100;
                        }
                    }
                }
                result.log.push_str(&line);
                result.log.push('\n');
                let _ = app.emit(
                    "repair:progress",
                    RepairProgress {
                        stage: stage.to_string(),
                        percent: pct,
                        line: line.clone(),
                    },
                );
            }
        }
        // wait for exit
        let status = child.wait();
        let code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
        if code != 0 {
            result.ok = false;
        }
        if result.cancelled {
            break;
        }
    }

    if result.cancelled {
        result.summary = "cancelled".into();
        result.ok = false;
    } else if result.needs_admin {
        // already set
    } else if result.ok {
        result.summary = "clean".into();
    } else {
        result.summary = "errors".into();
    }
    REPAIR_RUNNING.store(false, Ordering::Relaxed);
    let _ = app.emit("repair:done", result.clone());
    result
}
