//! VEXORA Smart Floating Bubble backend.
//!
//! Provides real system stats (CPU, RAM, network byte counters), real monitor
//! work areas (excluding the Windows taskbar) and pure helpers used by the
//! widget UI for edge snapping and taskbar-safe clamping.

use core::ffi::c_void;
use serde::Serialize;
use std::sync::Mutex;
use std::time::Instant;

use windows_sys::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MONITORENUMPROC, MONITORINFOEXW,
};

/// Lightweight live sample shown by the floating widget.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WidgetStats {
    pub cpu: f32,
    pub mem_pct: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    /// Bytes per second (computed from real interface counters).
    pub net_up: f64,
    pub net_down: f64,
}

/// A physical monitor with its true work area (taskbar excluded).
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WidgetMonitor {
    pub name: String,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub work_left: i32,
    pub work_top: i32,
    pub work_right: i32,
    pub work_bottom: i32,
    pub primary: bool,
}

static NET_LAST: Mutex<Option<(Instant, u64, u64)>> = Mutex::new(None);
static MONITORS: Mutex<Vec<WidgetMonitor>> = Mutex::new(Vec::new());

/// Sum of all non-loopback interface octet counters (real Windows counters).
fn net_totals() -> Option<(u64, u64)> {
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        FreeMibTable, GetIfTable2, MIB_IF_TYPE_LOOPBACK, MIB_IF_TABLE2,
    };
    unsafe {
        let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
        if GetIfTable2(&mut table) != 0 || table.is_null() {
            return None;
        }
        let count = (*table).NumEntries as usize;
        let rows = std::slice::from_raw_parts((*table).Table.as_ptr(), count);
        let mut up = 0u64;
        let mut down = 0u64;
        for row in rows {
            if row.Type == MIB_IF_TYPE_LOOPBACK {
                continue;
            }
            up = up.saturating_add(row.OutOctets);
            down = down.saturating_add(row.InOctets);
        }
        FreeMibTable(table.cast::<core::ffi::c_void>());
        Some((up, down))
    }
}

/// Real CPU/RAM/network sample. Network speeds derive from counter deltas.
#[tauri::command]
pub fn widget_stats() -> WidgetStats {
    let (cpu, mem_used, mem_total) = crate::sysinfo::quick_cpu_mem();
    let (mut net_up, mut net_down) = (0.0f64, 0.0f64);

    if let Some((up, down)) = net_totals() {
        let now = Instant::now();
        if let Ok(mut last) = NET_LAST.lock() {
            if let Some((prev, prev_up, prev_down)) = *last {
                let secs = now.duration_since(prev).as_secs_f64();
                if secs > 0.05 {
                    net_up = up.saturating_sub(prev_up) as f64 / secs;
                    net_down = down.saturating_sub(prev_down) as f64 / secs;
                }
            }
            *last = Some((now, up, down));
        }
    }

    let mem_pct = if mem_total > 0 {
        (mem_used as f64 / mem_total as f64 * 100.0).clamp(0.0, 100.0) as f32
    } else {
        0.0
    };

    WidgetStats {
        cpu,
        mem_pct,
        mem_used,
        mem_total,
        net_up,
        net_down,
    }
}

unsafe extern "system" fn enum_monitor(
    hmon: *mut c_void,
    _hdc: *mut c_void,
    _clip: *mut RECT,
    _data: LPARAM,
) -> BOOL {
    let mut info: MONITORINFOEXW = std::mem::zeroed();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    if GetMonitorInfoW(hmon, &mut info.monitorInfo) == 0 {
        return 1;
    }
    let end = info
        .szDevice
        .iter()
        .position(|c| *c == 0)
        .unwrap_or(info.szDevice.len());
    let name = String::from_utf16_lossy(&info.szDevice[..end]);
    let m = info.monitorInfo;
    if let Ok(mut list) = MONITORS.lock() {
        list.push(WidgetMonitor {
            name,
            left: m.rcMonitor.left,
            top: m.rcMonitor.top,
            right: m.rcMonitor.right,
            bottom: m.rcMonitor.bottom,
            work_left: m.rcWork.left,
            work_top: m.rcWork.top,
            work_right: m.rcWork.right,
            work_bottom: m.rcWork.bottom,
            primary: (m.dwFlags & 1) != 0, // MONITORINFOF_PRIMARY
        });
    }
    1
}

/// All monitors with physical-pixel monitor and work areas (taskbar excluded).
#[tauri::command]
pub fn widget_monitors() -> Vec<WidgetMonitor> {
    if let Ok(mut list) = MONITORS.lock() {
        list.clear();
    }
    unsafe {
        let callback: MONITORENUMPROC = Some(enum_monitor);
        EnumDisplayMonitors(std::ptr::null_mut(), std::ptr::null(), callback, 0);
    }
    MONITORS.lock().map(|l| l.clone()).unwrap_or_default()
}

/// Snap target on the horizontal axis, or `None` when the widget is not near
/// either edge. All values are physical pixels.
pub fn snap_x(left: i32, width: i32, work_left: i32, work_right: i32, distance: i32) -> Option<i32> {
    let d = distance.max(0);
    if left <= work_left + d {
        Some(work_left)
    } else if work_right - (left + width) <= d {
        Some(work_right - width)
    } else {
        None
    }
}

/// Keep the widget fully inside the work area (never over the taskbar).
pub fn clamp_y(y: i32, height: i32, work_top: i32, work_bottom: i32, margin: i32) -> i32 {
    let min = work_top + margin;
    let max = (work_bottom - margin - height).max(min);
    y.clamp(min, max)
}

/// Real cursor position in physical pixels, used by the auto-hide watcher.
pub fn cursor_physical() -> (i32, i32) {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut p = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut p); }
    (p.x, p.y)
}

/// Register or remove the optional "run with Windows" startup entry.
#[tauri::command]
pub fn widget_set_run_at_startup(enable: bool) -> Result<(), String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu
        .open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_SET_VALUE,
        )
        .map_err(|e| e.to_string())?;
    if enable {
        let exe = crate::util::exe_path();
        run.set_value("VEXORAWidget", &format!("\"{}\"", exe))
            .map_err(|e| e.to_string())?;
    } else {
        let _ = run.delete_value("VEXORAWidget");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snaps_to_left_edge_only_when_close() {
        assert_eq!(snap_x(2, 300, 0, 1920, 28), Some(0));
        assert_eq!(snap_x(27, 300, 0, 1920, 28), Some(0));
        assert_eq!(snap_x(100, 300, 0, 1920, 28), None);
    }

    #[test]
    fn snaps_to_right_edge_only_when_close() {
        assert_eq!(snap_x(1610, 300, 0, 1920, 28), Some(1620));
        assert_eq!(snap_x(1500, 300, 0, 1920, 28), None);
    }

    #[test]
    fn clamp_never_covers_taskbar() {
        // work area ends at 1040 (taskbar starts there)
        assert_eq!(clamp_y(1200, 96, 0, 1040, 8), 936);
        assert_eq!(clamp_y(-50, 96, 0, 1040, 8), 8);
        assert_eq!(clamp_y(400, 96, 0, 1040, 8), 400);
    }

    #[test]
    fn snap_handles_negative_monitor_origins() {
        assert_eq!(snap_x(-1918, 300, -1920, 0, 28), Some(-1920));
        assert_eq!(snap_x(-10, 300, -1920, 0, 28), Some(-300));
    }

    #[test]
    fn real_monitors_have_work_area() {
        let list = widget_monitors();
        assert!(!list.is_empty(), "no monitors returned by Win32");
        assert!(
            list.iter().any(|m| m.work_right > m.work_left && m.work_bottom > m.work_top),
            "work areas are empty"
        );
    }
}
