//! Read-only viewer for `.reg` backups created before registry cleanup.
//! Nothing here writes to the registry — restoring is handled by the
//! existing quarantine flow, which the user triggers explicitly.

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegBackup {
    pub name: String,
    pub bytes: u64,
    pub keys: usize,
    pub values: usize,
    pub modified: u64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegKeyView {
    pub key: String,
    pub lines: Vec<String>,
}

/// Parse a .reg export into key headers and their value lines.
fn parse(text: &str) -> Vec<RegKeyView> {
    let mut out: Vec<RegKeyView> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim_end_matches('\r');
        let trimmed = line.trim();
        if trimmed.starts_with("Windows Registry Editor") || trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            out.push(RegKeyView {
                key: trimmed.to_string(),
                lines: Vec::new(),
            });
        } else if let Some(last) = out.last_mut() {
            last.lines.push(line.to_string());
        }
    }
    out
}

fn safe_name(name: &str) -> bool {
    !name.trim().is_empty()
        && std::path::Path::new(name).file_name().map(|f| f == std::ffi::OsStr::new(name)).unwrap_or(false)
        && name.to_lowercase().ends_with(".reg")
}

pub fn list() -> Vec<RegBackup> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(crate::util::backups_dir()) {
        for e in rd.flatten() {
            let p = e.path();
            if !p.is_file() || !p.extension().map(|x| x == "reg").unwrap_or(false) {
                continue;
            }
            let meta = match std::fs::metadata(&p) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let keys = std::fs::read_to_string(&p)
                .map(|t| parse(&t).len())
                .unwrap_or(0);
            let values = std::fs::read_to_string(&p)
                .map(|t| parse(&t).iter().map(|k| k.lines.len()).sum())
                .unwrap_or(0);
            out.push(RegBackup {
                name: p.file_name().unwrap_or_default().to_string_lossy().to_string(),
                bytes: meta.len(),
                keys,
                values,
                modified: meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            });
        }
    }
    out.sort_by(|a, b| b.name.cmp(&a.name));
    out
}

pub fn view(name: &str) -> Result<Vec<RegKeyView>, String> {
    if !safe_name(name) {
        return Err("invalid backup name".into());
    }
    let path = crate::util::backups_dir().join(name);
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(parse(&text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reg_export() {
        let text = "Windows Registry Editor Version 5.00\r\n\r\n[HKEY_CURRENT_USER\\A]\r\n\"x\"=dword:1\r\n\r\n[HKEY_CURRENT_USER\\B]\r\n\"y\"=\"z\"\r\n";
        let keys = parse(text);
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].key, "[HKEY_CURRENT_USER\\A]");
        assert_eq!(keys[0].lines, vec!["\"x\"=dword:1"]);
        assert_eq!(keys[1].lines, vec!["\"y\"=\"z\""]);
    }

    #[test]
    fn rejects_traversal_names() {
        assert!(!safe_name("..\\evil.reg"));
        assert!(!safe_name("notdotreg"));
        assert!(safe_name("registry-123.reg"));
    }
}
