//! Startup manager: Run keys + Startup folders, with enable/disable support.

use serde::Serialize;

use crate::regutil;
use crate::util;

pub const RUN_KEYS: [&str; 3] = [
    r"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run",
    r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
    r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
];

const BACKUP_KEY: &str = r"HKEY_CURRENT_USER\Software\AbuMaryamCleaner\StartupDisabled";

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub source: String,
    pub source_ar: String,
    pub source_en: String,
    pub location: String,
    pub enabled: bool,
    pub exists: bool,
    pub publisher: String,
    pub safe_to_disable: bool,
    pub suggestion: String,
}

fn startup_dirs() -> Vec<(String, &'static str, &'static str)> {
    vec![
        (
            format!(
                r"{}\Microsoft\Windows\Start Menu\Programs\Startup",
                util::appdata()
            ),
            "مجلد بدء التشغيل (المستخدم)",
            "Startup folder (user)",
        ),
        (
            format!(
                r"{}\Microsoft\Windows\Start Menu\Programs\StartUp",
                util::program_data()
            ),
            "مجلد بدء التشغيل (الكل)",
            "Startup folder (all users)",
        ),
    ]
}

fn hex(s: &str) -> String {
    s.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn classify_startup(name: &str, command: &str) -> (bool, String) {
    let lower = format!("{} {}", name, command).to_lowercase();
    let essential = [
        "windows", "microsoft", "defender", "security", "antivirus", "audio", "realtek",
        "nvidia", "amd", "intel", "driver", "synaptics", "touchpad", "bluetooth",
        "wifi", "network", "explorer", "shell", "ctfmon", "dwm", "lsass",
    ];
    if essential.iter().any(|e| lower.contains(e)) {
        (false, "system/essential".into())
    } else {
        (true, "not required at startup".into())
    }
}

pub fn items() -> Vec<StartupItem> {
    let mut out: Vec<StartupItem> = Vec::new();

    for key in RUN_KEYS {
        let (src_ar, src_en) = if key.contains("HKEY_CURRENT_USER") {
            ("الريجستري (المستخدم)", "Registry (user)")
        } else if key.contains("WOW6432Node") {
            ("الريجستري (32-بت)", "Registry (32-bit)")
        } else {
            ("الريجستري (الكل)", "Registry (machine)")
        };
        for (name, value) in regutil::enum_values_at(key) {
            let exe = regutil::expand_env(&crate::registry::command_exe(&value));
            let (safe, sug) = classify_startup(&name, &value);
            out.push(StartupItem {
                name: name.clone(),
                command: value,
                source: format!("reg:{key}"),
                source_ar: src_ar.into(),
                source_en: src_en.into(),
                location: key.to_string(),
                enabled: true,
                exists: exe.is_empty() || std::path::Path::new(&exe).exists(),
                publisher: String::new(),
                safe_to_disable: safe,
                suggestion: sug,
            });
        }
    }

    // Disabled entries we parked earlier.
    if let Some(backup) = regutil::open(BACKUP_KEY) {
        for sub in backup.enum_keys().flatten() {
            let full = format!(r"{BACKUP_KEY}\{sub}");
            if let Some(k) = regutil::open(&full) {
                let name: String = k.get_value("name").unwrap_or_default();
                let command: String = k.get_value("cmd").unwrap_or_default();
                let src: String = k.get_value("src").unwrap_or_default();
                let s_ar: String = k.get_value("srcAr").unwrap_or_default();
                let s_en: String = k.get_value("srcEn").unwrap_or_default();
                out.push(StartupItem {
                    name,
                    command,
                    source: format!("disabled:{sub}"),
                    source_ar: s_ar,
                    source_en: s_en,
                    location: src,
                    enabled: false,
                    exists: true,
                    publisher: String::new(),
                    safe_to_disable: false,
                    suggestion: "disabled".into(),
                });
            }
        }
    }

    for (dir, ar, en) in startup_dirs() {
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if name.eq_ignore_ascii_case("desktop.ini") {
                    continue;
                }
                let (safe, sug) = classify_startup(&name, &path.to_string_lossy());
                out.push(StartupItem {
                    name,
                    command: path.to_string_lossy().to_string(),
                    source: format!("file:{}", path.to_string_lossy()),
                    source_ar: ar.into(),
                    source_en: en.into(),
                    location: dir.clone(),
                    enabled: true,
                    exists: true,
                    publisher: String::new(),
                    safe_to_disable: safe,
                    suggestion: sug,
                });
            }
        }
    }

    // Parked startup-folder files.
    let parked = util::data_dir().join("StartupDisabled");
    if let Ok(rd) = std::fs::read_dir(&parked) {
        for entry in rd.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let original = std::fs::read_to_string(path.with_extension("src"))
                .unwrap_or_default();
            out.push(StartupItem {
                name: name.clone(),
                command: path.to_string_lossy().to_string(),
                source: format!("parked:{name}"),
                source_ar: "مجلد بدء التشغيل (معطّل)".into(),
                source_en: "Startup folder (disabled)".into(),
                location: original,
                enabled: false,
                exists: true,
                publisher: String::new(),
                safe_to_disable: false,
                suggestion: "disabled".into(),
            });
        }
    }

    out.sort_by(|a, b| b.enabled.cmp(&a.enabled).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    out
}

pub fn disable(source: &str, name: &str) -> Result<(), String> {
    if let Some(key) = source.strip_prefix("reg:") {
        let value = regutil::enum_values_at(key)
            .into_iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
            .ok_or_else(|| "entry not found".to_string())?;
        let (src_ar, src_en) = if key.contains("HKEY_CURRENT_USER") {
            ("الريجستري (المستخدم)", "Registry (user)")
        } else if key.contains("WOW6432Node") {
            ("الريجستري (32-بت)", "Registry (32-bit)")
        } else {
            ("الريجستري (الكل)", "Registry (machine)")
        };
        let sub = hex(&format!("{key}|{name}"));
        let full = format!(r"{BACKUP_KEY}\{sub}");
        let k = regutil::create_key(&full).ok_or_else(|| "cannot write backup key".to_string())?;
        let _ = k.set_value("name", &name.to_string());
        let _ = k.set_value("cmd", &value);
        let _ = k.set_value("src", &key.to_string());
        let _ = k.set_value("srcAr", &src_ar.to_string());
        let _ = k.set_value("srcEn", &src_en.to_string());
        if regutil::delete_value(key, name) {
            Ok(())
        } else {
            Err("cannot delete startup value (run as admin?)".into())
        }
    } else if let Some(path) = source.strip_prefix("file:") {
        let src = std::path::Path::new(path);
        let dir = util::data_dir().join("StartupDisabled");
        let _ = std::fs::create_dir_all(&dir);
        let target = dir.join(src.file_name().unwrap_or_default());
        std::fs::rename(src, &target).map_err(|e| e.to_string())?;
        let _ = std::fs::write(target.with_extension("src"), path);
        Ok(())
    } else {
        Err("unsupported source".into())
    }
}

pub fn enable(source: &str, name: &str) -> Result<(), String> {
    let _ = name;
    if let Some(sub) = source.strip_prefix("disabled:") {
        let full = format!(r"{BACKUP_KEY}\{sub}");
        let k = regutil::open(&full).ok_or_else(|| "missing backup entry".to_string())?;
        let nm: String = k.get_value("name").unwrap_or_default();
        let cmd: String = k.get_value("cmd").unwrap_or_default();
        let src: String = k.get_value("src").unwrap_or_default();
        let target = src.strip_prefix("reg:").unwrap_or(&src).to_string();
        if regutil::set_string(&target, &nm, &cmd) {
            regutil::delete_subkey(BACKUP_KEY, sub);
            Ok(())
        } else {
            Err("cannot restore startup value (run as admin?)".into())
        }
    } else if let Some(file) = source.strip_prefix("parked:") {
        let dir = util::data_dir().join("StartupDisabled");
        let from = dir.join(file);
        let original = std::fs::read_to_string(from.with_extension("src")).unwrap_or_default();
        if original.is_empty() {
            return Err("original location unknown".into());
        }
        let to = std::path::PathBuf::from(&original).join(file);
        std::fs::rename(&from, &to).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(from.with_extension("src"));
        Ok(())
    } else {
        Err("unsupported source".into())
    }
}
pub fn remove(source: &str, name: &str) -> Result<(), String> {
    if let Some(key) = source.strip_prefix("reg:") {
        if regutil::delete_value(key, name) {
            Ok(())
        } else {
            Err("cannot delete startup value (run as admin?)".into())
        }
    } else if let Some(path) = source.strip_prefix("file:") {
        util::delete_path(std::path::Path::new(path));
        Ok(())
    } else if let Some(sub) = source.strip_prefix("disabled:") {
        if regutil::delete_subkey(BACKUP_KEY, sub) {
            Ok(())
        } else {
            Err("cannot delete backup entry".into())
        }
    } else if let Some(file) = source.strip_prefix("parked:") {
        let dir = util::data_dir().join("StartupDisabled");
        let _ = std::fs::remove_file(dir.join(file));
        let _ = std::fs::remove_file(dir.join(file).with_extension("src"));
        Ok(())
    } else {
        Err("unsupported source".into())
    }
}

#[derive(serde::Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct StartupExtras {
    pub icons: std::collections::HashMap<String, String>,
    pub publishers: std::collections::HashMap<String, String>,
}

pub fn extract_startup_extras(exes: &[String]) -> StartupExtras {
    let mut out = StartupExtras::default();
    if exes.is_empty() { return out; }
    out.icons = crate::boost::extract_process_icons(exes);
    let mut script = String::from("$out = @{}; ");
    for exe in exes {
        let safe = exe.replace('\'', "''");
        script.push_str(&format!(
            "try {{ $v = [System.Diagnostics.FileVersionInfo]::GetVersionInfo('{}'); if ($v -and $v.CompanyName) {{ $out['{}'] = $v.CompanyName }} }} catch {{ }}; ",
            safe, safe
        ));
    }
    script.push_str("$out | ConvertTo-Json -Compress");
    if let Ok(text) = crate::util::run_and_wait("powershell", &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script.as_str()]) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(obj) = parsed.as_object() {
                    for (k, v) in obj {
                        if let Some(s) = v.as_str() {
                            if !s.is_empty() {
                                out.publishers.insert(k.clone(), s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    out
}
