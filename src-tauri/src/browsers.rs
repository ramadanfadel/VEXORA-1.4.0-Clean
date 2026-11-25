//! Browser data cleaner: Chrome, Edge, Brave, Opera, Firefox.
//!
//! Only cache / cookies / history / sessions are removed. Passwords and
//! autofill data are NEVER touched.

use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::util;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Browser {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub paths: Vec<String>,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub removed: u64,
    pub freed: u64,
    pub failed: u64,
    pub details: Vec<String>,
}

fn local() -> String { std::env::var("LOCALAPPDATA").unwrap_or_default() }
fn roaming() -> String { std::env::var("APPDATA").unwrap_or_default() }

fn chrome_like_dirs(base: &str) -> Vec<PathBuf> {
    // base = .../User Data
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(base) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                if name == "default" || name.starts_with("profile ") {
                    out.push(p);
                }
            }
        }
    }
    if out.is_empty() {
        out.push(PathBuf::from(base).join("Default"));
    }
    out
}

fn clean_dir_files(dir: &Path, names: &[&str]) -> (u64, u64, u64) {
    let mut removed = 0u64; let mut freed = 0u64; let mut failed = 0u64;
    for name in names {
        let p = dir.join(name);
        if p.is_file() {
            let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
            if util::delete_path(&p) { removed += 1; freed += size; } else { failed += 1; }
        } else if p.is_dir() {
            let size = dir_size(&p);
            if util::delete_path(&p) { removed += 1; freed += size; } else { failed += 1; }
        }
    }
    (removed, freed, failed)
}

fn dir_size(path: &Path) -> u64 {
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

pub fn list() -> Vec<Browser> {
    let mut out = Vec::new();
    let candidates: &[(&str, &str, String)] = &[
        ("chrome", "Google Chrome", format!(r"{}\Google\Chrome\User Data", local())),
        ("edge", "Microsoft Edge", format!(r"{}\Microsoft\Edge\User Data", local())),
        ("brave", "Brave", format!(r"{}\BraveSoftware\Brave-Browser\User Data", local())),
        ("opera", "Opera", format!(r"{}\Opera Software\Opera Stable", roaming())),
    ];
    for (id, name, base) in candidates {
        let installed = Path::new(base).is_dir();
        let mut paths = Vec::new();
        if installed { paths.push(base.clone()); }
        out.push(Browser { id: id.to_string(), name: name.to_string(), installed, paths });
    }
    // Firefox
    let fx_base = format!(r"{}\Mozilla\Firefox\Profiles", roaming());
    let fx_installed = Path::new(&fx_base).is_dir();
    let mut fx_paths = Vec::new();
    if fx_installed {
        if let Ok(rd) = std::fs::read_dir(&fx_base) {
            for e in rd.flatten() {
                if e.path().is_dir() { fx_paths.push(e.path().to_string_lossy().to_string()); }
            }
        }
    }
    out.push(Browser { id: "firefox".into(), name: "Mozilla Firefox".into(), installed: fx_installed, paths: fx_paths });
    out
}

/// categories: "cache", "cookies", "history", "sessions", "siteData"
pub fn clean(id: &str, categories: &[String]) -> CleanResult {
    let mut res = CleanResult::default();
    let cats: std::collections::HashSet<String> = categories.iter().cloned().collect();
    let do_all = cats.is_empty();

    let is = |c: &str| do_all || cats.contains(c);

    match id {
        "chrome" | "edge" | "brave" => {
            let base = match id {
                "chrome" => format!(r"{}\Google\Chrome\User Data", local()),
                "edge" => format!(r"{}\Microsoft\Edge\User Data", local()),
                "brave" => format!(r"{}\BraveSoftware\Brave-Browser\User Data", local()),
                _ => String::new(),
            };
            for profile in chrome_like_dirs(&base) {
                if is("cache") {
                    for sub in ["Cache", "Code Cache", "GPUCache", "Cache_Data", "GrShaderCache", "ShaderCache"] {
                        let p = profile.join(sub);
                        if p.exists() {
                            let sz = dir_size(&p);
                            if util::delete_path(&p) { res.removed += 1; res.freed += sz; res.details.push(format!("cache: {}", p.display())); }
                            else { res.failed += 1; }
                        }
                    }
                }
                if is("cookies") {
                    let (r, f, fl) = clean_dir_files(&profile, &["Cookies", "Cookies-journal", "Network\\Cookies"]);
                    res.removed += r; res.freed += f; res.failed += fl;
                }
                if is("history") {
                    let (r, f, fl) = clean_dir_files(&profile, &["History", "History-journal", "Top Sites", "Top Sites-journal", "Visited Links"]);
                    res.removed += r; res.freed += f; res.failed += fl;
                }
                if is("sessions") {
                    let (r, f, fl) = clean_dir_files(&profile, &["Current Session", "Current Tabs", "Last Session", "Last Tabs"]);
                    res.removed += r; res.freed += f; res.failed += fl;
                }
                if is("siteData") {
                    for sub in ["Local Storage", "IndexedDB", "Service Worker", "File System"] {
                        let p = profile.join(sub);
                        if p.exists() {
                            let sz = dir_size(&p);
                            if util::delete_path(&p) { res.removed += 1; res.freed += sz; }
                            else { res.failed += 1; }
                        }
                    }
                }
            }
        }
        "opera" => {
            let base = format!(r"{}\Opera Software\Opera Stable", roaming());
            let p = PathBuf::from(&base);
            if is("cache") {
                for sub in ["Cache", "Code Cache", "GPUCache", "ShaderCache"] {
                    let pp = p.join(sub);
                    if pp.exists() { let sz = dir_size(&pp); if util::delete_path(&pp) { res.removed+=1; res.freed+=sz; } else { res.failed+=1; } }
                }
            }
            if is("cookies") { let (r,f,fl)=clean_dir_files(&p, &["Cookies","Cookies-journal"]); res.removed+=r; res.freed+=f; res.failed+=fl; }
            if is("history") { let (r,f,fl)=clean_dir_files(&p, &["History","History-journal","Top Sites"]); res.removed+=r; res.freed+=f; res.failed+=fl; }
            if is("sessions") { let (r,f,fl)=clean_dir_files(&p, &["Current Session","Current Tabs","Last Session","Last Tabs"]); res.removed+=r; res.freed+=f; res.failed+=fl; }
        }
        "firefox" => {
            let base = format!(r"{}\Mozilla\Firefox\Profiles", roaming());
            if let Ok(rd) = std::fs::read_dir(&base) {
                for e in rd.flatten() {
                    let profile = e.path();
                    if !profile.is_dir() { continue; }
                    if is("cache") {
                        for sub in ["cache2", "startupCache", "thumbnails"] {
                            let pp = profile.join(sub);
                            if pp.exists() { let sz = dir_size(&pp); if util::delete_path(&pp) { res.removed+=1; res.freed+=sz; } else { res.failed+=1; } }
                        }
                    }
                    if is("cookies") { let (r,f,fl)=clean_dir_files(&profile, &["cookies.sqlite","cookies.sqlite-wal","cookies.sqlite-shm"]); res.removed+=r; res.freed+=f; res.failed+=fl; }
                    if is("history") { let (r,f,fl)=clean_dir_files(&profile, &["places.sqlite","places.sqlite-wal","places.sqlite-shm"]); res.removed+=r; res.freed+=f; res.failed+=fl; }
                    if is("sessions") { let (r,f,fl)=clean_dir_files(&profile, &["sessionstore.jsonlz4","sessionstore-backups"]); res.removed+=r; res.freed+=f; res.failed+=fl; }
                }
            }
        }
        _ => {}
    }
    res
}
