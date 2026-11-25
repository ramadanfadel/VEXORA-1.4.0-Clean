//! Junk file cleaner: temp files, logs, dumps, empty folders.

use serde::Serialize;
use std::path::PathBuf;

use crate::util;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JunkItem {
    pub path: String,
    pub kind: String,
    pub size: u64,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub items: Vec<JunkItem>,
    pub total_size: u64,
    pub scanned: u64,
}

const JUNK_EXTS: &[&str] = &[
    "tmp", "temp", "log", "old", "bak", "dmp", "chk", "gid", "fts", "prv", "err",
];

fn is_junk_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower == "thumbs.db" || lower == "desktop.ini" || lower == "ehthumbs.db" {
        return true;
    }
    if let Some(ext) = lower.rsplit('.').next() {
        if ext != lower && JUNK_EXTS.contains(&ext) {
            return true;
        }
    }
    false
}

fn roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(t) = std::env::var("TEMP") { out.push(PathBuf::from(t)); }
    if let Ok(t) = std::env::var("TMP") { out.push(PathBuf::from(t)); }
    let la = util::local_appdata();
    if !la.is_empty() { out.push(PathBuf::from(la).join("Temp")); }
    let win = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    out.push(PathBuf::from(&win).join("Temp"));
    out.push(PathBuf::from(&win).join("Prefetch"));
    out.push(PathBuf::from(&win).join("Logs"));
    if let Ok(pd) = std::env::var("ProgramData") { out.push(PathBuf::from(pd)); }
    out
}

pub fn scan(limit: usize) -> ScanResult {
    let s = crate::settings::Settings::load();
    let mut res = ScanResult::default();
    let mut seen = std::collections::HashSet::new();
    for root in roots() {
        if !root.is_dir() { continue; }
        let mut stack = vec![root];
        let mut guard = 0;
        while let Some(dir) = stack.pop() {
            guard += 1;
            if guard > 200_000 { break; }
            let rd = match std::fs::read_dir(&dir) { Ok(r) => r, Err(_) => continue };
            for entry in rd.flatten() {
                let p = entry.path();
                let meta = match entry.metadata() { Ok(m) => m, Err(_) => continue };
                res.scanned += 1;
                if meta.is_dir() {
                    // empty folder?
                    if std::fs::read_dir(&p).map(|mut r| r.next().is_none()).unwrap_or(false) {
                        let key = p.to_string_lossy().to_lowercase();
                        if seen.insert(key) && allowed(&s, &p) {
                            res.items.push(JunkItem { path: p.to_string_lossy().to_string(), kind: "emptyFolder".into(), size: 0 });
                            res.total_size += 0;
                            if res.items.len() >= limit { return res; }
                        }
                    }
                    // don't descend into huge system dirs beyond depth; push
                    let depth = p.components().count();
                    if depth < 8 { stack.push(p); }
                } else {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if is_junk_name(&name) {
                        let key = p.to_string_lossy().to_lowercase();
                        if seen.insert(key) && allowed(&s, &p) {
                            res.items.push(JunkItem { path: p.to_string_lossy().to_string(), kind: "junkFile".into(), size: meta.len() });
                            res.total_size += meta.len();
                            if res.items.len() >= limit { return res; }
                        }
                    }
                }
            }
        }
    }
    res
}

/// Same rules for preview and execution: user exclusions + protected paths.
fn allowed(s: &crate::settings::Settings, p: &std::path::Path) -> bool {
    if !crate::safety::clean_path_allowed(p) {
        return false;
    }
    let text = p.to_string_lossy();
    !s.exclusions.iter().any(|ex| util::path_matches(ex, &text))
}

pub fn delete(paths: &[String]) -> (u64, u64, u64) {
    let s = crate::settings::Settings::load();
    let mut removed = 0u64; let mut freed = 0u64; let mut failed = 0u64;
    // Re-validate everything: exclusions, protected paths and reparse points.
    let allowed_paths: Vec<&String> = paths
        .iter()
        .filter(|p| {
            let path = std::path::Path::new(p);
            let dir_link = path.is_dir() && crate::safety::is_reparse_point(path);
            allowed(&s, path) && !dir_link
        })
        .collect();
    failed += (paths.len() - allowed_paths.len()) as u64;
    // Measure sizes BEFORE the items are moved to the Recycle Bin — once
    // recycled the paths are gone and metadata() would return nothing.
    let sizes: std::collections::HashMap<&String, u64> = allowed_paths
        .iter()
        .map(|p| {
            let path = std::path::Path::new(p);
            let size = if path.is_dir() {
                dir_size(path)
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            };
            ((*p), size)
        })
        .collect();
    // Recycle only — permanent deletion is not offered here.
    for chunk in allowed_paths.chunks(100) {
        let chunk_refs: Vec<String> = chunk.iter().map(|s| (*s).clone()).collect();
        if util::recycle_batch(&chunk_refs).is_ok() {
            for p in chunk {
                removed += 1;
                freed += sizes.get(*p).copied().unwrap_or(0);
            }
        } else {
            for p in chunk {
                if util::recycle_batch(std::slice::from_ref(*p)).is_ok() {
                    removed += 1;
                    freed += sizes.get(*p).copied().unwrap_or(0);
                } else {
                    failed += 1;
                }
            }
        }
    }
    crate::log::write(
        "junk",
        &format!("إلى السلة: {removed} عنصر ({freed} بايت)، مرفوض: {failed}"),
    );
    (removed, freed, failed)
}

fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64; let mut stack = vec![path.to_path_buf()]; let mut guard = 0;
    while let Some(d) = stack.pop() {
        guard += 1; if guard > 50_000 { break; }
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                let p = e.path();
                if let Ok(m) = e.metadata() {
                    if m.is_dir() { stack.push(p); } else { total += m.len(); }
                }
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_recycles_instead_of_permanent_delete() {
        let dir = std::env::temp_dir().join(format!("vexora_junk_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("vexora_test_file.tmp");
        std::fs::write(&f, b"phase0 test").unwrap();

        let (removed, _freed, failed) = delete(&[f.to_string_lossy().to_string()]);

        assert_eq!(failed, 0, "recycle should succeed for a normal temp file");
        assert_eq!(removed, 1, "file should have been recycled");
        assert!(!f.exists(), "file must leave its folder (moved to the Recycle Bin)");
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn delete_refuses_protected_paths() {
        // Personal folders must be refused even if listed explicitly.
        let (removed, _freed, failed) =
            delete(&[r"C:\Users\Public\Documents\should_not_touch.txt".into()]);
        assert_eq!(removed, 0, "protected path must never be removed");
        assert_eq!(failed, 1, "it must be counted as refused");
    }
}
