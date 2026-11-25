//! Registry cleaner: detection of stale / invalid entries, backups and repair.

use serde::{Deserialize, Serialize};

use crate::regutil::{self, Hive};
use crate::util;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: usize,
    /// "value" or "key"
    pub action: String,
    pub key_path: String,
    pub name: String,
    pub title_ar: String,
    pub title_en: String,
    pub detail: String,
    pub advanced: bool,
}

fn exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

/// Pull the executable out of a command line such as
/// `"C:\Program Files\App\app.exe" /background` or `C:\App\app.exe -x`.
pub fn command_exe(cmd: &str) -> String {
    let s = cmd.trim();
    if s.is_empty() {
        return String::new();
    }
    if let Some(rest) = s.strip_prefix('"') {
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    let lower = s.to_lowercase();
    for ext in [".exe", ".com", ".bat", ".cmd", ".dll", ".scr"] {
        if let Some(pos) = lower.find(ext) {
            let mut end = pos + ext.len();
            if end <= s.len() && s.is_char_boundary(end) {
                return s[..end].to_string();
            }
            if end > s.len() {
                end = s.len();
            }
            return s[..end].to_string();
        }
    }
    s.split_whitespace().next().unwrap_or("").to_string()
}

fn uninstall_roots() -> Vec<String> {
    vec![
        r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall".into(),
        r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall".into(),
        r"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall".into(),
    ]
}

pub fn scan() -> Vec<Issue> {
    let mut issues: Vec<Issue> = Vec::new();
    let mut next_id = 0usize;
    let mut push = |issues: &mut Vec<Issue>,
                    action: &str,
                    key_path: String,
                    name: String,
                    tar: String,
                    ten: String,
                    detail: String,
                    advanced: bool| {
        issues.push(Issue {
            id: next_id,
            action: action.into(),
            key_path,
            name,
            title_ar: tar,
            title_en: ten,
            detail,
            advanced,
        });
        next_id += 1;
    };

    // 1) Uninstall entries pointing at missing programs -------------------
    for root in uninstall_roots() {
        let key = match regutil::open(&root) {
            Some(k) => k,
            None => continue,
        };
        for sub in key.enum_keys().flatten() {
            let full = format!(r"{root}\{sub}");
            let sk = match regutil::open(&full) {
                Some(k) => k,
                None => continue,
            };
            let display = sk.get_value::<String, _>("DisplayName").unwrap_or_default();
            if display.trim().is_empty() {
                continue;
            }
            // Entries explicitly marked as system components are left alone.
            if sk.get_value::<u32, _>("SystemComponent").unwrap_or(0) == 1 {
                continue;
            }
            let uninst = sk.get_value::<String, _>("UninstallString").unwrap_or_default();
            if uninst.trim().is_empty() {
                continue;
            }
            let exe = command_exe(&uninst);
            if exe.is_empty() || exe.to_lowercase().contains("msiexec") {
                continue;
            }
            let exe = regutil::expand_env(&exe);
            if !exists(&exe) {
                push(
                    &mut issues,
                    "key",
                    root.clone(),
                    sub.clone(),
                    "برنامج غير موجود".into(),
                    "Missing program".into(),
                    format!("{display} → {exe}"),
                    false,
                );
            }
        }
    }

    // 2) App Paths pointing at missing executables ------------------------
    for (root, label_ar, label_en) in [
        (
            r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
            "مسار تطبيق غير صالح",
            "Invalid App Path",
        ),
        (
            r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths",
            "مسار تطبيق غير صالح",
            "Invalid App Path",
        ),
    ] {
        let key = match regutil::open(root) {
            Some(k) => k,
            None => continue,
        };
        for sub in key.enum_keys().flatten() {
            let full = format!(r"{root}\{sub}");
            let sk = match regutil::open(&full) {
                Some(k) => k,
                None => continue,
            };
            let target: String = sk.get_value("").unwrap_or_default();
            if target.trim().is_empty() {
                continue;
            }
            let exe = regutil::expand_env(&command_exe(&target));
            if !exists(&exe) {
                push(
                    &mut issues,
                    "key",
                    root.to_string(),
                    sub.clone(),
                    label_ar.into(),
                    label_en.into(),
                    format!("{sub} → {exe}"),
                    false,
                );
            }
        }
    }

    // 3) Shared DLL references to deleted files ---------------------------
    let shared = r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\SharedDLLs";
    for (name, _) in regutil::enum_values_at(shared) {
        if name.is_empty() {
            continue;
        }
        let file = regutil::expand_env(&name);
        if !exists(&file) {
            push(
                &mut issues,
                "value",
                shared.to_string(),
                name.clone(),
                "مكتبة مشتركة مفقودة".into(),
                "Missing shared DLL".into(),
                file,
                false,
            );
        }
    }

    // 4) Startup entries pointing at missing files ------------------------
    for root in [
        r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
        r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
        r"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
    ] {
        for (name, value) in regutil::enum_values_at(root) {
            let exe = regutil::expand_env(&command_exe(&value));
            if exe.is_empty() {
                continue;
            }
            if !exists(&exe) {
                push(
                    &mut issues,
                    "value",
                    root.to_string(),
                    name.clone(),
                    "عنصر بدء تشغيل معطّل".into(),
                    "Broken startup entry".into(),
                    format!("{name} → {exe}"),
                    false,
                );
            }
        }
    }

    // 5) Fonts registered but missing ------------------------------------
    let fonts_key = r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts";
    let fonts_dir = format!(r"{}\Fonts", util::windir());
    for (name, file) in regutil::enum_values_at(fonts_key) {
        if file.is_empty() || file.to_lowercase().starts_with('c') {
            continue;
        }
        let path = format!(r"{fonts_dir}\{file}");
        if !exists(&path) {
            push(
                &mut issues,
                "value",
                fonts_key.to_string(),
                name.clone(),
                "خط مفقود".into(),
                "Missing font".into(),
                path,
                true,
            );
        }
    }

    issues
}

/// Merge raw `reg export` files into one restorable .reg file. Kept pure so
/// the backup format can be tested without touching the real registry.
fn merge_reg_exports(exports: &[String]) -> String {
    let mut merged = String::from("Windows Registry Editor Version 5.00\r\n\r\n");
    for text in exports {
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Windows Registry Editor") || trimmed.is_empty() {
                continue;
            }
            merged.push_str(line);
            merged.push_str("\r\n");
        }
        merged.push_str("\r\n");
    }
    merged
}

pub fn backup(issues: &[Issue]) -> Result<String, String> {
    let stamp = util::now_unix();
    let file = util::backups_dir().join(format!("registry-{stamp}.reg"));
    let mut targets: Vec<String> = issues.iter().map(|i| i.key_path.clone()).collect();
    targets.sort();
    targets.dedup();

    let mut exports: Vec<String> = Vec::new();
    for (idx, key) in targets.iter().enumerate() {
        let tmp = util::backups_dir().join(format!("registry-{stamp}-{idx}.reg"));
        let tmp_s = tmp.to_string_lossy().to_string();
        let args = vec![
            "export".to_string(),
            key.clone(),
            tmp_s,
            "/y".to_string(),
        ];
        if util::run_and_wait_vec("reg", &args).is_err() {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&tmp) {
            exports.push(text);
        }
        let _ = std::fs::remove_file(&tmp);
    }
    let merged = merge_reg_exports(&exports);
    std::fs::write(&file, merged).map_err(|e| e.to_string())?;
    Ok(file.to_string_lossy().to_string())
}

pub fn fix(issues: &[Issue]) -> (usize, Vec<String>) {
    let mut fixed = 0;
    let mut errors = Vec::new();
    for issue in issues {
        let ok = match issue.action.as_str() {
            "value" => regutil::delete_value(&issue.key_path, &issue.name),
            "key" => regutil::delete_subkey(&issue.key_path, &issue.name),
            _ => false,
        };
        if ok {
            fixed += 1;
        } else if errors.len() < 10 {
            errors.push(format!("{} :: {}", issue.key_path, issue.name));
        }
    }
    (fixed, errors)
}

pub fn hive_of(path: &str) -> Option<Hive> {
    regutil::split_path(path).map(|(h, _)| h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_merges_multiple_exports_without_duplicate_header() {
        let a = "Windows Registry Editor Version 5.00\r\n\r\n[HKEY_CURRENT_USER\\A]\r\n\"x\"=dword:1\r\n";
        let b = "Windows Registry Editor Version 5.00\r\n\r\n[HKEY_CURRENT_USER\\B]\r\n\"y\"=dword:2\r\n";
        let merged = merge_reg_exports(&[a.into(), b.into()]);
        assert_eq!(merged.matches("Windows Registry Editor").count(), 1);
        assert!(merged.contains("[HKEY_CURRENT_USER\\A]"));
        assert!(merged.contains("[HKEY_CURRENT_USER\\B]"));
        assert!(merged.contains("\"x\"=dword:1"));
    }
}
