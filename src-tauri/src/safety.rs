//! Shared safety guards for destructive cleanup operations.
//!
//! Every destructive path in VEXORA (forced uninstall, installation monitor)
//! must pass through these checks. The frontend is never trusted to decide
//! what is safe — selection is validated again here before any deletion.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// Leaf folder names that must never be deleted wholesale: they are shared
/// containers used by Windows or by many applications at once.
const PROTECTED_NAMES: &[&str] = &[
    "windows", "system32", "syswow64", "winsxs", "systemapps", "servicing",
    "program files", "program files (x86)", "programdata", "users",
    "documents and settings", "appdata", "local", "roaming", "temp", "tmp",
    "microsoft", "common files", "installer", "packages", "cache", "caches",
    "logs", "download", "downloads", "desktop", "documents", "pictures",
    "music", "videos", "fonts", "assembly", "shell", "store", "bin", "lib",
    "include", "share", "node_modules", ".git", ".vs", ".idea", "recovery",
    "boot", "efi", "perflogs", "recycler", "$recycle.bin",
    "system volume information", "vexora", "google", "adobe", "mozilla",
    "oracle", "apple", "nvidia", "intel", "autodesk", "samsung", "sony",
    "lenovo", "asus", "dell",
];

/// Vendor folders are shared containers: never delete them wholesale, but it
/// is safe to inspect one level inside for an app-specific subfolder.
pub const VENDOR_CONTAINERS: &[&str] = &[
    "google", "microsoft", "adobe", "mozilla", "oracle", "apple", "nvidia",
    "intel", "amd", "autodesk", "samsung", "sony", "lenovo", "asus", "dell",
    "hp", "canonical", "docker", "steam", "discord",
];

/// Tokens too generic to justify deleting a directory.
const GENERIC_TOKENS: &[&str] = &[
    "data", "info", "apps", "app", "core", "file", "files", "update", "updates",
    "setup", "install", "installer", "package", "packages", "cache", "temp",
    "tmp", "log", "logs", "common", "shared", "support", "tools", "tool",
    "service", "services", "driver", "drivers", "network", "system", "user",
    "users", "program", "software", "client", "server", "desktop", "portal",
    "manager", "helper", "platform", "framework", "runtime", "library",
    "module", "plugin", "plugins", "extension", "extensions", "bin", "lib",
    "windows", "microsoft", "corporation", "company", "version", "edition",
];

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CleanupItem {
    pub kind: String,     // "directory" | "registry"
    pub path: String,
    pub size: u64,
    pub safe: bool,
    pub reason: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlan {
    pub items: Vec<CleanupItem>,
    pub safe_count: usize,
    pub blocked_count: usize,
}

impl CleanupPlan {
    pub fn finish(mut self) -> Self {
        self.safe_count = self.items.iter().filter(|i| i.safe).count();
        self.blocked_count = self.items.len() - self.safe_count;
        self
    }
    pub fn safe_paths(&self) -> Vec<String> {
        self.items.iter().filter(|i| i.safe).map(|i| i.path.clone()).collect()
    }
}

fn norm(p: &Path) -> String {
    p.to_string_lossy().replace('/', "\\").trim_end_matches('\\').to_lowercase()
}

/// A leaf name in this list is a shared container — never delete it.
pub fn is_protected(path: &Path) -> bool {
    let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    if name.is_empty() || PROTECTED_NAMES.contains(&name.as_str()) {
        return true;
    }
    let full = norm(path);
    if path.parent().is_none() {
        return true;
    }
    let win = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into()).to_lowercase();
    if full == win.replace('/', "\\").trim_end_matches('\\') || full.starts_with(&(win + "\\")) {
        return true;
    }
    false
}

/// Narrower guard for general cleanup: only hard-critical roots are blocked,
/// so legitimate temp/cache cleaning inside Windows folders still works.
const CRITICAL_ROOTS: &[&str] = &[
    "windows", "system32", "syswow64", "winsxs", "program files",
    "program files (x86)", "programdata", "users", "documents and settings",
    "boot", "efi", "recovery", "$recycle.bin", "system volume information",
    "vexora",
];

pub fn is_critical_root(path: &Path) -> bool {
    let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    name.is_empty() || CRITICAL_ROOTS.contains(&name.as_str())
}

/// Personal folders that must never be deleted, neither as the target itself
/// nor as any ancestor of it. This implements the project rule: the cleaner
/// never touches Documents / Desktop / Pictures / Videos / Music / Downloads.
const USER_DATA_FOLDERS: &[&str] = &[
    "documents", "desktop", "pictures", "videos", "music", "downloads",
    "contacts", "favorites", "links", "searches", "saved games",
    "recycler", "$recycle.bin", "system volume information",
];

/// Sync-client roots are personal data as a whole: never delete the root
/// itself, but their local app caches (e.g. `Local\Microsoft\OneDrive\logs`)
/// remain cleanable.
const SYNC_ROOTS: &[&str] = &["onedrive", "dropbox"];

/// Guard for the normal cleaning path, applied to both the preview scan and
/// the execution so they always agree. Blocks critical system roots, personal
/// data folders (as the folder itself or as any ancestor) and VEXORA's own
/// data folder — while still allowing temp/cache/log files inside system
/// folders like `C:\Windows\Temp` or `%LOCALAPPDATA%\Temp`.
pub fn clean_path_allowed(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if name.is_empty() || name == "vexora" {
        return false;
    }
    if CRITICAL_ROOTS.contains(&name.as_str())
        || USER_DATA_FOLDERS.contains(&name.as_str())
        || SYNC_ROOTS.contains(&name.as_str())
    {
        return false;
    }
    for comp in path.components() {
        let c = comp.as_os_str().to_string_lossy().to_lowercase();
        if USER_DATA_FOLDERS.contains(&c.as_str()) || c == "vexora" {
            return false;
        }
    }
    true
}

/// Centralized path validation for deletion operations.
/// Canonicalizes (resolves symlinks/junctions), rejects traversal, reparse
/// points, protected/critical roots and personal data folders.
pub fn validate_path_for_delete(path: &str) -> Result<std::path::PathBuf, String> {
    if path.trim().is_empty() {
        return Err("empty path".into());
    }
    if path.contains('\0') {
        return Err("invalid path".into());
    }
    let raw = std::path::Path::new(path);
    for comp in raw.components() {
        if let std::path::Component::ParentDir = comp {
            return Err("path traversal not allowed".into());
        }
    }
    if is_reparse_point(raw) {
        return Err("reparse point (junction/symlink) not allowed".into());
    }
    let canon = raw.canonicalize().map_err(|e| format!("cannot resolve path: {e}"))?;
    if is_reparse_point(&canon) {
        return Err("reparse point target not allowed".into());
    }
    if is_protected(&canon) {
        return Err("protected path".into());
    }
    if !clean_path_allowed(&canon) {
        return Err("path not allowed for deletion".into());
    }
    Ok(canon)
}

/// Centralized path validation for read/scan operations.
pub fn validate_path_for_read(path: &str) -> Result<std::path::PathBuf, String> {
    if path.trim().is_empty() {
        return Err("empty path".into());
    }
    if path.contains('\0') {
        return Err("invalid path".into());
    }
    let raw = std::path::Path::new(path);
    for comp in raw.components() {
        if let std::path::Component::ParentDir = comp {
            return Err("path traversal not allowed".into());
        }
    }
    raw.canonicalize().map_err(|e| format!("cannot resolve path: {e}"))
}

/// Junctions/symlinks can point anywhere; `remove_dir_all` may follow them.
pub fn is_reparse_point(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    match std::fs::symlink_metadata(path) {
        Ok(m) => m.file_attributes() & 0x400 != 0, // FILE_ATTRIBUTE_REPARSE_POINT
        Err(_) => false,
    }
}

/// Word-boundary aware match. "code" matches "Visual Studio Code" but not "Knowledge".
pub fn name_matches(dir_name: &str, tokens: &[String]) -> bool {
    let segments: Vec<String> = dir_name
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    tokens.iter().any(|t| {
        if t.len() < 4 || GENERIC_TOKENS.contains(&t.as_str()) {
            return false;
        }
        segments.iter().any(|s| {
            s == t || s.trim_end_matches(|c: char| c.is_ascii_digit()) == t
        })
    })
}

/// True when deleting `candidate` would remove a location belonging to a
/// *different* still-installed program (ancestor or descendant overlap).
pub fn owned_by_other_program(candidate: &Path, other_locations: &[String]) -> bool {
    let c = norm(candidate);
    if c.is_empty() {
        return false;
    }
    for loc in other_locations {
        let l = norm(Path::new(loc));
        if l.is_empty() || l == c {
            continue;
        }
        if l.starts_with(&(c.clone() + "\\")) || c.starts_with(&(l.clone() + "\\")) {
            return true;
        }
    }
    false
}

/// True when a candidate folder directly contains a subdirectory belonging to
/// another installed program (e.g. `Program Files\Google` holds Chrome + Earth).
pub fn contains_other_program(candidate: &Path, other_tokens: &[Vec<String>]) -> bool {
    if other_tokens.is_empty() {
        return false;
    }
    let rd = match std::fs::read_dir(candidate) {
        Ok(r) => r,
        Err(_) => return false,
    };
    for entry in rd.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let child = entry.file_name().to_string_lossy().to_string();
        if other_tokens.iter().any(|toks| name_matches(&child, toks)) {
            return true;
        }
    }
    false
}

pub fn quarantine_dir() -> PathBuf {
    crate::util::data_dir().join("Quarantine").join(crate::util::now_unix().to_string())
}

/// Move a path into quarantine instead of destroying it. Same-volume rename is
/// instant; cross-volume moves are refused so nothing is lost.
pub fn quarantine(path: &Path) -> Result<PathBuf, String> {
    let root = quarantine_dir();
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let name = path.file_name().ok_or_else(|| "invalid path".to_string())?;
    let mut dest = root.join(name);
    if dest.exists() {
        dest = root.join(format!("{}_{}", crate::util::now_unix(), name.to_string_lossy()));
    }
    std::fs::rename(path, &dest).map_err(|e| format!("{}: {}", path.display(), e))?;
    // Remember the original location so the item can be restored later.
    let manifest = root.join("manifest.json");
    let mut map: std::collections::HashMap<String, String> = std::fs::read_to_string(&manifest)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    map.insert(dest.file_name().unwrap_or_default().to_string_lossy().to_string(), path.to_string_lossy().to_string());
    if let Ok(text) = serde_json::to_string_pretty(&map) {
        let _ = std::fs::write(&manifest, text);
    }
    Ok(dest)
}

/// Restore everything currently in quarantine (folders + exported registry keys).
pub fn quarantine_restore() -> Result<usize, String> {
    let root = crate::util::data_dir().join("Quarantine");
    let mut restored = 0usize;
    let batches = match std::fs::read_dir(&root) { Ok(r) => r, Err(_) => return Ok(0) };
    for batch in batches.flatten() {
        let dir = batch.path();
        if !dir.is_dir() { continue; }
        let manifest = dir.join("manifest.json");
        if let Ok(text) = std::fs::read_to_string(&manifest) {
            if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, String>>(&text) {
                for (stored, original) in map {
                    let src = dir.join(&stored);
                    if !src.exists() { continue; }
                    let dest = std::path::Path::new(&original);
                    if let Some(parent) = dest.parent() { let _ = std::fs::create_dir_all(parent); }
                    if std::fs::rename(&src, dest).is_ok() { restored += 1; }
                }
            }
        }
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.extension().map(|e| e == "reg").unwrap_or(false) {
                    let _ = crate::util::run_and_wait_vec("reg", &vec!["import".to_string(), p.to_string_lossy().to_string()]);
                    restored += 1;
                }
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
    Ok(restored)
}

/// Permanently empty the quarantine folder.
pub fn quarantine_empty() -> Result<usize, String> {
    let root = crate::util::data_dir().join("Quarantine");
    let mut n = 0usize;
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() { if e.path().is_dir() { n += 1; } }
    }
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::create_dir_all(&root);
    Ok(n)
}

/// Export a registry key to .reg before deleting it, so it can be restored.
pub fn backup_registry_key(key: &str) -> Result<PathBuf, String> {
    let root = quarantine_dir();
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let leaf = key.rsplit('\\').next().unwrap_or("key");
    let safe: String = leaf.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
    let file = root.join(format!("{}.reg", safe));
    crate::util::run_and_wait_vec(
        "reg",
        &vec!["export".to_string(), key.to_string(), file.to_string_lossy().to_string(), "/y".to_string()],
    )?;
    Ok(file)
}

/// Validate a path reported by the frontend before deletion.
pub fn validate_selected(path: &str, plan: &CleanupPlan) -> bool {
    plan.items.iter().any(|i| i.safe && i.path == path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_personal_folders_and_theirs_inside() {
        assert!(!clean_path_allowed(Path::new(r"C:\Users\me\Documents\file.txt")));
        assert!(!clean_path_allowed(Path::new(r"C:\Users\me\Desktop")));
        assert!(!clean_path_allowed(Path::new(r"C:\Users\me\Videos")));
        assert!(!clean_path_allowed(Path::new(r"C:\Users\me\Pictures\pic.jpg")));
        assert!(!clean_path_allowed(Path::new(r"D:\Downloads\setup.exe")));
        assert!(!clean_path_allowed(Path::new(r"C:\Users\me\Music")));
        assert!(!clean_path_allowed(Path::new(r"C:\Users\me\OneDrive\Documents\a.txt")));
    }

    #[test]
    fn allows_legitimate_temp_and_cache_paths() {
        assert!(clean_path_allowed(Path::new(
            r"C:\Users\me\AppData\Local\Temp\a.tmp"
        )));
        assert!(clean_path_allowed(Path::new(
            r"C:\Windows\SoftwareDistribution\Download\setup.exe"
        )));
        assert!(clean_path_allowed(Path::new(r"C:\Windows\Temp")));
        assert!(clean_path_allowed(Path::new(r"C:\ProgramData\Temp\b.log")));
        assert!(clean_path_allowed(Path::new(
            r"C:\Users\me\AppData\Local\Microsoft\OneDrive\logs\x.log"
        )));
    }

    #[test]
    fn blocks_critical_roots_and_vexora_data() {
        assert!(!clean_path_allowed(Path::new(r"C:\Windows")));
        assert!(!clean_path_allowed(Path::new(r"C:\Program Files")));
        assert!(!clean_path_allowed(Path::new(r"C:\Users\me\AppData\Roaming\VEXORA\log.txt")));
        assert!(!clean_path_allowed(Path::new(r"C:\$Recycle.bin")));
    }

    #[test]
    fn validate_rejects_empty_and_traversal() {
        assert!(validate_path_for_delete("").is_err());
        assert!(validate_path_for_read("").is_err());
        assert!(validate_path_for_delete("C:\\Windows\\..\\secret").is_err());
        assert!(validate_path_for_read("C:\\Windows\\..\\secret").is_err());
        assert!(validate_path_for_delete("C:\\foo\0bar").is_err());
    }

    #[test]
    fn is_protected_blocks_windows() {
        let win = Path::new("C:\\Windows");
        assert!(is_protected(win));
    }

    #[test]
    fn sanitize_settings_clamps() {
        let mut s = crate::settings::Settings::default();
        s.shred_passes = 99;
        s.bg_monitor_interval_min = 0;
        s.bg_monitor_mem_threshold = 200;
        s.widget_opacity = 5.0;
        s.exclusions = vec!["".into(), "  ".into(), "C:\\ok".into(), "C:\\a\\..\\b".into()];
        s.sanitize();
        assert_eq!(s.shred_passes, 3);
        assert_eq!(s.bg_monitor_interval_min, 1);
        assert_eq!(s.bg_monitor_mem_threshold, 100);
        assert!((s.widget_opacity - 1.0).abs() < 1e-9);
        assert_eq!(s.exclusions, vec!["C:\\ok".to_string()]);
    }
}
