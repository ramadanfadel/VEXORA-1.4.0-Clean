//! Cleaning history (JSON lines).

use serde::{Deserialize, Serialize};

use crate::util;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub ts: u64,
    pub kind: String,
    pub title: String,
    pub detail: String,
    pub size: u64,
    pub items: u64,
}

fn path() -> std::path::PathBuf {
    util::data_dir().join("history.jsonl")
}

pub fn add(kind: &str, title: &str, detail: &str, size: u64, items: u64) {
    let entry = Entry {
        ts: util::now_unix(),
        kind: kind.into(),
        title: title.into(),
        detail: detail.into(),
        size,
        items,
    };
    if let Ok(line) = serde_json::to_string(&entry) {
        use std::io::Write;
        if let Ok(mut fh) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path())
        {
            let _ = writeln!(fh, "{line}");
        }
    }
}

pub fn list(limit: usize) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    if let Ok(text) = std::fs::read_to_string(path()) {
        for line in text.lines().rev() {
            if out.len() >= limit {
                break;
            }
            if let Ok(e) = serde_json::from_str::<Entry>(line) {
                out.push(e);
            }
        }
    }
    out
}

pub fn clear() -> Result<(), String> {
    let p = path();
    // Truncate in place; propagate errors so a locked/readonly file is
    // reported to the UI instead of failing silently.
    std::fs::write(&p, "").map_err(|e| format!("{}: {e}", p.display()))
}

fn fmt_ts(ts: u64) -> String {
    // UTC civil date (Howard Hinnant's algorithm).
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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Full history, oldest first.
pub fn all() -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    if let Ok(text) = std::fs::read_to_string(path()) {
        for line in text.lines() {
            if let Ok(e) = serde_json::from_str::<Entry>(line) {
                out.push(e);
            }
        }
    }
    out
}

pub fn export_csv(dest: &str) -> Result<(), String> {
    let mut text = String::from("\u{FEFF}Date (UTC),Type,Title,Detail,Size (bytes),Items\n");
    for e in all() {
        let clean = |v: &str| v.replace('"', "\"\"");
        text.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",{},{}\n",
            fmt_ts(e.ts),
            clean(&e.kind),
            clean(&e.title),
            clean(&e.detail),
            e.size,
            e.items
        ));
    }
    std::fs::write(dest, text).map_err(|e| e.to_string())
}

pub fn export_html(dest: &str) -> Result<(), String> {
    let (total_size, total_items) = totals();
    let mut rows = String::new();
    for e in all().into_iter().rev() {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            fmt_ts(e.ts),
            html_escape(&e.kind),
            html_escape(&e.title),
            html_escape(&e.detail),
            crate::util::fmt_bytes(e.size),
            e.items
        ));
    }
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="ar" dir="rtl"><head><meta charset="utf-8" />
<title>VEXORA — تقرير التنظيف</title>
<style>
body{{font-family:"Segoe UI",Tahoma,sans-serif;background:#f5f8fc;color:#0f1b2d;margin:0;padding:32px}}
.wrap{{max-width:1000px;margin:0 auto;background:#fff;border-radius:16px;padding:28px;box-shadow:0 10px 30px rgba(15,27,45,.08)}}
h1{{margin:0;font-size:24px}} .sub{{color:#5f7186;margin:6px 0 22px}}
.cards{{display:flex;gap:14px;flex-wrap:wrap;margin-bottom:22px}}
.card{{flex:1;min-width:180px;background:#eaf6fb;border-radius:12px;padding:14px}}
.card b{{display:block;font-size:20px;color:#0891b2}}
table{{width:100%;border-collapse:collapse;font-size:13px}}
th,td{{padding:9px 10px;border-bottom:1px solid #e3ecf5;text-align:start}}
th{{background:#f0f6fc;color:#38506b}}
.foot{{margin-top:22px;color:#5f7186;font-size:12px;border-top:1px solid #e3ecf5;padding-top:14px}}
</style></head><body><div class="wrap">
<h1>VEXORA — تقرير التنظيف والصيانة</h1>
<div class="sub">Clean Smarter. Run Faster. — تنظيف أذكى، وأداء أسرع</div>
<div class="cards">
<div class="card">إجمالي المساحة المحررة<b>{}</b></div>
<div class="card">إجمالي العناصر<b>{}</b></div>
<div class="card">عدد العمليات<b>{}</b></div>
</div>
<table><thead><tr><th>التاريخ (UTC)</th><th>النوع</th><th>العملية</th><th>التفاصيل</th><th>الحجم</th><th>العناصر</th></tr></thead>
<tbody>{}</tbody></table>
<div class="foot">تصميم وإعداد: أ. رمضان فاضل مصطفى (أبو مريم)<br/>الإصدار {}</div>
</div></body></html>"#,
        crate::util::fmt_bytes(total_size),
        total_items,
        all().len(),
        rows,
        env!("CARGO_PKG_VERSION")
    );
    std::fs::write(dest, html).map_err(|e| e.to_string())
}

pub fn totals() -> (u64, u64) {
    let mut size = 0u64;
    let mut items = 0u64;
    if let Ok(text) = std::fs::read_to_string(path()) {
        for line in text.lines() {
            if let Ok(e) = serde_json::from_str::<Entry>(line) {
                size += e.size;
                items += e.items;
            }
        }
    }
    (size, items)
}
