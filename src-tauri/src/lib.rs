//! Tauri command surface for VEXORA.

pub mod catalog;
pub mod battery;
pub mod hwmon;
pub mod boost;
pub mod bgmonitor;
pub mod browsers;
pub mod clean;
pub mod history;
pub mod fonts;
pub mod junk;
pub mod log;
pub mod monitor;
pub mod ramopt;
pub mod ctxmenu;
pub mod repair;
pub mod regutil;
pub mod registry;
pub mod restore;
pub mod safety;
pub mod scan;
pub mod schedule;
pub mod services;
pub mod settings;
pub mod shredder;
pub mod startup;
pub mod state;
pub mod sysinfo;
pub mod hyperscan_adapter;
pub mod tools;
pub mod tune;
pub mod uninstall;
pub mod update;
pub mod util;
pub mod widget;

use std::sync::atomic::Ordering;

/// Last version for which an update toast was emitted (avoid duplicates).
static UPDATE_NOTIFIED: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use catalog::{Cleaner, Danger, Kind};
use settings::Settings;
use state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanerRow {
    id: String,
    name: String,
    desc: String,
    category: String,
    paths: Vec<String>,
    admin: bool,
    danger: String,
    sensitivity: String, // safe | review | sensitive
    default_on: bool,
    kind: String,
}

fn row(c: &Cleaner, s: &Settings) -> CleanerRow {
    let ar = s.language == "ar";
    CleanerRow {
        id: c.id.clone(),
        name: if ar { c.ar.clone() } else { c.en.clone() },
        desc: if ar { c.dar.clone() } else { c.den.clone() },
        category: c.cat.clone(),
        paths: c.paths.clone(),
        admin: c.admin,
        danger: match c.danger {
            Danger::Safe => "safe",
            Danger::Caution => "caution",
            Danger::Advanced => "advanced",
        }
        .into(),
        sensitivity: match c.danger {
            Danger::Safe => "safe",
            Danger::Caution => "review",
            Danger::Advanced => "sensitive",
        }
        .into(),
        default_on: c.default_on,
        kind: match c.kind {
            Kind::Files => "files",
            Kind::Recycle => "recycle",
            Kind::Command => "command",
            Kind::Clipboard => "clipboard",
            Kind::RegistryValues => "registry",
        }
        .into(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Catalog {
    categories: Vec<CatRow>,
    cleaners: Vec<CleanerRow>,
    admin: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CatRow {
    id: String,
    name: String,
    icon: String,
}

#[tauri::command]
fn get_catalog(state: State<'_, AppState>) -> Catalog {
    let s = state.settings.lock().unwrap().clone();
    let ar = s.language == "ar";
    Catalog {
        categories: catalog::categories()
            .into_iter()
            .map(|(id, nar, nen, icon)| CatRow {
                id: id.to_string(),
                name: if ar { nar } else { nen }.to_string(),
                icon: icon.to_string(),
            })
            .collect(),
        cleaners: catalog::cleaners().iter().map(|c| row(c, &s)).collect(),
        admin: util::is_admin(),
    }
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn save_settings(mut settings: Settings, state: State<'_, AppState>) -> Result<Settings, String> {
    // Never keep the GitHub token in plaintext settings.json.
    let token = settings.github_token.clone();
    settings.github_token.clear();
    util::store_github_token(&token)?;
    settings.save()?;
    *state.settings.lock().unwrap() = settings.clone();
    Ok(settings)
}

#[tauri::command]
async fn system_specs() -> Result<sysinfo::SystemSpecs, String> {
    tauri::async_runtime::spawn_blocking(sysinfo::specs)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_health() -> Result<sysinfo::Health, String> {
    tauri::async_runtime::spawn_blocking(sysinfo::health)
        .await
        .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ scanning
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ScanRow {
    id: String,
    size: u64,
    count: u64,
    truncated: bool,
    note: String,
    items: Vec<scan::Item>,
}

#[tauri::command]
async fn scan_cleaners(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<Vec<ScanRow>, String> {
    let s = state.settings.lock().unwrap().clone();
    let all = catalog::cleaners();
    let selected: Vec<Cleaner> = all
        .into_iter()
        .filter(|c| ids.iter().any(|i| i == &c.id))
        .collect();
    let exclusions = s.exclusions.clone();
    let ar = s.language == "ar";

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut rows = Vec::new();
        for c in selected {
            let outcome = scan::scan_one(&c, &exclusions, 400);
            let row = ScanRow {
                id: c.id.clone(),
                size: outcome.size,
                count: outcome.count,
                truncated: outcome.truncated,
                note: if ar { outcome.note_ar.clone() } else { outcome.note_en.clone() },
                items: outcome.items,
            };
            let _ = app.emit(
                "scan:progress",
                serde_json::json!({
                    "id": row.id,
                    "size": row.size,
                    "count": row.count,
                }),
            );
            rows.push(row);
        }
        rows
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}

// ------------------------------------------------------------------ cleaning
#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct CleanRow {
    id: String,
    removed: u64,
    freed: u64,
    skipped: u64,
    errors: Vec<String>,
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct CleanSummary {
    removed: u64,
    freed: u64,
    skipped: u64,
    /// "" when disabled, "ok" when a restore point was created,
    /// "admin" when elevation is needed, "error:…" on failure.
    restore: String,
    rows: Vec<CleanRow>,
}

#[tauri::command]
async fn clean_cleaners(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<CleanSummary, String> {
    let s = state.settings.lock().unwrap().clone();
    let all = catalog::cleaners();
    let selected: Vec<Cleaner> = all
        .into_iter()
        .filter(|c| ids.iter().any(|i| i == &c.id))
        .collect();

    let opts = clean::CleanOptions {
        recycle: s.recycle_bin,
        shred: s.shred,
        passes: s.shred_passes,
        exclusions: s.exclusions.clone(),
    };
    // A restore point is only needed for non-reversible work. When every
    // selected cleaner is file-based and the Recycle Bin is on, cleaning is
    // already recoverable — skip the very slow VSS snapshot.
    let only_reversible_files = s.recycle_bin
        && !s.shred
        && selected.iter().all(|c| matches!(c.kind, Kind::Files));
    let restore_first = s.restore_before_clean && !only_reversible_files;

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut summary = CleanSummary::default();
        if restore_first {
            if util::is_admin() {
                match restore::create("VEXORA — قبل التنظيف") {
                    Ok(_) => summary.restore = "ok".into(),
                    Err(e) => summary.restore = format!("error:{e}"),
                }
            } else {
                summary.restore = "admin".into();
            }
        }
        // Run up to four cleaners in parallel: file walks and registry reads
        // are independent I/O, so this cuts the visible wait substantially.
        for batch in selected.chunks(4) {
            let mut handles = Vec::with_capacity(batch.len());
            for c in batch {
                let c = c.clone();
                let o = opts.clone();
                handles.push(std::thread::spawn(move || {
                    let out = clean::clean_one(&c, &o);
                    (c, out)
                }));
            }
            for h in handles {
                if let Ok((c, out)) = h.join() {
                    summary.removed += out.removed;
                    summary.freed += out.freed;
                    summary.skipped += out.skipped;
                    let row = CleanRow {
                        id: c.id.clone(),
                        removed: out.removed,
                        freed: out.freed,
                        skipped: out.skipped,
                        errors: out.errors.clone(),
                    };
                    let _ = app.emit("clean:progress", row.clone());
                    summary.rows.push(row);
                }
            }
        }
        summary
    })
    .await
    .map_err(|e| e.to_string())?;

    if result.restore == "ok" {
        let mut st = state.settings.lock().unwrap();
        st.last_restore = util::now_unix();
        let _ = st.save();
    }
    if result.removed > 0 {
        history::add(
            "clean",
            "تنظيف الملفات",
            &format!("{} عنصراً", result.removed),
            result.freed,
            result.removed,
        );
    }
    let rows: Vec<String> = result
        .rows
        .iter()
        .filter(|r| r.removed > 0 || r.skipped > 0)
        .map(|r| format!("{}={} (تخطي {})", r.id, r.removed, r.skipped))
        .collect();
    log::write(
        "clean",
        &format!(
            "تنظيف {}: {} عنصراً، {} بايت، {} تخطي{}",
            if result.restore.is_empty() { "يدوي" } else { "مع نقطة استعادة" },
            result.removed,
            result.freed,
            result.skipped,
            if rows.is_empty() { String::new() } else { format!(" | {}", rows.join(", ")) }
        ),
    );
    Ok(result)
}

#[tauri::command]
fn cancel_work(state: State<'_, AppState>) {
    state.cancel.store(true, Ordering::Relaxed);
}

// ------------------------------------------------------------------ registry
#[tauri::command]
async fn registry_scan(state: State<'_, AppState>) -> Result<Vec<registry::Issue>, String> {
    let issues = tauri::async_runtime::spawn_blocking(registry::scan)
        .await
        .map_err(|e| e.to_string())?;
    let mut cache = state.registry_issues.lock().unwrap();
    *cache = issues.clone();
    let mut list = issues;
    // Re-number so the frontend can reference the cached vector by index.
    for (i, item) in list.iter_mut().enumerate() {
        item.id = i;
    }
    Ok(list)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FixResult {
    fixed: usize,
    backup: String,
    errors: Vec<String>,
}

#[tauri::command]
async fn registry_fix(
    state: State<'_, AppState>,
    ids: Vec<usize>,
) -> Result<FixResult, String> {
    let issues = {
        let cache = state.registry_issues.lock().unwrap();
        ids.iter()
            .filter_map(|i| cache.get(*i).cloned())
            .collect::<Vec<_>>()
    };
    if issues.is_empty() {
        return Err("nothing selected".into());
    }
    let details: Vec<String> = issues.iter().map(|i| i.detail.clone()).take(10).collect();
    let total = issues.len();
    let backup = registry::backup(&issues).unwrap_or_default();
    let (fixed, errors) = tauri::async_runtime::spawn_blocking(move || registry::fix(&issues))
        .await
        .map_err(|e| e.to_string())?;
    if fixed > 0 {
        history::add("registry", "إصلاح الريجستري", &format!("{fixed} مشكلة"), 0, fixed as u64);
    }
    log::write(
        "registry",
        &format!(
            "إصلاح {fixed} مشكلة من {total} | نسخة احتياطية: {} | {}{}",
            if backup.is_empty() { "—" } else { &backup },
            details.join(" ; "),
            if total > 10 { " …" } else { "" }
        ),
    );
    Ok(FixResult {
        fixed,
        backup,
        errors,
    })
}

#[tauri::command]
async fn registry_backup_only(state: State<'_, AppState>) -> Result<String, String> {
    let issues = state.registry_issues.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || registry::backup(&issues))
        .await
        .map_err(|e| e.to_string())?
}

// ------------------------------------------------------------------ startup
// ------------------------------------------------------------------ startup extras (icons + publishers)
#[tauri::command]
async fn startup_extras(exes: Vec<String>) -> Result<startup::StartupExtras, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(startup::extract_startup_extras(&exes)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn startup_list() -> Result<Vec<startup::StartupItem>, String> {
    tauri::async_runtime::spawn_blocking(startup::items)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn startup_set(source: String, name: String, enabled: bool) -> Result<(), String> {
    let (src2, name2) = (source.clone(), name.clone());
    let res = tauri::async_runtime::spawn_blocking(move || {
        if enabled {
            startup::enable(&source, &name)
        } else {
            startup::disable(&source, &name)
        }
    })
    .await
    .map_err(|e| e.to_string())?;
    log::write(
        "startup",
        &format!("{} «{name2}» ({src2})", if enabled { "تفعيل بدء التشغيل" } else { "تعطيل بدء التشغيل" }),
    );
    res
}

#[tauri::command]
async fn startup_remove(source: String, name: String) -> Result<(), String> {
    let (src2, name2) = (source.clone(), name.clone());
    let res = tauri::async_runtime::spawn_blocking(move || startup::remove(&source, &name))
        .await
        .map_err(|e| e.to_string())?;
    log::write("startup", &format!("حذف «{name2}» من بدء التشغيل ({src2})"));
    res
}

// ------------------------------------------------------------------ uninstall
#[tauri::command]
async fn programs_list() -> Result<Vec<uninstall::Program>, String> {
    tauri::async_runtime::spawn_blocking(uninstall::list)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn program_uninstall(key: String, name: Option<String>) -> Result<uninstall::UninstallOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let all = uninstall::list();
        let found = all.into_iter().find(|p| {
            p.key == key || name.as_ref().map(|n| p.name.eq_ignore_ascii_case(n)).unwrap_or(false)
        });
        match found {
            Some(p) => Ok(uninstall::launch_uninstall(&p)),
            None => Err("program not found".into()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Delete leftovers the user reviewed and selected. Honours the recycle-bin
/// setting and re-validates every path on the Rust side.
#[tauri::command]
async fn delete_leftovers(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<uninstall::DeleteOutcome, String> {
    let recycle = state.settings.lock().unwrap().recycle_bin;
    let out = tauri::async_runtime::spawn_blocking(move || {
        uninstall::delete_leftovers(&paths, recycle)
    })
    .await
    .map_err(|e| e.to_string())?;
    if out.removed > 0 {
        history::add(
            "uninstall",
            "تنظيف البقايا",
            &format!("{} عنصراً", out.removed),
            out.freed,
            out.removed,
        );
    }
    Ok(out)
}

/// Automatic search for install/uninstall remains (broken entries, orphan
/// folders, dead shortcuts) — returns a review plan, deletes nothing.
#[tauri::command]
async fn leftover_scan() -> Result<safety::CleanupPlan, String> {
    tauri::async_runtime::spawn_blocking(|| Ok(uninstall::leftover_scan()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn program_remove_entry(key: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let all = uninstall::list();
        match all.into_iter().find(|p| p.key == key) {
            Some(p) => uninstall::remove_entry(&p),
            None => Err("program not found".into()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn program_leftovers(key: String) -> Result<Vec<uninstall::Leftover>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let all = uninstall::list();
        match all.into_iter().find(|p| p.key == key) {
            Some(p) => Ok(uninstall::leftovers(&p)),
            None => Err("program not found".to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

// ------------------------------------------------------------------ tools
#[tauri::command]
async fn dup_scan(
    app: tauri::AppHandle,
    roots: Vec<String>,
    min_size: u64,
    engine: String,
) -> Result<tools::DupResult, String> {
    let engine_lower = engine.to_lowercase();
    if engine_lower == "hyper" {
        // Use czkawka-core powered HyperScan adapter
        use crate::hyperscan_adapter::{scan, ScanEngine};
        let root0 = roots.first().cloned().unwrap_or_default();
        let res = tauri::async_runtime::spawn_blocking(move || {
            let result = scan(ScanEngine::HyperScan, &root0, min_size);
            // Convert ScanResult to DupResult for backward compatibility
            tools::DupResult {
                groups: Vec::new(), // groups will be populated in full integration
                reclaimable: result.reclaimable,
                scanned_files: result.scanned_files,
                errors: result.errors,
            }
        });
        let res = res.await.map_err(|e| e.to_string())?;
        Ok(res)
    } else {
        // Use classic VEXORA engine (original find_duplicates)
        use tauri::Emitter;
        let res = tauri::async_runtime::spawn_blocking(move || {
            tools::find_duplicates(&roots, min_size, |n| {
                let _ = app.emit("dup-progress", n);
            })
        });
        let res = res.await.map_err(|e| e.to_string())?;
        Ok(res)
    }
}

#[tauri::command]
fn dup_cancel() -> bool {
    tools::DUP_CANCEL.store(true, std::sync::atomic::Ordering::Relaxed);
    true
}

// ------------------------------------------------------------------ system repair
#[tauri::command]
async fn system_repair_run(app: AppHandle) -> Result<repair::RepairResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = match util::try_op_lock() {
            Some(g) => g,
            None => {
                return repair::RepairResult {
                    ok: false,
                    cancelled: false,
                    needs_admin: false,
                    summary: "another operation is running".into(),
                    log: String::new(),
                }
            }
        };
        repair::run(&app)
    })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn system_repair_cancel() -> bool {
    repair::REPAIR_CANCEL.store(true, std::sync::atomic::Ordering::Relaxed);
    true
}

// ------------------------------------------------------------------ spooler
#[tauri::command]
async fn spooler_status() -> Result<u32, String> {
    tauri::async_runtime::spawn_blocking(tools::spooler_pending_count)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn spooler_fix() -> Result<(bool, String), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = match util::try_op_lock() {
            Some(g) => g,
            None => return (false, "another operation is running".into()),
        };
        tools::spooler_fix()
    })
        .await
        .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ context menu
#[tauri::command]
async fn ctxmenu_list() -> Result<Vec<ctxmenu::CtxItem>, String> {
    tauri::async_runtime::spawn_blocking(ctxmenu::list)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ctxmenu_set(scope: String, name: String, clsid: String, enable: bool) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || ctxmenu::set(&scope, &name, &clsid, enable))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ctxmenu_win11_classic(enabled: bool) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || ctxmenu::win11_classic(enabled))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ctxmenu_win11_status() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(ctxmenu::win11_classic_status)
        .await
        .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ adobe cache
#[tauri::command]
async fn adobe_cache_items() -> Result<Vec<tools::CacheItem>, String> {
    tauri::async_runtime::spawn_blocking(tools::adobe_cache_items)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn adobe_running() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(tools::adobe_running)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn drivers_backup(folder: String) -> Result<(bool, String), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = match util::try_op_lock() {
            Some(g) => g,
            None => return (false, "another operation is running".into()),
        };
        tools::drivers_backup(&folder)
    })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn runtimes_status() -> Result<Vec<tools::RuntimeStatus>, String> {
    tauri::async_runtime::spawn_blocking(tools::runtimes_status)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn winget_install(id: String) -> Result<(bool, String), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = match util::try_op_lock() {
            Some(g) => g,
            None => return (false, "another operation is running".into()),
        };
        tools::winget_install(&id)
    })
    .await
    .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ hardware monitor
#[tauri::command]
async fn hwmon_status() -> Result<hwmon::HardwareInfo, String> {
    tauri::async_runtime::spawn_blocking(hwmon::read)
        .await
        .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ battery
#[tauri::command]
async fn battery_status() -> Result<battery::BatteryInfo, String> {
    tauri::async_runtime::spawn_blocking(battery::status)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn battery_monitor_apply(enabled: bool, upper: u32, lower: u32) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        battery::apply(enabled, upper, lower);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

// ------------------------------------------------------------------ shredder / unlocker
#[tauri::command]
async fn unlocker_list(path: String) -> Result<Vec<shredder::LockingProcess>, String> {
    tauri::async_runtime::spawn_blocking(move || shredder::locking_processes(&path))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn shred_path(path: String, passes: u32) -> Result<shredder::ShredResult, String> {
    tauri::async_runtime::spawn_blocking(move || shredder::shred(&path, passes))
        .await
        .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ fonts
#[tauri::command]
async fn fonts_list() -> Result<Vec<fonts::FontInfo>, String> {
    tauri::async_runtime::spawn_blocking(fonts::list)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn font_backup(name: String, path: String, source: String, dest: String) -> Result<bool, String> {
    // reconstruct FontInfo
    let fi = fonts::FontInfo { name, file: String::new(), path, source, exists: true, duplicate: false };
    tauri::async_runtime::spawn_blocking(move || Ok(fonts::backup(&fi, &dest)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn font_delete(name: String, path: String, source: String) -> Result<bool, String> {
    let fi = fonts::FontInfo { name, file: String::new(), path, source, exists: true, duplicate: false };
    tauri::async_runtime::spawn_blocking(move || Ok(fonts::delete(&fi)))
        .await
        .map_err(|e| e.to_string())?
}

// ------------------------------------------------------------------ hyperscan
// Standalone HyperScan command (experimental).
// Uses czkawka-core under the hood; results are compatible with DupResult.
// This is the backend command; the UI typically calls dup_scan with engine="hyper".
#[tauri::command]
#[allow(dead_code)] // experimental: kept for future UI wiring; currently reachable only via dup_scan(engine="hyper")
async fn hyper_dup_scan(_root: String, _min_size: u64) -> Result<tools::DupResult, String> {
    // Use czkawka-core powered HyperScan adapter
    use crate::hyperscan_adapter::{scan, ScanEngine};
    let res = tauri::async_runtime::spawn_blocking(move || {
        let result = scan(ScanEngine::HyperScan, "", 0); // root/min_size not used in placeholder
        // Convert ScanResult to DupResult for backward compatibility
        tools::DupResult {
            groups: Vec::new(),
            reclaimable: result.reclaimable,
            scanned_files: result.scanned_files,
            errors: result.errors,
        }
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(res)
}

// ------------------------------------------------------------------ drives
#[tauri::command]
async fn drives_list() -> Result<Vec<tools::DriveInfo>, String> {
    tauri::async_runtime::spawn_blocking(tools::drives_info)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn disk_check(letter: String) -> Result<tools::DriveCheck, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = match util::try_op_lock() {
            Some(g) => g,
            None => {
                return tools::DriveCheck {
                    letter: letter.clone(),
                    ok: false,
                    code: -1,
                    status: "unavailable".into(),
                    raw: "another operation is running".into(),
                }
            }
        };
        tools::disk_check(&letter)
    })
    .await
    .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ similar images
#[tauri::command]
async fn similar_images_scan(root: String, max_diff: u32) -> Result<Vec<tools::SimilarImageGroup>, String> {
    tauri::async_runtime::spawn_blocking(move || tools::find_similar_images(&root, max_diff))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn big_scan(root: String, min_size: u64, limit: usize) -> Result<Vec<tools::BigFile>, String> {
    tauri::async_runtime::spawn_blocking(move || tools::big_files(&root, min_size, limit))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn empty_scan(root: String, limit: usize) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || tools::empty_dirs(&root, limit))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn disk_map(root: String, depth: u32) -> Result<tools::MapNode, String> {
    tauri::async_runtime::spawn_blocking(move || tools::disk_map(&root, depth))
        .await
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteResult {
    removed: u64,
    freed: u64,
    failed: u64,
}

#[tauri::command]
async fn delete_paths(
    state: State<'_, AppState>,
    paths: Vec<String>,
    shred: bool,
) -> Result<DeleteResult, String> {
    let passes = state.settings.lock().unwrap().shred_passes;
    let recycle = state.settings.lock().unwrap().recycle_bin;
    let (removed, freed, failed) = tauri::async_runtime::spawn_blocking(move || {
        // Security: validate every path before touching the filesystem.
        let mut valid: Vec<String> = Vec::new();
        let mut rejected = 0u64;
        for p in &paths {
            match safety::validate_path_for_delete(p) {
                Ok(canon) => valid.push(canon.to_string_lossy().to_string()),
                Err(_) => rejected += 1,
            }
        }
        if valid.is_empty() {
            return (0u64, 0u64, rejected + paths.len() as u64);
        }
        let (r, f, e) = if shred {
            tools::delete_many(&valid, true, passes)
        } else if recycle {
            let freed: u64 = valid.iter().filter_map(|p| std::fs::metadata(p).ok().map(|m| m.len())).sum();
            match util::recycle_batch(&valid) {
                Ok(()) => (valid.len() as u64, freed, 0),
                Err(_) => tools::delete_many(&valid, false, passes),
            }
        } else {
            tools::delete_many(&valid, false, passes)
        };
        (r, f, e + rejected)
    })
    .await
    .map_err(|e| e.to_string())?;
    if removed > 0 {
        history::add("tools", "حذف عناصر", &format!("{removed} عنصراً"), freed, removed);
    }
    Ok(DeleteResult {
        removed,
        freed,
        failed,
    })
}

#[tauri::command]
async fn wipe_free_space(
    app: AppHandle,
    state: State<'_, AppState>,
    drive: String,
    passes: u32,
) -> Result<u64, String> {
    state.cancel.store(false, Ordering::Relaxed);
    let cancel = state.cancel.clone();
    let app2 = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut last = 0u64;
        tools::wipe_free_space(&drive, passes, &cancel, |written, total| {
            if written.saturating_sub(last) > 8 * 1024 * 1024 || written == total {
                last = written;
                let _ = app2.emit(
                    "wipe:progress",
                    serde_json::json!({ "written": written, "total": total }),
                );
            }
        })
    })
    .await
    .map_err(|e| e.to_string())?;
    state.cancel.store(false, Ordering::Relaxed);
    let bytes = result?;
    history::add(
        "wipe",
        "مسح المساحة الحرة",
        &format!("{passes} مرة"),
        bytes,
        0,
    );
    Ok(bytes)
}

// ------------------------------------------------------------------ misc
#[tauri::command]
fn flush_dns() -> Result<(), String> {
    util::run_command("ipconfig", &["/flushdns".into()])
}

#[tauri::command]
fn empty_recycle() -> Result<(), String> {
    util::empty_recycle_bin()
}

#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "VEXORA",
        "version": env!("CARGO_PKG_VERSION"),
        "tagline": "Clean Smarter. Run Faster.",
        "taglineAr": "تنظيف أذكى، وأداء أسرع",
        "designer": "أ. رمضان فاضل مصطفى (أبو مريم)",
        "copyright": "© 2026 أ. رمضان فاضل مصطفى (أبو مريم)",
        "admin": util::is_admin(),
        "dataDir": util::data_dir().to_string_lossy(),
        "backupsDir": util::backups_dir().to_string_lossy(),
        "reportsDir": util::reports_dir().to_string_lossy(),
        "exe": util::exe_path(),
        "os": "windows",
    })
}

#[tauri::command]
fn open_path(path: String) {
    util::open_path(&path);
}

#[tauri::command]
fn reveal_path(path: String) -> Result<(), String> {
    util::reveal_path(&path)
}

#[tauri::command]
async fn pick_folder() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(util::pick_folder)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn pick_file() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(util::pick_file)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn relaunch_admin() -> bool {
    util::relaunch_as_admin()
}

#[tauri::command]
fn history_list(limit: usize) -> Vec<history::Entry> {
    history::list(limit)
}

#[tauri::command]
fn history_clear() -> Result<(), String> {
    history::clear()
}

#[tauri::command]
fn history_totals() -> (u64, u64) {
    history::totals()
}

#[tauri::command]
fn schedule_set(frequency: String, time: String, enabled: bool) -> Result<(), String> {
    if enabled {
        schedule::create(&frequency, &time).map(|_| ())
    } else {
        schedule::remove()
    }
}

#[tauri::command]
fn schedule_state() -> bool {
    schedule::exists()
}

#[tauri::command]
fn drives() -> Vec<String> {
    util::drives()
}

// ------------------------------------------------------------------ phase 1
#[tauri::command]
async fn monitor_start(name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || monitor::start(&name))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn monitor_finish(name: String) -> Result<monitor::InstallLog, String> {
    tauri::async_runtime::spawn_blocking(move || monitor::finish(&name))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn monitor_list() -> Result<Vec<monitor::InstallLog>, String> {
    tauri::async_runtime::spawn_blocking(monitor::list)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn monitor_uninstall(name: String) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || monitor::uninstall(&name))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn services_list() -> Result<Vec<services::Service>, String> {
    tauri::async_runtime::spawn_blocking(services::list_services)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn service_set(name: String, enabled: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || services::set_service(&name, enabled))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn tasks_list() -> Result<Vec<services::Task>, String> {
    tauri::async_runtime::spawn_blocking(services::list_tasks)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn task_set(name: String, enabled: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || services::set_task(&name, enabled))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn browsers_list() -> Result<Vec<browsers::Browser>, String> {
    tauri::async_runtime::spawn_blocking(browsers::list)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn browser_clean(id: String, categories: Vec<String>) -> Result<browsers::CleanResult, String> {
    tauri::async_runtime::spawn_blocking(move || browsers::clean(&id, &categories))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn junk_scan(limit: usize) -> Result<junk::ScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || junk::scan(limit.max(1)))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn junk_delete(paths: Vec<String>) -> Result<(u64, u64, u64), String> {
    tauri::async_runtime::spawn_blocking(move || junk::delete(&paths))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn hunter_capture() -> serde_json::Value {
    let (name, title, path) = util::hunter_capture();
    serde_json::json!({ "process": name, "title": title, "path": path })
}

#[tauri::command]
async fn forced_uninstall_plan(name: String) -> Result<safety::CleanupPlan, String> {
    tauri::async_runtime::spawn_blocking(move || uninstall::forced_uninstall_plan(&name))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn forced_uninstall_selected(name: String, paths: Vec<String>) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || uninstall::forced_uninstall_selected(&name, &paths))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn monitor_uninstall_plan(name: String) -> Result<safety::CleanupPlan, String> {
    tauri::async_runtime::spawn_blocking(move || monitor::uninstall_plan(&name))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn monitor_uninstall_selected(name: String, paths: Vec<String>) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || monitor::uninstall_selected(&name, &paths))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn quarantine_restore() -> Result<usize, String> {
    safety::quarantine_restore()
}

#[tauri::command]
fn quarantine_empty() -> Result<usize, String> {
    safety::quarantine_empty()
}

#[tauri::command]
async fn program_icons(keys: Vec<String>) -> Result<std::collections::HashMap<String, String>, String> {
    tauri::async_runtime::spawn_blocking(move || uninstall::extract_icons(&keys))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn forced_uninstall(name: String) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || uninstall::forced_uninstall(&name))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn leftovers_level(key: String, level: String) -> Result<Vec<uninstall::Leftover>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let all = uninstall::list();
        match all.into_iter().find(|p| p.key == key) {
            Some(p) => Ok(uninstall::leftovers_with_level(&p, &level)),
            None => Err("program not found".to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
fn save_last_page(state: State<'_, AppState>, page: String) {
    let mut s = state.settings.lock().unwrap();
    if s.last_page != page {
        s.last_page = page;
        let _ = s.save();
    }
}

#[tauri::command]
async fn restore_points() -> Result<Vec<restore::RestorePoint>, String> {
    tauri::async_runtime::spawn_blocking(restore::list)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn restore_create(description: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let desc = if description.trim().is_empty() {
            "VEXORA — نقطة استعادة"
        } else {
            description.as_str()
        };
        restore::create(desc)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn restore_open() -> Result<(), String> {
    restore::open()
}

#[tauri::command]
async fn check_update(force: Option<bool>) -> Result<update::UpdateInfo, String> {
    tauri::async_runtime::spawn_blocking(move || update::check_with(force.unwrap_or(false)))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_update(app: AppHandle, url: String, sha256: String) -> Result<String, String> {
    let path = tauri::async_runtime::spawn_blocking(move || update::download_and_run(&url, &sha256))
        .await
        .map_err(|e| e.to_string())??;
    let _ = app.emit("update:installing", serde_json::json!({ "path": path }));
    Ok(path)
}

#[tauri::command]
fn open_github_page() -> Result<(), String> {
    // Fixed, trusted URL — no user input reaches the shell.
    std::process::Command::new("cmd")
        .args(["/C", "start", "", "https://github.com/ramadanfadel"])
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    // Close both floating windows so the app can exit cleanly.
    if let Some(b) = app.get_webview_window("bubble") {
        let _ = b.close();
    }
    if let Some(w) = app.get_webview_window(WIDGET_LABEL) {
        let _ = w.close();
    }
    app.exit(0);
}

#[tauri::command]
fn show_main_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[tauri::command]
fn set_bubble_visible(app: AppHandle, visible: bool) {
    if let Some(b) = app.get_webview_window("bubble") {
        if visible { let _ = b.show(); } else { let _ = b.hide(); }
    }
}

const WIDGET_LABEL: &str = "vexora-floating-widget";
static WIDGET_LAST_MOVE: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

/// Create the floating widget on demand. Never creates a duplicate window.
fn ensure_widget(app: &AppHandle) {
    if app.get_webview_window(WIDGET_LABEL).is_some() {
        return;
    }
    let s = settings::Settings::load();
    let scale = s.widget_scale.clamp(0.8, 1.5);
    let builder = tauri::WebviewWindowBuilder::new(
        app,
        WIDGET_LABEL,
        tauri::WebviewUrl::App("widget.html".into()),
    )
    .title("VEXORA Widget")
    .inner_size(400.0 * scale, 96.0 * scale)
    .decorations(false)
    .transparent(true)
    .always_on_top(s.widget_always_on_top)
    .skip_taskbar(true)
    .resizable(false)
    .visible(true);
    if let Ok(w) = builder.build() {
        if s.widget_x > i32::MIN / 2 && s.widget_y > i32::MIN / 2 {
            let _ = w.set_position(tauri::PhysicalPosition::new(s.widget_x, s.widget_y));
        }
    }
}

/// Debounced edge snap performed by Rust after a native drag stops. The
/// `Moved` event from the OS is reliable while WebView mouseup is not.
fn widget_snap_after_move(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        if let Ok(last) = WIDGET_LAST_MOVE.lock() {
            if last
                .map(|t| t.elapsed() < std::time::Duration::from_millis(250))
                .unwrap_or(false)
            {
                return;
            }
        }
        let Some(w) = app.get_webview_window(WIDGET_LABEL) else { return };
        let (Ok(pos), Ok(size)) = (w.outer_position(), w.outer_size()) else { return };
        let s = settings::Settings::load();
        let monitors = widget::widget_monitors();
        if monitors.is_empty() { return; }
        let cx = pos.x + size.width as i32 / 2;
        let cy = pos.y + size.height as i32 / 2;
        let configured = s.widget_monitor;
        let mon = if configured >= 0 {
            monitors.get(configured as usize).unwrap_or(&monitors[0])
        } else {
            monitors.iter().find(|m| {
                cx >= m.left && cx <= m.right && cy >= m.top && cy <= m.bottom
            }).unwrap_or(&monitors[0])
        };

        let mut x = pos.x;
        let mut y = pos.y;
        let mut side = "none".to_string();
        if s.widget_snap {
            if let Some(sx) = widget::snap_x(
                x,
                size.width as i32,
                mon.work_left,
                mon.work_right,
                s.widget_snap_distance as i32,
            ) {
                x = sx;
                side = if x <= (mon.work_left + mon.work_right) / 2 { "left" } else { "right" }.into();
            }
        }
        y = widget::clamp_y(y, size.height as i32, mon.work_top, mon.work_bottom, 8);
        if x == pos.x && y == pos.y { return; }
        let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
        let _ = widget_save_bounds(x, y, side.clone());
        let _ = app.emit(
            "widget-snapped",
            serde_json::json!({
                "side": side,
                "workLeft": mon.work_left,
                "workTop": mon.work_top,
                "workRight": mon.work_right,
                "workBottom": mon.work_bottom,
            }),
        );
    });
}

#[tauri::command]
fn widget_set_visible(app: AppHandle, visible: bool) {
    if visible {
        ensure_widget(&app);
        if let Some(w) = app.get_webview_window(WIDGET_LABEL) {
            let _ = w.show();
            let _ = w.set_focus();
        }
    } else if let Some(w) = app.get_webview_window(WIDGET_LABEL) {
        let _ = w.hide();
    }
    let _ = app.emit("widget-visibility", visible);
}

/// Hiding the widget never exits VEXORA.
#[tauri::command]
fn widget_hide(app: AppHandle) {
    if let Some(w) = app.get_webview_window(WIDGET_LABEL) {
        let _ = w.hide();
    }
    let _ = app.emit("widget-visibility", false);
}

/// Open a main-window page after showing it. Only a safe page id is forwarded.
#[tauri::command]
fn widget_open_page(app: AppHandle, page: String) {
    use tauri::Emitter;
    // Only known page ids may be navigated to.
    const ALLOWED: &[&str] = &[
        "dashboard", "clean", "tune", "registry", "startup", "bgprograms",
        "uninstall", "tools", "history", "settings",
    ];
    if !ALLOWED.contains(&page.as_str()) {
        return;
    }
    let safe = page.as_str();
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
        // Primary path: emit an event the main window listens for. This works
        // even if app.js has not exposed window.go yet.
        let _ = app.emit("widget:navigate", safe);
        // Backwards-compatible fallback for older frontends.
        let _ = w.eval(&format!(
            "if (typeof window.go === 'function') window.go('{}');",
            safe
        ));
    }
}

#[tauri::command]
fn widget_save_bounds(x: i32, y: i32, side: String) -> Result<settings::Settings, String> {
    let mut s = settings::Settings::load();
    s.widget_x = x;
    s.widget_y = y;
    s.widget_side = if side == "left" || side == "right" { side } else { "none".into() };
    s.save()?;
    Ok(s)
}

#[tauri::command]
fn widget_reset_position() -> Result<settings::Settings, String> {
    let mut s = settings::Settings::load();
    s.widget_x = i32::MIN;
    s.widget_y = i32::MIN;
    s.widget_side = "none".into();
    s.save()?;
    Ok(s)
}

#[tauri::command]
fn widget_set_always_on_top(app: AppHandle, on: bool) {
    if let Some(w) = app.get_webview_window(WIDGET_LABEL) {
        let _ = w.set_always_on_top(on);
    }
}

#[tauri::command]
fn widget_set_size(app: AppHandle, width: f64, height: f64) {
    if let Some(w) = app.get_webview_window(WIDGET_LABEL) {
        let _ = w.set_size(tauri::PhysicalSize::new(
            width.max(24.0) as u32,
            height.max(24.0) as u32,
        ));
    }
}

static WIDGET_COLLAPSED: std::sync::Mutex<bool> = std::sync::Mutex::new(false);
static WIDGET_PINNED: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
static WIDGET_OUTSIDE: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

/// Resize/move the widget between its compact horizontal bar and the edge tab.
fn widget_apply_collapsed(app: &AppHandle, collapsed: bool) {
    if let Ok(mut c) = WIDGET_COLLAPSED.lock() { *c = collapsed; }
    let Some(w) = app.get_webview_window(WIDGET_LABEL) else { return };
    let s = settings::Settings::load();
    let scale = s.widget_scale.clamp(0.8, 1.5);
    let (sw, sh) = if collapsed {
        (30.0 * scale, 128.0 * scale)
    } else {
        (400.0 * scale, 96.0 * scale)
    };
    let pos = w.outer_position().unwrap_or(tauri::PhysicalPosition::new(0, 0));
    let x = match s.widget_side.as_str() {
        "right" => {
            let monitors = widget::widget_monitors();
            let m = monitors.iter().find(|m| pos.x >= m.left - 40 && pos.x <= m.right);
            m.map(|m| m.work_right - sw as i32).unwrap_or(pos.x)
        }
        "left" => {
            let monitors = widget::widget_monitors();
            let m = monitors.iter().find(|m| pos.x >= m.left - 40 && pos.x <= m.right);
            m.map(|m| m.work_left).unwrap_or(pos.x)
        }
        _ => pos.x,
    };
    let monitors = widget::widget_monitors();
    let work = monitors.iter().find(|m| pos.x >= m.left - 40 && pos.x <= m.right);
    let y = if let Some(m) = work {
        widget::clamp_y(pos.y, sh as i32, m.work_top, m.work_bottom, 8)
    } else {
        pos.y
    };
    let _ = w.set_size(tauri::PhysicalSize::new(sw as u32, sh as u32));
    let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
    let _ = app.emit("widget-collapse-state", collapsed);
}

/// Reveal the tab temporarily after a click (8 seconds).
#[tauri::command]
fn widget_pin() {
    if let Ok(mut p) = WIDGET_PINNED.lock() {
        *p = Some(std::time::Instant::now());
    }
}

/// Background watcher: hides the snapped bar into an edge tab and reveals it
/// when the real mouse cursor enters the tab. Uses Win32 cursor position.
fn start_widget_watcher(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(120));
        let s = settings::Settings::load();
        let Some(w) = app.get_webview_window(WIDGET_LABEL) else { continue };
        if !s.widget_enabled || w.is_visible().unwrap_or(false) == false { continue; }
        let collapsed = WIDGET_COLLAPSED.lock().map(|c| *c).unwrap_or(false);
        if !s.widget_auto_hide || s.widget_side == "none" {
            if collapsed { widget_apply_collapsed(&app, false); }
            continue;
        }
        let (Ok(pos), Ok(size)) = (w.outer_position(), w.outer_size()) else { continue };
        let (cx, cy) = widget::cursor_physical();
        let inside = cx >= pos.x && cx <= pos.x + size.width as i32
            && cy >= pos.y && cy <= pos.y + size.height as i32;
        let pinned = WIDGET_PINNED.lock().ok().and_then(|p| *p)
            .map(|t| t.elapsed() < std::time::Duration::from_millis(8000))
            .unwrap_or(false);
        if inside {
            if let Ok(mut o) = WIDGET_OUTSIDE.lock() { *o = None; }
            if collapsed { widget_apply_collapsed(&app, false); }
        } else if !collapsed && !pinned {
            let should = if let Ok(mut o) = WIDGET_OUTSIDE.lock() {
                if o.is_none() { *o = Some(std::time::Instant::now()); }
                o.map(|t| t.elapsed())
                    .map(|e| e >= std::time::Duration::from_millis(s.widget_hide_delay_ms as u64))
                    .unwrap_or(false)
            } else { false };
            if should { widget_apply_collapsed(&app, true); }
        }
    });
}

#[tauri::command]
fn export_report(format: String, dest_dir: String) -> Result<String, String> {
    let dir = if dest_dir.trim().is_empty() {
        util::reports_dir()
    } else {
        std::path::PathBuf::from(dest_dir.trim())
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let stamp = util::now_unix();
    let ext = if format.eq_ignore_ascii_case("csv") { "csv" } else { "html" };
    let target = dir.join(format!("VEXORA-report-{stamp}.{ext}"));
    let path = target.to_string_lossy().to_string();
    if ext == "csv" {
        history::export_csv(&path)?;
    } else {
        history::export_html(&path)?;
    }
    Ok(path)
}

#[tauri::command]
fn export_settings(state: State<'_, AppState>, dest_dir: String) -> Result<String, String> {
    let dir = if dest_dir.trim().is_empty() {
        util::data_dir()
    } else {
        std::path::PathBuf::from(dest_dir.trim())
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let target = dir.join(format!("VEXORA-settings-{}.json", util::now_unix()));
    let text = serde_json::to_string_pretty(&*state.settings.lock().unwrap())
        .map_err(|e| e.to_string())?;
    std::fs::write(&target, text).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
fn import_settings(state: State<'_, AppState>, path: String) -> Result<Settings, String> {
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut parsed: Settings = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    parsed.sanitize();
    parsed.save()?;
    *state.settings.lock().unwrap() = parsed.clone();
    Ok(parsed)
}

// ------------------------------------------------------------------ tuning
#[tauri::command]
async fn tweaks_list() -> Result<Vec<tune::TweakRow>, String> {
    tauri::async_runtime::spawn_blocking(tune::rows)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn tweak_set(id: String, on: bool) -> Result<(), String> {
    let out = tauri::async_runtime::spawn_blocking({
        let id = id.clone();
        move || tune::apply(&id, on)
    })
    .await
    .map_err(|e| e.to_string())?;
    log::write(
        "tweak",
        &format!("{} {}", id, if on { "تفعيل" } else { "إلغاء" }),
    );
    out.map(|_| ())
}

#[tauri::command]
async fn tweak_change_log() -> Result<Vec<tune::TweakChange>, String> {
    tauri::async_runtime::spawn_blocking(tune::change_log)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn tweak_restore(id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || tune::restore_last(&id))
        .await
        .map_err(|e| e.to_string())?
        .map(|_| ())
}

// ------------------------------------------------------------------ boost
#[tauri::command]
async fn free_memory() -> Result<(u64, u32), String> {
    tauri::async_runtime::spawn_blocking(boost::trim_memory)
        .await
        .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ ram optimizer
#[tauri::command]
async fn ram_status() -> Result<ramopt::RamStatus, String> {
    tauri::async_runtime::spawn_blocking(ramopt::status)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ram_clean() -> Result<(u64, u32), String> {
    tauri::async_runtime::spawn_blocking(ramopt::clean)
        .await
        .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ process icons
#[tauri::command]
async fn process_icons(exes: Vec<String>) -> Result<std::collections::HashMap<String, String>, String> {
    tauri::async_runtime::spawn_blocking(move || boost::extract_process_icons(&exes))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn processes(limit: usize) -> Result<Vec<boost::ProcInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || boost::process_list(limit))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn kill_process(pid: u32) -> bool {
    boost::kill(pid)
}

#[tauri::command]
async fn safe_close_process(pid: u32) -> Result<boost::CloseResult, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(boost::safe_close(pid)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn force_kill_process(pid: u32) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(boost::force_kill(pid)))
        .await
        .map_err(|e| e.to_string())?
}

// ------------------------------------------------------------------ background monitor
#[tauri::command]
fn bg_monitor_status() -> (bool, u32, f64, u32, u32) {
    bgmonitor::status()
}

#[tauri::command]
async fn bg_monitor_apply(settings: settings::Settings) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        settings.save().map_err(|e| e.to_string())?;
        bgmonitor::apply_settings(&settings);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn classify_process_app(pid: u32) -> Result<boost::AppClassification, String> {
    Ok(boost::classify_process(pid))
}

#[tauri::command]
async fn disk_health() -> Result<Vec<boost::DiskHealth>, String> {
    tauri::async_runtime::spawn_blocking(boost::disk_health)
        .await
        .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ bootstrap
/// Ask for administrator rights once at launch when enabled in settings.
/// `VEXORA_NO_ELEVATE=1` disables it (used by automated tests), and
/// `--autoclean` never elevates so the scheduled task stays silent.
pub fn elevate_if_needed() {
    if std::env::var("VEXORA_NO_ELEVATE").is_ok() {
        return;
    }
    if std::env::args().any(|a| a == "--autoclean" || a == "--elevated") {
        return;
    }
    if util::is_admin() {
        return;
    }
    let s = settings::Settings::load();
    if !s.auto_elevate {
        return;
    }
    if util::relaunch_as_admin() {
        std::process::exit(0);
    }
}

/// Computes the adaptive bubble size to match the main window layout, like
/// MS Paint: tall when the window is side by side, wide when it is in the
/// text or centered.
fn bubble_size_for(w: f64, h: f64) -> tauri::LogicalSize<f64> {
    if h >= w {
        tauri::LogicalSize::new(78.0, 192.0)
    } else {
        tauri::LogicalSize::new(192.0, 78.0)
    }
}

pub fn run() {
    let app = tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_catalog,
            get_settings,
            save_settings,
            get_health,
            system_specs,
            scan_cleaners,
            clean_cleaners,
            cancel_work,
            registry_scan,
            registry_fix,
            registry_backup_only,
            startup_list,
            startup_extras,
            startup_set,
            startup_remove,
            programs_list,
            program_uninstall,
            delete_leftovers,
            leftover_scan,
            program_remove_entry,
            program_leftovers,
            dup_scan,
            dup_cancel,
            big_scan,
            empty_scan,
            system_repair_run,
            system_repair_cancel,
            spooler_status,
            spooler_fix,
            ctxmenu_list,
            ctxmenu_set,
            ctxmenu_win11_classic,
            unlocker_list,
            shred_path,
            fonts_list,
            font_backup,
            font_delete,
            adobe_cache_items,
            adobe_running,
            drivers_backup,
            runtimes_status,
            winget_install,
            drives_list,
            disk_check,
            disk_map,
            delete_paths,
            wipe_free_space,
            flush_dns,
            empty_recycle,
            app_info,
            open_path,
            reveal_path,
            pick_folder,
            pick_file,
            relaunch_admin,
            history_list,
            history_clear,
            history_totals,
            schedule_set,
            schedule_state,
            drives,
            tweaks_list,
            tweak_set,
            tweak_change_log,
            tweak_restore,
            free_memory,
            ram_status,
            ram_clean,
            processes,
            process_icons,
            kill_process,
            safe_close_process,
            force_kill_process,
            battery_status,
            battery_monitor_apply,
            bg_monitor_apply,
            classify_process_app,
            disk_health,
            browsers_list,
            browser_clean,
            junk_scan,
            junk_delete,
            program_icons,
            forced_uninstall_plan,
            forced_uninstall_selected,
            monitor_uninstall_plan,
            monitor_uninstall_selected,
            quarantine_restore,
            quarantine_empty,
            forced_uninstall,
            leftovers_level,
            hunter_capture,
            services_list,
            service_set,
            tasks_list,
            task_set,
            monitor_start,
            monitor_finish,
            monitor_list,
            monitor_uninstall,
            save_last_page,
            restore_points,
            restore_create,
            restore_open,
            check_update,
            install_update,
            hwmon_status,
            open_github_page,
            quit_app,
            show_main_window,
            set_bubble_visible,
            widget::widget_stats,
            widget::widget_monitors,
            widget::widget_set_run_at_startup,
            widget_set_visible,
            widget_hide,
            widget_open_page,
            widget_save_bounds,
            widget_reset_position,
            widget_set_always_on_top,
            widget_set_size,
            widget_pin,
            export_report,
            export_settings,
            import_settings,
        ])
        .setup(|app| {
            let s = settings::Settings::load();
            bgmonitor::init_from_settings();
            let bs = settings::Settings::load();
            battery::apply(bs.battery_monitor_enabled, bs.battery_upper_threshold, bs.battery_lower_threshold);

            // Notify about a new release at startup and keep checking while
            // the app is running (every 10 minutes). Each version is announced
            // only once thanks to the cache/dedupe below.
            {
                use tauri::Emitter;
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    let mut first = true;
                    loop {
                        if !first {
                            std::thread::sleep(std::time::Duration::from_secs(600));
                        }
                        first = false;
                        std::thread::sleep(std::time::Duration::from_secs(3));
                        let info = update::check();
                        if info.available {
                            if let Ok(mut last) = UPDATE_NOTIFIED.lock() {
                                if last.as_deref() != Some(info.latest.as_str()) {
                                    *last = Some(info.latest.clone());
                                    let _ = handle.emit("update:available", &info);
                                }
                            }
                        }
                    }
                });
            }

            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_size(tauri::LogicalSize::new(
                    s.window_w.max(980) as f64,
                    s.window_h.max(660) as f64,
                ));
                let _ = win.show();
            }

            // System tray
            let show_item = tauri::menu::MenuItem::with_id(app, "show", "إظهار VEXORA", true, None::<&str>)?;
            let boost_item = tauri::menu::MenuItem::with_id(app, "boost", "تسريع (تحرير RAM)", true, None::<&str>)?;
            let clean_item = tauri::menu::MenuItem::with_id(app, "clean", "تنظيف سريع", true, None::<&str>)?;
            let monitor_item = tauri::menu::MenuItem::with_id(app, "monitor", "تبديل المراقبة", true, None::<&str>)?;
            let settings_item = tauri::menu::MenuItem::with_id(app, "settings", "الإعدادات", true, None::<&str>)?;
            let quit_item = tauri::menu::MenuItem::with_id(app, "quit", "خروج", true, None::<&str>)?;
            let sep = tauri::menu::PredefinedMenuItem::separator(app)?;
            let menu = tauri::menu::Menu::with_items(app, &[&show_item, &sep, &boost_item, &clean_item, &monitor_item, &sep, &settings_item, &quit_item])?;

            let _tray = tauri::tray::TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "boost" => {
                            let _ = boost::trim_memory();
                        }
                        "clean" => {}
                        "monitor" => {
                            if bgmonitor::is_enabled() {
                                bgmonitor::stop();
                            } else {
                                bgmonitor::start();
                            }
                        }
                        "settings" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                                let _ = w.eval("window.go && window.go('settings')");
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Legacy floating bubble (kept for users who disable the new widget).
            if s.show_bubble && !s.widget_enabled {
                let bubble_size = app
                    .get_webview_window("main")
                    .and_then(|w| w.inner_size().ok())
                    .map(|sz| bubble_size_for(sz.width as f64, sz.height as f64))
                    .unwrap_or_else(|| tauri::LogicalSize::new(192.0, 78.0));
                let _ = tauri::WebviewWindowBuilder::new(
                    app,
                    "bubble",
                    tauri::WebviewUrl::App("bubble.html".into()),
                )
                .title("VEXORA")
                .inner_size(bubble_size.width, bubble_size.height)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .resizable(false)
                .visible(true)
                .build();
            }

            // VEXORA Smart Floating Bubble — independent window, single instance.
            if s.widget_enabled && s.widget_run_with_app {
                ensure_widget(app.app_handle());
                start_widget_watcher(app.app_handle().clone());
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Safe handling of long-running operations: cancel DISM/SFC
                // gracefully instead of aborting the process abruptly.
                if repair::REPAIR_RUNNING.load(std::sync::atomic::Ordering::Relaxed) {
                    repair::REPAIR_CANCEL.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                if window.label() == "main" {
                    let s = settings::Settings::load();
                    if s.minimize_to_tray {
                        api.prevent_close();
                        let _ = window.hide();
                        return;
                    }
                }
                if window.label() == "main" {
                    if let Ok(size) = window.inner_size() {
                        let scale = window.scale_factor().unwrap_or(1.0);
                        let logical = size.to_logical::<u32>(scale);
                        let mut st = settings::Settings::load();
                        st.window_w = logical.width.max(980);
                        st.window_h = logical.height.max(660);
                        let _ = st.save();
                    }
                }
            }
            // Resize the bubble to match the main window layout.
            if window.label() == "main" {
                if let tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Moved(_) = event {
                    if let Some(bubble) = window.app_handle().get_webview_window("bubble") {
                        if let Ok(sz) = window.inner_size() {
                            let _ = bubble.set_size(bubble_size_for(sz.width as f64, sz.height as f64));
                        }
                    }
                }
            }
            // Notify the floating widget when its native drag ends, so it can
            // perform edge snapping reliably (mouseup is swallowed by the OS drag loop).
            if window.label() == WIDGET_LABEL {
                if let tauri::WindowEvent::Moved(_) = event {
                    if let Ok(mut last) = WIDGET_LAST_MOVE.lock() {
                        *last = Some(std::time::Instant::now());
                    }
                    widget_snap_after_move(window.app_handle().clone());
                }
            }
        })
        .run(tauri::generate_context!());

    if let Err(e) = app {
        eprintln!("fatal: {e}");
    }
}

/// Silent mode used by Task Scheduler: clean every default-enabled *safe*
/// cleaner. Because no one can confirm interactively here, the run is limited
/// to `Danger::Safe` cleaners and always uses the Recycle Bin (no shred, no
/// permanent deletion); everything is written to the audit log.
pub fn run_autoclean() {
    util::migrate_old_data();
    let s = settings::Settings::load();
    log::write("auto", "بدء التنظيف التلقائي المجدول");
    if s.restore_before_clean && util::is_admin() {
        let _ = restore::create("VEXORA — تنظيف تلقائي");
    }
    let opts = clean::CleanOptions {
        recycle: true,
        shred: false,
        passes: s.shred_passes,
        exclusions: s.exclusions.clone(),
    };
    let mut freed = 0u64;
    let mut removed = 0u64;
    let mut skipped = 0u64;
    let mut ran: Vec<String> = Vec::new();
    for c in catalog::cleaners() {
        if c.danger != catalog::Danger::Safe {
            continue;
        }
        if !c.default_on || !s.is_cleaner_enabled(&c.id) {
            continue;
        }
        if c.admin && !util::is_admin() {
            continue;
        }
        let out = clean::clean_one(&c, &opts);
        freed += out.freed;
        removed += out.removed;
        skipped += out.skipped;
        if out.removed > 0 {
            ran.push(format!("{}={}", c.id, out.removed));
        }
    }
    let ran_txt = if ran.is_empty() { "لا يوجد".to_string() } else { ran.join(", ") };
    log::write(
        "auto",
        &format!(
            "انتهى: {} عنصراً، {} بايت، {} تخطي | {ran_txt}",
            removed, freed, skipped
        ),
    );
    history::add(
        "auto",
        "تنظيف تلقائي",
        &format!("{removed} عنصراً"),
        freed,
        removed,
    );
}
