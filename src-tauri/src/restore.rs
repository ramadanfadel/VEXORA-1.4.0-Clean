//! Windows System Restore integration: create, list and open restore points.

use serde::Serialize;

use crate::util;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestorePoint {
    pub seq: u32,
    pub description: String,
    pub created: String,
    pub kind: u32,
}

fn ps(script: &str) -> Result<String, String> {
    util::run_and_wait(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ],
    )
}

/// Create a restore point. Requires administrator rights and System Protection.
pub fn create(description: &str) -> Result<(), String> {
    if !util::is_admin() {
        return Err("admin required".into());
    }
    let desc = description
        .replace('\'', "''")
        .replace(['\r', '\n'], " ")
        .chars()
        .take(120)
        .collect::<String>();
    let script = format!(
        "$ErrorActionPreference='Stop'; \
         try {{ Enable-ComputerRestore -Drive ($env:SystemDrive + '\\') -ErrorAction SilentlyContinue }} catch {{}}; \
         Checkpoint-Computer -Description '{desc}' -RestorePointType 'MODIFY_SETTINGS'"
    );
    ps(&script).map(|_| ())
}

pub fn list() -> Vec<RestorePoint> {
    let script = "Get-ComputerRestorePoint -ErrorAction SilentlyContinue | \
                  Select-Object SequenceNumber,Description,CreationTime,RestorePointType | \
                  ConvertTo-Json -Compress";
    let text = match ps(script) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let parsed: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let arr = if parsed.is_array() {
        parsed.as_array().cloned().unwrap_or_default()
    } else {
        vec![parsed]
    };
    arr.into_iter()
        .map(|v| RestorePoint {
            seq: v.get("SequenceNumber").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
            description: v
                .get("Description")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            created: v
                .get("CreationTime")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            kind: v
                .get("RestorePointType")
                .and_then(|x| x.as_u64())
                .unwrap_or(0) as u32,
        })
        .collect()
}

/// Open the built-in System Restore wizard.
pub fn open() -> Result<(), String> {
    util::run_command("rstrui.exe", &[]).map_err(|e| e)
}
