//! Read-only network overview. This module never changes adapters, DNS or
//! routing — it only reports what Windows already exposes.

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdapterInfo {
    pub name: String,
    pub status: String,
    pub mac: String,
    pub ips: Vec<String>,
    pub gateways: Vec<String>,
    pub dns: Vec<String>,
    pub dhcp: bool,
}

const SCRIPT: &str = r#"
[Console]::OutputEncoding=[System.Text.Encoding]::UTF8
Get-CimInstance Win32_NetworkAdapterConfiguration -ErrorAction SilentlyContinue |
  Where-Object { $_.IPEnabled -eq $true } |
  ForEach-Object {
    [pscustomobject]@{
      name = [string]$_.Description
      status = [string]$_.Status
      mac = [string]$_.MACAddress
      ips = @($_.IPAddress)
      gateways = @($_.DefaultIPGateway)
      dns = @($_.DNSServerSearchOrder)
      dhcp = [bool]$_.DHCPEnabled
    }
  } | ConvertTo-Json -Depth 4 -Compress
"#;

pub fn status() -> Vec<AdapterInfo> {
    let text = match crate::util::run_and_wait(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", SCRIPT],
    ) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }
    let value: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let rows = if value.is_array() {
        value.as_array().unwrap().clone()
    } else {
        vec![value]
    };
    let strings = |v: &serde_json::Value, k: &str| -> Vec<String> {
        v.get(k)
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    };
    let field = |v: &serde_json::Value, k: &str| {
        v.get(k)
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string()
    };
    rows.iter()
        .map(|r| AdapterInfo {
            name: field(r, "name"),
            status: field(r, "status"),
            mac: field(r, "mac"),
            ips: strings(r, "ips"),
            gateways: strings(r, "gateways"),
            dns: strings(r, "dns"),
            dhcp: r.get("dhcp").and_then(|x| x.as_bool()).unwrap_or(false),
        })
        .collect()
}
