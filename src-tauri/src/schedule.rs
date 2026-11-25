//! Scheduled cleaning through the Windows Task Scheduler.

use crate::util;

pub const TASK_NAME: &str = "VEXORAAutoClean";
/// Task name used by the previous brand; removed on upgrade.
pub const LEGACY_TASK: &str = "AbuMaryamCleaner-AutoClean";

pub fn create(frequency: &str, time: &str) -> Result<String, String> {
    let exe = util::exe_path();
    if exe.is_empty() {
        return Err("cannot locate executable".into());
    }
    let _ = remove();
    // Delete the old brand's scheduled task if it is still registered.
    let _ = util::run_and_wait_vec(
        "schtasks",
        &[
            "/Delete".into(),
            "/TN".into(),
            LEGACY_TASK.into(),
            "/F".into(),
        ],
    );

    let (sc, extra): (&str, Vec<String>) = match frequency {
        "daily" => ("DAILY", vec![]),
        "monthly" => ("MONTHLY", vec!["/D".into(), "1".into()]),
        _ => ("WEEKLY", vec!["/D".into(), "SUN".into()]),
    };

    let mut args: Vec<String> = vec![
        "/Create".into(),
        "/TN".into(),
        TASK_NAME.into(),
        "/TR".into(),
        format!("\"{exe}\" --autoclean"),
        "/SC".into(),
        sc.into(),
        "/ST".into(),
        time.to_string(),
        "/F".into(),
        "/RL".into(),
        "HIGHEST".into(),
    ];
    args.extend(extra);

    util::run_and_wait_vec("schtasks", &args).map(|_| TASK_NAME.to_string())
}

pub fn remove() -> Result<(), String> {
    let args = vec![
        "/Delete".to_string(),
        "/TN".to_string(),
        TASK_NAME.to_string(),
        "/F".to_string(),
    ];
    let _ = util::run_and_wait_vec("schtasks", &args);
    Ok(())
}

pub fn exists() -> bool {
    let args = vec![
        "/Query".to_string(),
        "/TN".to_string(),
        TASK_NAME.to_string(),
    ];
    util::run_and_wait_vec("schtasks", &args).is_ok()
}
