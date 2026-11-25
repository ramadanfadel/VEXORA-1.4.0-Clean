//! Installation monitor: snapshot → install → diff → uninstall.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::safety;
use crate::uninstall;
use crate::util;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub kind: String, // "registryKey", "registryValue", "file", "directory"
    pub path: String,
    pub detail: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallLog {
    pub name: String,
    pub ts: u64,
    pub changes: Vec<Change>,
}

fn log_dir() -> PathBuf {
    let p = util::data_dir().join("installs");
    let _ = std::fs::create_dir_all(&p);
    p
}

fn snapshot_registry() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let roots = [
        r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    ];
    for root in roots {
        collect_reg_keys(root, &mut out, 2);
    }
    // Also snapshot HKCU\Software and HKLM\SOFTWARE top-level keys
    for root in [r"HKEY_CURRENT_USER\Software", r"HKEY_LOCAL_MACHINE\SOFTWARE"] {
        if let Some(key) = crate::regutil::open(root) {
            for sub in key.enum_keys().flatten() {
                out.insert(format!(r"{}\{}", root, sub));
            }
        }
    }
    out
}

fn collect_reg_keys(root: &str, out: &mut BTreeSet<String>, depth: u32) {
    if depth == 0 { return; }
    if let Some(key) = crate::regutil::open(root) {
        for sub in key.enum_keys().flatten() {
            let full = format!(r"{}\{}", root, sub);
            out.insert(full.clone());
            collect_reg_keys(&full, out, depth - 1);
        }
    }
}

fn snapshot_files() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let roots = [
        std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into()),
        std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into()),
        util::local_appdata(),
        util::program_data(),
    ];
    for root in roots {
        if root.is_empty() { continue; }
        let rd = match std::fs::read_dir(&root) { Ok(r) => r, Err(_) => continue };
        for entry in rd.flatten() {
            if entry.path().is_dir() {
                out.insert(entry.path().to_string_lossy().to_lowercase());
            }
        }
    }
    out
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Snapshot {
    registry: BTreeSet<String>,
    files: BTreeSet<String>,
}

fn snapshot_path(name: &str) -> PathBuf {
    log_dir().join(format!("{}.snapshot.json", sanitize(name)))
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect()
}

pub fn start(name: &str) -> Result<(), String> {
    if name.trim().is_empty() { return Err("name required".into()); }
    let snap = Snapshot { registry: snapshot_registry(), files: snapshot_files() };
    let text = serde_json::to_string(&snap).map_err(|e| e.to_string())?;
    std::fs::write(snapshot_path(name), text).map_err(|e| e.to_string())
}

pub fn finish(name: &str) -> Result<InstallLog, String> {
    if name.trim().is_empty() { return Err("name required".into()); }
    let snap_path = snapshot_path(name);
    let old_text = std::fs::read_to_string(&snap_path).map_err(|e| format!("no snapshot: {e}"))?;
    let old: Snapshot = serde_json::from_str(&old_text).map_err(|e| e.to_string())?;
    let new = Snapshot { registry: snapshot_registry(), files: snapshot_files() };

    let mut changes = Vec::new();
    // Registry keys added
    for key in new.registry.difference(&old.registry) {
        changes.push(Change { kind: "registryKey".into(), path: key.clone(), detail: "added".into() });
    }
    // Files/dirs added
    for f in new.files.difference(&old.files) {
        changes.push(Change { kind: "directory".into(), path: f.clone(), detail: "added".into() });
    }

    let log = InstallLog { name: name.to_string(), ts: util::now_unix(), changes };
    let text = serde_json::to_string_pretty(&log).map_err(|e| e.to_string())?;
    std::fs::write(log_dir().join(format!("{}.json", sanitize(name))), text).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&snap_path);
    Ok(log)
}

pub fn list() -> Vec<InstallLog> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(log_dir()) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().map(|x| x == "json").unwrap_or(false) && !p.to_string_lossy().ends_with(".snapshot.json") {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    if let Ok(log) = serde_json::from_str::<InstallLog>(&text) {
                        out.push(log);
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| b.ts.cmp(&a.ts));
    out
}

/// Build a reviewable plan from a recorded install log. Nothing is deleted.
pub fn uninstall_plan(name: &str) -> Result<safety::CleanupPlan, String> {
    let path = log_dir().join(format!("{}.json", sanitize(name)));
    let text = std::fs::read_to_string(&path).map_err(|e| format!("no log: {e}"))?;
    let log: InstallLog = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    let all = uninstall::list();
    let other_locations: Vec<String> = all.iter().map(|p| p.location.clone()).filter(|s| !s.trim().is_empty()).collect();
    let other_tokens: Vec<Vec<String>> = all.iter().map(|p| uninstall::tokens(&p.name)).filter(|t| !t.is_empty()).collect();

    let mut items: Vec<safety::CleanupItem> = Vec::new();
    let mut dirs: Vec<String> = log
        .changes
        .iter()
        .filter(|c| c.kind == "directory")
        .map(|c| c.path.clone())
        .collect();
    dirs.sort_by(|a, b| b.len().cmp(&a.len())); // deepest first
    for d in dirs {
        let p = std::path::Path::new(&d);
        if !p.is_dir() { continue; }
        let (safe, reason) = if safety::is_protected(p) {
            (false, "protected")
        } else if safety::is_reparse_point(p) {
            (false, "reparse")
        } else if safety::owned_by_other_program(p, &other_locations) {
            (false, "otherProgram")
        } else if safety::contains_other_program(p, &other_tokens) {
            (false, "containsOther")
        } else {
            (true, "match")
        };
        let size: u64 = walkdir::WalkDir::new(p)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok().map(|m| m.len()))
            .sum();
        items.push(safety::CleanupItem {
            kind: "directory".into(),
            path: d,
            size,
            safe,
            reason: reason.into(),
        });
    }
    for c in &log.changes {
        if c.kind != "registryKey" { continue; }
        let lower = c.path.to_lowercase();
        let safe = !lower.contains("\\microsoft\\windows\\")
            && !lower.contains("\\system32")
            && !lower.contains("\\winsxs");
        items.push(safety::CleanupItem {
            kind: "registry".into(),
            path: c.path.clone(),
            size: 0,
            safe,
            reason: if safe { "match" } else { "protected" }.into(),
        });
    }
    Ok(safety::CleanupPlan { items, safe_count: 0, blocked_count: 0 }.finish())
}

/// Delete only reviewed paths, re-validated against the plan.
pub fn uninstall_selected(name: &str, paths: &[String]) -> Result<usize, String> {
    let plan = uninstall_plan(name)?;
    let mut removed = 0usize;
    for p in paths {
        if !safety::validate_selected(p, &plan) { continue; }
        let item = match plan.items.iter().find(|i| i.path == *p) { Some(i) => i, None => continue };
        if item.kind == "registry" {
            let _ = safety::backup_registry_key(p);
            if let Some((parent, leaf)) = p.rsplit_once('\\') {
                if crate::regutil::delete_subkey(parent, leaf) { removed += 1; }
            }
        } else {
            let path = std::path::Path::new(p);
            if path.is_dir() && safety::quarantine(path).is_ok() { removed += 1; }
        }
    }
    Ok(removed)
}

/// Backwards-compatible entry point: only removes safety-approved items.
pub fn uninstall(name: &str) -> Result<usize, String> {
    let plan = uninstall_plan(name)?;
    let paths = plan.safe_paths();
    uninstall_selected(name, &paths)
}
