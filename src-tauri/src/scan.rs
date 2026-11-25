//! Scan engine: turn cleaner definitions into concrete file lists and sizes.

use std::collections::HashSet;
use std::path::Path;

use serde::Serialize;

use crate::catalog::{Cleaner, Danger, Kind};
use crate::regutil;
use crate::util;

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Clone, Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScanOutcome {
    pub items: Vec<Item>,
    pub size: u64,
    pub count: u64,
    pub truncated: bool,
    pub errors: u64,
    pub note_ar: String,
    pub note_en: String,
}

pub fn glob_match(pattern: &str, text: &str) -> bool {
    let p = pattern.to_lowercase();
    let t = text.to_lowercase();
    let pb = p.as_bytes();
    let tb = t.as_bytes();
    let (mut pi, mut ti) = (0usize, 0usize);
    let (mut star, mut mark) = (usize::MAX, 0usize);
    while ti < tb.len() {
        if pi < pb.len() && (pb[pi] == b'?' || pb[pi] == tb[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < pb.len() && pb[pi] == b'*' {
            star = pi;
            mark = ti;
            pi += 1;
        } else if star != usize::MAX {
            pi = star + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < pb.len() && pb[pi] == b'*' {
        pi += 1;
    }
    pi == pb.len()
}

fn matches_any(patterns: &[String], name: &str) -> bool {
    patterns.iter().any(|p| glob_match(p, name))
}

fn is_excluded(path: &str, exclusions: &[String]) -> bool {
    exclusions.iter().any(|ex| util::path_matches(ex, path))
}

/// Expand a path that may contain wildcards in any component.
fn expand_glob(raw: &str) -> Vec<String> {
    let expanded = util::expand(raw);
    if !expanded.contains('*') && !expanded.contains('?') {
        return vec![expanded];
    }
    let mut current: Vec<String> = vec![String::new()];
    let sep = if expanded.contains('\\') { '\\' } else { '/' };
    let parts: Vec<&str> = expanded.split(sep).filter(|s| !s.is_empty()).collect();
    for (idx, part) in parts.iter().enumerate() {
        let mut next: Vec<String> = Vec::new();
        for base in &current {
            if part.contains('*') || part.contains('?') {
                let dir = if base.is_empty() {
                    String::new()
                } else {
                    base.clone()
                };
                let entries = match std::fs::read_dir(if dir.is_empty() { "." } else { &dir }) {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if glob_match(part, &name) {
                        let joined = if base.is_empty() {
                            name
                        } else {
                            format!("{base}{sep}{name}")
                        };
                        next.push(joined);
                    }
                }
            } else {
                let joined = if base.is_empty() {
                    part.to_string()
                } else {
                    format!("{base}{sep}{part}")
                };
                next.push(joined);
            }
        }
        current = next;
        if current.is_empty() {
            return Vec::new();
        }
        // A drive letter ("C:") needs the separator re-attached on Windows.
        if idx == 0 && current.len() == 1 && current[0].ends_with(':') {
            current[0].push(sep);
        }
    }
    current
}

pub fn scan_files(c: &Cleaner, exclusions: &[String], max_items: usize) -> ScanOutcome {
    let mut out = ScanOutcome::default();
    let mut seen: HashSet<String> = HashSet::new();
    let patterns = if c.patterns.is_empty() {
        vec!["*".to_string()]
    } else {
        c.patterns.clone()
    };

    for raw in &c.paths {
        for path in expand_glob(raw) {
            let p = Path::new(&path);
            let meta = match std::fs::metadata(p) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_file() {
                if is_excluded(&path, exclusions) {
                    continue;
                }
                if !crate::safety::clean_path_allowed(p) {
                    continue;
                }
                let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if !matches_any(&patterns, &name) {
                    continue;
                }
                if seen.insert(path.to_lowercase()) {
                    out.size += meta.len();
                    out.count += 1;
                    push_item(&mut out, &path, meta.len(), false, max_items);
                }
                continue;
            }

            let mut wd = walkdir::WalkDir::new(p).follow_links(false);
            if let Some(d) = c.depth {
                wd = wd.max_depth(d + 1);
            }
            let walker = wd
                .into_iter()
                .filter_entry(|e| !is_excluded(&e.path().to_string_lossy(), exclusions));
            for entry in walker {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => {
                        out.errors += 1;
                        continue;
                    }
                };
                if entry.depth() == 0 {
                    continue;
                }
                if entry.file_type().is_symlink() {
                    continue;
                }
                if !entry.file_type().is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if !matches_any(&patterns, &name) {
                    continue;
                }
                if !crate::safety::clean_path_allowed(entry.path()) {
                    continue;
                }
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let path = entry.path().to_string_lossy().to_string();
                if !seen.insert(path.to_lowercase()) {
                    continue;
                }
                out.size += size;
                out.count += 1;
                push_item(&mut out, &path, size, false, max_items);
            }
        }
    }
    out
}

fn push_item(out: &mut ScanOutcome, path: &str, size: u64, is_dir: bool, max_items: usize) {
    if out.items.len() < max_items {
        out.items.push(Item {
            path: path.to_string(),
            size,
            is_dir,
        });
    } else {
        out.truncated = true;
    }
}

pub fn scan_registry_values(c: &Cleaner) -> ScanOutcome {
    let mut out = ScanOutcome::default();
    let key = match &c.reg_key {
        Some(k) => k.clone(),
        None => return out,
    };
    let pattern = c.reg_pattern.clone().unwrap_or_else(|| "*".into());
    for (name, data) in regutil::enum_values_at(&key) {
        if c.reg_keep.iter().any(|k| k.eq_ignore_ascii_case(&name)) {
            continue;
        }
        if !glob_match(&pattern, &name) {
            continue;
        }
        out.count += 1;
        out.size += (name.len() + data.len()) as u64 * 2;
        out.items.push(Item {
            path: format!(r"{key}\{name}"),
            size: 0,
            is_dir: false,
        });
    }
    out.note_ar = format!("{} قيمة في الريجستري", out.count);
    out.note_en = format!("{} registry values", out.count);
    out
}

pub fn scan_recycle_bin() -> ScanOutcome {
    use windows_sys::Win32::UI::Shell::{SHQueryRecycleBinW, SHQUERYRBINFO};
    let mut info = SHQUERYRBINFO {
        cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32,
        i64Size: 0,
        i64NumItems: 0,
    };
    let rc = unsafe { SHQueryRecycleBinW(std::ptr::null(), &mut info) };
    let mut out = ScanOutcome::default();
    if rc == 0 {
        out.size = info.i64Size.max(0) as u64;
        out.count = info.i64NumItems.max(0) as u64;
        out.note_ar = format!("{} عنصر في السلة", out.count);
        out.note_en = format!("{} items in the bin", out.count);
    }
    out
}

pub fn scan_one(c: &Cleaner, exclusions: &[String], max_items: usize) -> ScanOutcome {
    match c.kind {
        Kind::Files => scan_files(c, exclusions, max_items),
        Kind::RegistryValues => scan_registry_values(c),
        Kind::Recycle => scan_recycle_bin(),
        Kind::Command => {
            let mut o = ScanOutcome::default();
            o.note_ar = "سيتم تنفيذ الأمر عند التنظيف".into();
            o.note_en = "Command runs when cleaning".into();
            o
        }
        Kind::Clipboard => {
            let mut o = ScanOutcome::default();
            o.note_ar = "سيتم إفراغ الحافظة".into();
            o.note_en = "Clipboard will be cleared".into();
            o
        }
    }
}

pub fn is_risky(c: &Cleaner) -> bool {
    matches!(c.danger, Danger::Caution | Danger::Advanced)
}
