//! Explainable system assessment. Every point in the score maps to a real
//! check with a visible reason — no invented "health percentage".
//! This module never changes the system; it only reads data gathered by
//! existing modules.

use serde::Serialize;

use crate::battery::BatteryInfo;
use crate::sysinfo::Health;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DoctorCheck {
    pub label: String,
    pub ok: bool,
    pub detail: String,
    pub action: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
    pub score: u32,
    pub explainable: bool,
}

pub fn assess(
    health: &Health,
    battery: &BatteryInfo,
    restore_count: usize,
    sensors_available: usize,
) -> DoctorReport {
    let mut checks = Vec::new();

    let mem_pct = if health.mem_total > 0 {
        100.0 * (health.mem_total - health.mem_used.min(health.mem_total)) as f32
            / health.mem_total as f32
    } else {
        0.0
    };
    checks.push(DoctorCheck {
        label: "Free memory".into(),
        ok: mem_pct >= 15.0,
        detail: format!("{mem_pct:.0}% free"),
        action: "Use Memory Boost or close background apps".into(),
    });

    let worst_disk_pct = health
        .disks
        .iter()
        .filter(|d| d.total > 0)
        .map(|d| 100.0 * d.free as f32 / d.total as f32)
        .fold(f32::INFINITY, f32::min);
    let disk_ok = worst_disk_pct.is_finite() && worst_disk_pct >= 10.0;
    checks.push(DoctorCheck {
        label: "Disk space".into(),
        ok: disk_ok,
        detail: if worst_disk_pct.is_finite() {
            format!("lowest free space {worst_disk_pct:.0}%")
        } else {
            "no disk data".into()
        },
        action: "Run the junk cleaner or Disk Map".into(),
    });

    checks.push(DoctorCheck {
        label: "Restore points".into(),
        ok: restore_count > 0,
        detail: format!("{restore_count} point(s) available"),
        action: "Create a restore point before major changes".into(),
    });

    checks.push(DoctorCheck {
        label: "Startup items".into(),
        ok: health.startup_count <= 15,
        detail: format!("{} enabled items", health.startup_count),
        action: "Review entries in Startup Analyzer".into(),
    });

    if battery.present {
        checks.push(DoctorCheck {
            label: "Battery health".into(),
            ok: battery.health_pct >= 70,
            detail: format!("{}% of design capacity", battery.health_pct),
            action: "Enable Smart Charge Guard".into(),
        });
    } else {
        checks.push(DoctorCheck {
            label: "Battery health".into(),
            ok: true,
            detail: "not applicable (desktop)".into(),
            action: "—".into(),
        });
    }

    checks.push(DoctorCheck {
        label: "Hardware sensors".into(),
        ok: sensors_available > 0,
        detail: format!("{sensors_available} sensor reading(s)"),
        action: "Sensors depend on the device/driver; N/A is normal".into(),
    });

    checks.push(DoctorCheck {
        label: "Administrator features".into(),
        ok: health.admin,
        detail: if health.admin {
            "available".into()
        } else {
            "not elevated — some repair tools are unavailable".into()
        },
        action: "Run as administrator only when needed".into(),
    });

    let ok = checks.iter().filter(|c| c.ok).count();
    let score = ((ok as f32 / checks.len() as f32) * 100.0).round() as u32;
    DoctorReport {
        checks,
        score,
        explainable: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn health(mem_used: u64, startup: usize) -> Health {
        Health {
            cpu: 10.0,
            mem_used,
            mem_total: 100,
            disks: vec![crate::sysinfo::Disk {
                path: "C:\\".into(),
                total: 100,
                used: 20,
                free: 80,
            }],
            uptime: 10,
            os: "Windows 11".into(),
            os_build: "22631".into(),
            host: "PC".into(),
            user: "user".into(),
            admin: true,
            cores: 4,
            startup_count: startup,
        }
    }

    fn battery(present: bool, health_pct: i32) -> BatteryInfo {
        BatteryInfo {
            present,
            health_pct,
            ..Default::default()
        }
    }

    #[test]
    fn score_is_explainable() {
        let r = assess(&health(10, 5), &battery(false, 0), 1, 3);
        assert_eq!(r.score, 100);
        assert_eq!(r.checks.len(), 7);
        assert!(r.explainable);
        assert!(r.checks.iter().all(|c| !c.detail.is_empty() && !c.action.is_empty()));
    }

    #[test]
    fn low_memory_lowers_score_with_reason() {
        let r = assess(&health(95, 5), &battery(false, 0), 1, 3);
        assert!(r.score < 100);
        let mem = r.checks.iter().find(|c| c.label == "Free memory").unwrap();
        assert!(!mem.ok);
        assert!(mem.detail.contains("5%"));
    }
}
