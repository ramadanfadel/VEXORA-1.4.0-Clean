//! Windows services and scheduled tasks manager.

use serde::Serialize;

use crate::util;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub name: String,
    pub display: String,
    pub status: String,
    pub start_type: String,
    pub pid: u32,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub name: String,
    pub status: String,
    pub next_run: String,
    pub last_run: String,
    pub author: String,
}

pub fn list_services() -> Vec<Service> {
    let script = "Get-Service | Select-Object Name,DisplayName,Status,StartType,Id | ConvertTo-Json -Compress";
    let text = match util::run_and_wait(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script],
    ) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let trimmed = text.trim();
    if trimmed.is_empty() { return Vec::new(); }
    let parsed: serde_json::Value = match serde_json::from_str(trimmed) { Ok(v) => v, Err(_) => return Vec::new() };
    let arr = if parsed.is_array() { parsed.as_array().cloned().unwrap_or_default() } else { vec![parsed] };
    arr.into_iter().map(|v| Service {
        name: v.get("Name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        display: v.get("DisplayName").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        status: v.get("Status").and_then(|x| x.as_u64()).map(|n| match n { 1 => "Stopped", 4 => "Running", _ => "Other" }).unwrap_or("Unknown").to_string(),
        start_type: v.get("StartType").and_then(|x| x.as_u64()).map(|n| match n { 0 => "Boot", 1 => "System", 2 => "Automatic", 3 => "Manual", 4 => "Disabled", _ => "Unknown" }).unwrap_or("Unknown").to_string(),
        pid: v.get("Id").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
    }).collect()
}

pub fn set_service(name: &str, enabled: bool) -> Result<(), String> {
    if name.trim().is_empty() { return Err("name required".into()); }
    let safe = name.replace('\'', "''");
    let cmd = if enabled {
        format!("Set-Service -Name '{safe}' -StartupType Automatic -ErrorAction Stop; Start-Service -Name '{safe}' -ErrorAction SilentlyContinue")
    } else {
        format!("Stop-Service -Name '{safe}' -Force -ErrorAction SilentlyContinue; Set-Service -Name '{safe}' -StartupType Disabled -ErrorAction Stop")
    };
    util::run_and_wait("powershell", &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", cmd.as_str()]).map(|_| ())
}

pub fn list_tasks() -> Vec<Task> {
    // Use schtasks /Query /FO CSV
    let out = match util::run_and_wait_vec("schtasks", &["/Query".into(), "/FO".into(), "CSV".into(), "/NH".into()]) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut tasks = Vec::new();
    for line in out.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        // Parse CSV: "TaskName","Next Run Time","Status"
        let fields: Vec<String> = parse_csv(line);
        if fields.len() < 3 { continue; }
        let name = fields[0].trim().to_string();
        if name.is_empty() || name.eq_ignore_ascii_case("TaskName") { continue; }
        tasks.push(Task {
            name,
            next_run: fields.get(1).cloned().unwrap_or_default().trim().to_string(),
            status: fields.get(2).cloned().unwrap_or_default().trim().to_string(),
            last_run: String::new(),
            author: String::new(),
        });
    }
    tasks.truncate(200);
    tasks
}

fn parse_csv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            if in_quotes && chars.peek() == Some(&'"') {
                cur.push('"');
                chars.next();
            } else {
                in_quotes = !in_quotes;
            }
        } else if c == ',' && !in_quotes {
            out.push(cur.clone());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    out.push(cur);
    out
}

pub fn set_task(name: &str, enabled: bool) -> Result<(), String> {
    if name.trim().is_empty() { return Err("name required".into()); }
    let args = if enabled {
        vec!["/Change".into(), "/TN".into(), name.to_string(), "/ENABLE".into()]
    } else {
        vec!["/Change".into(), "/TN".into(), name.to_string(), "/DISABLE".into()]
    };
    util::run_and_wait_vec("schtasks", &args).map(|_| ())
}
