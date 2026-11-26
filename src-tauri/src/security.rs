//! Read-only Security Center data. Nothing here changes Defender, Firewall
//! or any system protection setting.

use serde::Serialize;

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DefenderStatus {
    pub real_time: bool,
    pub antivirus: bool,
    pub tamper_protection: bool,
    pub signature_updated: String,
    pub available: bool,
    pub error: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FirewallProfile {
    pub name: String,
    pub enabled: bool,
}

const DEFENDER_SCRIPT: &str = r#"
[Console]::OutputEncoding=[System.Text.Encoding]::UTF8
try {
  $s = Get-MpComputerStatus -ErrorAction Stop
  [pscustomobject]@{
    available = $true
    realTime = [bool]$s.RealTimeProtectionEnabled
    antivirus = [bool]$s.AntivirusEnabled
    tamper = [bool]$s.IsTamperProtected
    signature = [string]$s.SignatureLastUpdated
    error = ''
  } | ConvertTo-Json -Compress
} catch {
  [pscustomobject]@{ available=$false; realTime=$false; antivirus=$false; tamper=$false; signature=''; error=$_.Exception.Message } | ConvertTo-Json -Compress
}
"#;

const FIREWALL_SCRIPT: &str = r#"
[Console]::OutputEncoding=[System.Text.Encoding]::UTF8
try {
  Get-NetFirewallProfile -ErrorAction Stop | Select-Object Name, Enabled | ConvertTo-Json -Compress
} catch { '[]' }
"#;

pub fn defender_status() -> DefenderStatus {
    let text = match crate::util::run_and_wait(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", DEFENDER_SCRIPT],
    ) {
        Ok(t) => t,
        Err(e) => {
            return DefenderStatus {
                error: e,
                ..Default::default()
            }
        }
    };
    let v: serde_json::Value = match serde_json::from_str(text.trim()) {
        Ok(v) => v,
        Err(e) => {
            return DefenderStatus {
                error: e.to_string(),
                ..Default::default()
            }
        }
    };
    let b = |k: &str| v.get(k).and_then(|x| x.as_bool()).unwrap_or(false);
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    DefenderStatus {
        real_time: b("realTime"),
        antivirus: b("antivirus"),
        tamper_protection: b("tamper"),
        signature_updated: s("signature"),
        available: b("available"),
        error: s("error"),
    }
}

pub fn firewall_status() -> Vec<FirewallProfile> {
    let text = match crate::util::run_and_wait(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", FIREWALL_SCRIPT],
    ) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let v: serde_json::Value = match serde_json::from_str(text.trim()) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let rows = if v.is_array() {
        v.as_array().unwrap().clone()
    } else {
        vec![v]
    };
    rows.iter()
        .map(|r| FirewallProfile {
            name: r.get("Name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            enabled: r.get("Enabled").and_then(|x| x.as_bool()).unwrap_or(false),
        })
        .collect()
}

fn take_history(text: &str, limit: usize) -> Vec<String> {
    text.lines().rev().take(limit).map(|s| s.to_string()).collect()
}

/// Last local security-relevant log lines. Never sent anywhere.
pub fn history(limit: usize) -> Vec<String> {
    let path = crate::util::data_dir().join("log.txt");
    match std::fs::read_to_string(path) {
        Ok(text) => take_history(&text, limit),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_takes_latest_lines_only() {
        let out = take_history("one\ntwo\nthree", 2);
        assert_eq!(out, vec!["three", "two"]);
    }
}
