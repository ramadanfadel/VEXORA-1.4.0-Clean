//! System health snapshot: CPU, memory, disks, uptime, Windows build.

use serde::Serialize;

use crate::regutil;
use crate::util;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Disk {
    pub path: String,
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    pub cpu: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub disks: Vec<Disk>,
    pub uptime: u64,
    pub os: String,
    pub os_build: String,
    pub host: String,
    pub user: String,
    pub admin: bool,
    pub cores: usize,
    pub startup_count: usize,
}

fn cpu_sample() -> (u64, u64) {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::GetSystemTimes;
    let mut idle = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut kernel = idle;
    let mut user = idle;
    unsafe {
        GetSystemTimes(&mut idle, &mut kernel, &mut user);
    }
    let f = |t: FILETIME| ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64;
    let idle_t = f(idle);
    let total = f(kernel) + f(user);
    (idle_t, total)
}

pub(crate) fn cpu_percent() -> f32 {
    let (i1, t1) = cpu_sample();
    std::thread::sleep(std::time::Duration::from_millis(150));
    let (i2, t2) = cpu_sample();
    let dt = t2.saturating_sub(t1) as f64;
    let di = i2.saturating_sub(i1) as f64;
    if dt <= 0.0 {
        return 0.0;
    }
    (((dt - di) / dt) * 100.0).clamp(0.0, 100.0) as f32
}

pub(crate) fn memory() -> (u64, u64) {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        dwMemoryLoad: 0,
        ullTotalPhys: 0,
        ullAvailPhys: 0,
        ullTotalPageFile: 0,
        ullAvailPageFile: 0,
        ullTotalVirtual: 0,
        ullAvailVirtual: 0,
        ullAvailExtendedVirtual: 0,
    };
    let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
    if ok == 0 {
        return (0, 0);
    }
    (
        status.ullTotalPhys - status.ullAvailPhys,
        status.ullTotalPhys,
    )
}

fn uptime_secs() -> u64 {
    use windows_sys::Win32::System::SystemInformation::GetTickCount64;
    unsafe { GetTickCount64() / 1000 }
}

/// Windows 11 starts at build 22000. Later builds (e.g. 26H2 / 26300)
/// remain Windows 11 — we deliberately do not invent an unverified newer
/// product name. The family is never taken from ProductName, because Windows
/// 11 still stores "Windows 10 Pro" there.
fn windows_family(build: u32) -> &'static str {
    if build >= 22000 { "Windows 11" } else { "Windows 10" }
}

/// Edition is read independently from EditionID; ProductName is only a
/// last-resort hint for the edition text (never for the family name).
fn windows_edition(edition_id: &str, product_name: &str) -> String {
    let id = edition_id.trim();
    if !id.is_empty() {
        let mapped = match id {
            "Professional" => "Pro",
            "ProfessionalN" => "Pro N",
            "Core" => "Home",
            "CoreN" => "Home N",
            "EnterpriseN" => "Enterprise N",
            "ProEducation" => "Pro Education",
            "ProEducationN" => "Pro Education N",
            "EducationN" => "Education N",
            _ => id,
        };
        return mapped.to_string();
    }
    let p = product_name.trim();
    for prefix in ["Windows 11 ", "Windows 10 ", "Windows "] {
        if let Some(rest) = p.strip_prefix(prefix) {
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }
    String::new()
}

fn windows_name() -> (String, String) {
    let key = r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion";
    let mut product = String::new();
    let mut display = String::new();
    let mut edition = String::new();
    let mut build_str = String::new();
    if let Some(k) = regutil::open(key) {
        product = k.get_value::<String, _>("ProductName").unwrap_or_default();
        display = k.get_value::<String, _>("DisplayVersion").unwrap_or_default();
        edition = k.get_value::<String, _>("EditionID").unwrap_or_default();
        build_str = k.get_value::<String, _>("CurrentBuildNumber").unwrap_or_default();
        if build_str.trim().is_empty() {
            // Older systems may only expose CurrentBuild.
            build_str = k.get_value::<String, _>("CurrentBuild").unwrap_or_default();
        }
    }
    let build_num: u32 = build_str.trim().parse().unwrap_or(0);
    windows_info(build_num, &product, &display, &edition, &build_str)
}

/// Pure assembly helper so the classification can be unit-tested without
/// touching the live registry.
fn windows_info(
    build_num: u32,
    product: &str,
    display: &str,
    edition: &str,
    build_str: &str,
) -> (String, String) {
    let family = if build_num == 0 {
        // Registry unreadable: ProductName is a hint only in this fallback.
        if product.to_lowercase().contains("windows 11") {
            "Windows 11"
        } else {
            "Windows 10"
        }
    } else {
        windows_family(build_num)
    };
    let ed = windows_edition(edition, product);
    let name = if ed.is_empty() {
        family.to_string()
    } else {
        format!("{family} {ed}")
    };
    let mut parts: Vec<String> = Vec::new();
    if !display.trim().is_empty() {
        parts.push(display.trim().to_string());
    }
    if !build_str.trim().is_empty() {
        parts.push(build_str.trim().to_string());
    }
    (name, parts.join(" · "))
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpecDisk {
    pub path: String,
    pub model: String,
    pub total: u64,
    pub kind: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpecGpu {
    pub name: String,
    pub vram: u64,
    pub driver: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemSpecs {
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub cpu_mhz: u64,
    pub ram_total: u64,
    pub ram_slots: String,
    pub gpus: Vec<SpecGpu>,
    pub disks: Vec<SpecDisk>,
    pub motherboard: String,
    pub bios: String,
    pub os_name: String,
    pub os_version: String,
    pub os_arch: String,
    pub hostname: String,
    pub username: String,
}

pub fn specs() -> SystemSpecs {
    // CPU info via WMI
    let cpu_info = wmi_query("Win32_Processor", &["Name", "NumberOfCores", "NumberOfLogicalProcessors", "MaxClockSpeed"]);
    let (cpu_name, cpu_cores, cpu_threads, cpu_mhz) = if let Some(row) = cpu_info.first() {
        (
            row.get("Name").cloned().unwrap_or_default(),
            row.get("NumberOfCores").and_then(|s| s.parse().ok()).unwrap_or(num_cpus::get()),
            row.get("NumberOfLogicalProcessors").and_then(|s| s.parse().ok()).unwrap_or(num_cpus::get()),
            row.get("MaxClockSpeed").and_then(|s| s.parse().ok()).unwrap_or(0),
        )
    } else {
        (std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "CPU".into()), num_cpus::get(), num_cpus::get(), 0)
    };

    let (mem_total, _) = memory();
    // RAM slots via WMI
    let ram_slots = wmi_query("Win32_PhysicalMemory", &["Capacity", "Speed", "Manufacturer"])
        .len()
        .to_string();

    // GPU via WMI
    let gpu_rows = wmi_query("Win32_VideoController", &["Name", "AdapterRAM", "DriverVersion"]);
    let mut gpus = Vec::new();
    for r in gpu_rows {
        let name = r.get("Name").cloned().unwrap_or_default();
        if name.to_lowercase().contains("microsoft") && name.to_lowercase().contains("basic") {
            continue; // skip basic render driver
        }
        let vram = r.get("AdapterRAM").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let driver = r.get("DriverVersion").cloned().unwrap_or_default();
        gpus.push(SpecGpu { name, vram, driver });
    }
    if gpus.is_empty() {
        // fallback: keep at least one entry
        gpus.push(SpecGpu { name: "Unknown GPU".into(), vram: 0, driver: String::new() });
    }

    // Disks via WMI
    let disk_rows = wmi_query("Win32_DiskDrive", &["Model", "Size", "MediaType"]);
    let mut disks = Vec::new();
    for (i, d) in util::drives().into_iter().enumerate() {
        let (total, _used, _free) = util::disk_space(&d);
        let (model, kind) = if i < disk_rows.len() {
            (
                disk_rows[i].get("Model").cloned().unwrap_or_default(),
                disk_rows[i].get("MediaType").cloned().unwrap_or_default(),
            )
        } else {
            (String::new(), String::new())
        };
        let kind = if kind.to_lowercase().contains("ssd") || kind.to_lowercase().contains("fixed") {
            "SSD/Fixed".into()
        } else if kind.is_empty() {
            "Fixed".into()
        } else {
            kind
        };
        disks.push(SpecDisk { path: d, model, total, kind });
    }

    // Motherboard
    let mb_rows = wmi_query("Win32_BaseBoard", &["Manufacturer", "Product"]);
    let motherboard = if let Some(r) = mb_rows.first() {
        format!(
            "{} {}",
            r.get("Manufacturer").cloned().unwrap_or_default(),
            r.get("Product").cloned().unwrap_or_default()
        )
        .trim()
        .to_string()
    } else {
        String::new()
    };

    // BIOS
    let bios_rows = wmi_query("Win32_BIOS", &["SMBIOSBIOSVersion", "Manufacturer"]);
    let bios = if let Some(r) = bios_rows.first() {
        format!(
            "{} {}",
            r.get("Manufacturer").cloned().unwrap_or_default(),
            r.get("SMBIOSBIOSVersion").cloned().unwrap_or_default()
        )
        .trim()
        .to_string()
    } else {
        String::new()
    };

    let (os_name, os_version) = windows_name();
    let arch = std::env::var("PROCESSOR_ARCHITECTURE").unwrap_or_else(|_| "x64".into());

    SystemSpecs {
        cpu_name,
        cpu_cores,
        cpu_threads,
        cpu_mhz,
        ram_total: mem_total,
        ram_slots,
        gpus,
        disks,
        motherboard,
        bios,
        os_name,
        os_version,
        os_arch: arch,
        hostname: std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC".into()),
        username: std::env::var("USERNAME").unwrap_or_default(),
    }
}

/// Run a WMI query via PowerShell and return rows as Vec<HashMap<String,String>>.
fn wmi_query(class: &str, fields: &[&str]) -> Vec<std::collections::HashMap<String, String>> {
    let field_list = fields.join(",");
    let script = format!(
        "Get-CimInstance -ClassName {} -ErrorAction SilentlyContinue | Select-Object {} | ConvertTo-Json -Compress",
        class, field_list
    );
    let text = match util::run_and_wait(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script.as_str(),
        ],
    ) {
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
        .map(|v| {
            let mut m = std::collections::HashMap::new();
            for f in fields {
                let val = v
                    .get(*f)
                    .map(|x| {
                        if x.is_string() {
                            x.as_str().unwrap_or("").to_string()
                        } else if x.is_number() {
                            x.to_string()
                        } else {
                            String::new()
                        }
                    })
                    .unwrap_or_default();
                m.insert(f.to_string(), val);
            }
            m
        })
        .collect()
}



/// Lightweight CPU/RAM sample for the floating widget (no WMI, no disk scan).
pub fn quick_cpu_mem() -> (f32, u64, u64) {
    let (used, total) = memory();
    (cpu_percent(), used, total)
}

pub fn health() -> Health {
    let (mem_used, mem_total) = memory();
    let (os, os_build) = windows_name();
    let disks = util::drives()
        .into_iter()
        .filter(|d| {
            util::disk_space(d).0 > 0
        })
        .map(|d| {
            let (total, used, free) = util::disk_space(&d);
            Disk {
                path: d,
                total,
                used,
                free,
            }
        })
        .collect();

    let host = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC".into());
    let user = std::env::var("USERNAME").unwrap_or_default();
    let startup_count = crate::startup::items().len();

    Health {
        cpu: cpu_percent(),
        mem_used,
        mem_total,
        disks,
        uptime: uptime_secs(),
        os,
        os_build,
        host,
        user,
        admin: util::is_admin(),
        cores: num_cpus::get(),
        startup_count,
    }
}

#[cfg(test)]
mod os_tests {
    use super::*;

    #[test]
    fn family_uses_build_number_threshold() {
        assert_eq!(windows_family(19045), "Windows 10");
        assert_eq!(windows_family(21999), "Windows 10");
        assert_eq!(windows_family(22000), "Windows 11");
        assert_eq!(windows_family(22631), "Windows 11");
        assert_eq!(windows_family(26300), "Windows 11");
        // Future builds must not be guessed as a newer product name.
        assert_eq!(windows_family(40000), "Windows 11");
    }

    #[test]
    fn edition_read_independently_and_preserved() {
        assert_eq!(windows_edition("Professional", "Windows 10 Pro"), "Pro");
        assert_eq!(windows_edition("Core", "Windows 11 Home"), "Home");
        assert_eq!(windows_edition("Enterprise", "Windows 10 Enterprise"), "Enterprise");
        assert_eq!(windows_edition("ProfessionalN", ""), "Pro N");
        assert_eq!(windows_edition("", "Windows 10 Enterprise"), "Enterprise");
    }

    #[test]
    fn windows_11_26h2_build_26300() {
        // ProductName still says Windows 10 on this system — must be ignored.
        let (name, version) = windows_info(
            26300,
            "Windows 10 Pro",
            "26H2",
            "Professional",
            "26300",
        );
        assert_eq!(name, "Windows 11 Pro");
        assert_eq!(version, "26H2 · 26300");
        assert_eq!(format!("{name} {version}"), "Windows 11 Pro 26H2 · 26300");
    }

    #[test]
    fn windows_10_stays_windows_10() {
        let (name, version) = windows_info(
            19045,
            "Windows 10 Pro",
            "22H2",
            "Professional",
            "19045",
        );
        assert_eq!(name, "Windows 10 Pro");
        assert_eq!(version, "22H2 · 19045");
    }

    #[test]
    fn future_build_keeps_windows_11_name() {
        let (name, version) = windows_info(
            30000,
            "Windows 10 Pro",
            "30H1",
            "Professional",
            "30000",
        );
        assert_eq!(name, "Windows 11 Pro");
        assert_eq!(version, "30H1 · 30000");
    }

    #[test]
    fn handles_missing_build_and_display() {
        let (name, version) = windows_info(0, "Windows 11 Pro", "", "Professional", "");
        assert_eq!(name, "Windows 11 Pro");
        assert_eq!(version, "");
    }
}
