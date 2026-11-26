//! Read-only inventory of applications with available updates, using the
//! winget CLI already shipped with Windows. No third-party service is used.
//! Applying an update is always a separate, explicit user action.

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdate {
    pub id: String,
    pub name: String,
    pub installed: String,
    pub available: String,
    pub source: String,
}

/// Parse winget's JSON output. Accept both a bare array and `{Packages:[...]}`.
fn parse(text: &str) -> Vec<AppUpdate> {
    let value: serde_json::Value = match serde_json::from_str(text.trim()) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let arr = value
        .get("Packages")
        .and_then(|x| x.as_array())
        .or_else(|| value.as_array());
    let Some(rows) = arr else { return Vec::new() };
    let field = |v: &serde_json::Value, k: &str| {
        v.get(k)
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string()
    };
    rows.iter()
        .filter_map(|r| {
            let id = field(r, "Id");
            let available = field(r, "AvailableVersion");
            if id.is_empty() || available.is_empty() {
                return None;
            }
            Some(AppUpdate {
                id,
                name: field(r, "Name"),
                installed: field(r, "InstalledVersion"),
                available,
                source: field(r, "Source"),
            })
        })
        .collect()
}

pub fn scan() -> Result<Vec<AppUpdate>, String> {
    let text = crate::util::run_and_wait(
        "winget",
        &[
            "list",
            "--upgrade-available",
            "--format=json",
            "--accept-source-agreements",
            "--disable-interactivity",
        ],
    )?;
    Ok(parse(&text))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 200
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
}

pub fn upgrade(id: &str) -> Result<(bool, String), String> {
    if !valid_id(id) {
        return Ok((false, "invalid package id".into()));
    }
    let out = crate::util::run_and_wait(
        "winget",
        &[
            "upgrade",
            "--id",
            id,
            "-e",
            "--silent",
            "--accept-package-agreements",
            "--accept-source-agreements",
            "--disable-interactivity",
        ],
    )?;
    Ok((true, out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_winget_json() {
        let json = r#"{"Packages":[{"Id":"Vendor.App","Name":"App","InstalledVersion":"1.0","AvailableVersion":"2.0","Source":"winget"}]}"#;
        let list = parse(json);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "Vendor.App");
        assert_eq!(list[0].available, "2.0");
    }

    #[test]
    fn empty_or_broken_output_yields_no_updates() {
        assert!(parse("not json").is_empty());
        assert!(parse(r#"{"Packages":[]}"#).is_empty());
    }

    #[test]
    fn package_id_validation_blocks_shell_input() {
        assert!(valid_id("Vendor.App-1+2"));
        assert!(!valid_id("a & calc"));
        assert!(!valid_id(""));
    }
}
