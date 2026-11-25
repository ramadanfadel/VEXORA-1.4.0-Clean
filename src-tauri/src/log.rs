//! Append-only operation log so every destructive action can be audited.
//!
//! This is separate from `history.jsonl` (the user-facing activity list):
//! `log.txt` is a plain-text trail meant for support and debugging.

use std::io::Write;
use std::path::PathBuf;

fn path() -> PathBuf {
    crate::util::data_dir().join("log.txt")
}

fn fmt_ts(ts: u64) -> String {
    // UTC civil date (Howard Hinnant's algorithm) — same as history.rs.
    let days = (ts / 86_400) as i64;
    let secs = ts % 86_400;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60
    )
}

/// Append one line: `2026-11-23 14:05 [clean] message`. Rotates at 1 MB.
pub fn write(area: &str, message: &str) {
    let p = path();
    if let Ok(meta) = std::fs::metadata(&p) {
        if meta.len() > 1_048_576 {
            let _ = std::fs::rename(&p, p.with_extension("txt.1"));
        }
    }
    let line = format!(
        "{} [{}] {}\n",
        fmt_ts(crate::util::now_unix()),
        area,
        message
    );
    if let Ok(mut fh) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
    {
        let _ = fh.write_all(line.as_bytes());
    }
}
