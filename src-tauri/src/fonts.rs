//! Font manager: list, detect broken/duplicate, backup, delete.

use serde::Serialize;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::RegKey;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    pub name: String,
    pub file: String,
    pub path: String,
    pub source: String, // "system" | "user"
    pub exists: bool,
    pub duplicate: bool,
}

pub fn list() -> Vec<FontInfo> {
    use std::collections::HashMap;
    let mut out: Vec<FontInfo> = Vec::new();
    let mut seen_files: HashMap<String, usize> = HashMap::new();

    // System fonts (HKLM)
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts", KEY_READ) {
        for (name, value) in key.enum_values().filter_map(|v| v.ok()) {
            let file = value.to_string();
            let path = resolve_font_path(&file, true);
            let exists = std::path::Path::new(&path).exists();
            out.push(FontInfo { name, file: file.clone(), path: path.clone(), source: "system".into(), exists, duplicate: false });
            let lower = file.to_lowercase();
            *seen_files.entry(lower).or_insert(0) += 1;
        }
    }
    // User fonts (HKCU)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey_with_flags(r"Software\Microsoft\Windows NT\CurrentVersion\Fonts", KEY_READ) {
        for (name, value) in key.enum_values().filter_map(|v| v.ok()) {
            let file = value.to_string();
            let path = resolve_font_path(&file, false);
            let exists = std::path::Path::new(&path).exists();
            out.push(FontInfo { name, file: file.clone(), path: path.clone(), source: "user".into(), exists, duplicate: false });
            let lower = file.to_lowercase();
            *seen_files.entry(lower).or_insert(0) += 1;
        }
    }

    // mark duplicates
    for f in out.iter_mut() {
        if *seen_files.get(&f.file.to_lowercase()).unwrap_or(&0) > 1 {
            f.duplicate = true;
        }
    }
    out
}

fn resolve_font_path(file: &str, system: bool) -> String {
    let f = file.trim();
    if f.contains('\\') || f.contains('/') {
        return f.to_string();
    }
    if system {
        format!(r"C:\Windows\Fonts\{}", f)
    } else {
        // user fonts live in %LOCALAPPDATA%\Microsoft\Windows\Fonts
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            format!(r"{}\Microsoft\Windows\Fonts\{}", local, f)
        } else {
            f.to_string()
        }
    }
}

pub fn backup(font: &FontInfo, dest: &str) -> bool {
    if dest.trim().is_empty() { return false; }
    let _ = std::fs::create_dir_all(dest);
    let src = std::path::Path::new(&font.path);
    if !src.exists() { return false; }
    let fname = src.file_name().unwrap_or_default();
    let target = std::path::Path::new(dest).join(fname);
    std::fs::copy(src, target).is_ok()
}

pub fn delete(font: &FontInfo) -> bool {
    if font.source != "user" {
        return false;
    }
    // Security: never touch protected/system paths.
    let p = std::path::Path::new(&font.path);
    if crate::safety::is_protected(p) || !crate::safety::clean_path_allowed(p) {
        return false;
    }
    // remove registry entry
    let root = if font.source == "system" {
        RegKey::predef(HKEY_LOCAL_MACHINE)
    } else {
        RegKey::predef(HKEY_CURRENT_USER)
    };
    let subkey = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts";
    let user_subkey = r"Software\Microsoft\Windows NT\CurrentVersion\Fonts";
    let path = if font.source == "system" { subkey } else { user_subkey };
    if let Ok(key) = root.open_subkey_with_flags(path, KEY_WRITE) {
        let _ = key.delete_value(&font.name);
    }
    // delete file (best-effort; system fonts may be locked)
    let _ = std::fs::remove_file(&font.path);
    true
}
