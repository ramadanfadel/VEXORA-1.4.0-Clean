//! Optional background memory/CPU monitor.
//!
//! Everything here is OFF by default. The monitor never kills processes or
//! forces memory trims on its own unless the user explicitly opts in to the
//! safe auto-actions (and even then only for processes the user listed).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use crate::settings;
use crate::util;

static ENABLED: AtomicBool = AtomicBool::new(false);
static INTERVAL_MIN: AtomicU32 = AtomicU32::new(30);
static MEM_THRESHOLD: AtomicU32 = AtomicU32::new(80);
static CPU_THRESHOLD: AtomicU32 = AtomicU32::new(80);
static AUTO_CLOSE: AtomicBool = AtomicBool::new(false);
static AUTO_CLEAN: AtomicBool = AtomicBool::new(false);
static ALLOWED_CLOSE: Mutex<Vec<String>> = Mutex::new(Vec::new());
static RUNNING: AtomicBool = AtomicBool::new(false);

pub fn init_from_settings() {
    let s = settings::Settings::load();
    ENABLED.store(s.bg_monitor_enabled, Ordering::Relaxed);
    INTERVAL_MIN.store(s.bg_monitor_interval_min.max(1), Ordering::Relaxed);
    MEM_THRESHOLD.store(s.bg_monitor_mem_threshold.clamp(1, 100), Ordering::Relaxed);
    CPU_THRESHOLD.store(s.bg_monitor_cpu_threshold.clamp(1, 100), Ordering::Relaxed);
    AUTO_CLOSE.store(s.bg_monitor_auto_close, Ordering::Relaxed);
    AUTO_CLEAN.store(s.bg_monitor_auto_clean_temp, Ordering::Relaxed);
    if let Ok(mut g) = ALLOWED_CLOSE.lock() {
        *g = s.bg_monitor_allowed_close.clone();
    }
    if s.bg_monitor_enabled {
        start();
    }
}

pub fn apply_settings(s: &settings::Settings) {
    ENABLED.store(s.bg_monitor_enabled, Ordering::Relaxed);
    INTERVAL_MIN.store(s.bg_monitor_interval_min.max(1), Ordering::Relaxed);
    MEM_THRESHOLD.store(s.bg_monitor_mem_threshold.clamp(1, 100), Ordering::Relaxed);
    CPU_THRESHOLD.store(s.bg_monitor_cpu_threshold.clamp(1, 100), Ordering::Relaxed);
    AUTO_CLOSE.store(s.bg_monitor_auto_close, Ordering::Relaxed);
    AUTO_CLEAN.store(s.bg_monitor_auto_clean_temp, Ordering::Relaxed);
    if let Ok(mut g) = ALLOWED_CLOSE.lock() {
        *g = s.bg_monitor_allowed_close.clone();
    }
    if s.bg_monitor_enabled {
        start();
    } else {
        stop();
    }
}

pub fn start() {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return; // already running
    }
    ENABLED.store(true, Ordering::Relaxed);
    std::thread::spawn(|| {
        loop {
            if !ENABLED.load(Ordering::Relaxed) {
                RUNNING.store(false, Ordering::SeqCst);
                return;
            }
            let interval = INTERVAL_MIN.load(Ordering::Relaxed).max(1) as u64;
            // sleep in small chunks so we react to stop quickly
            for _ in 0..(interval * 6) {
                if !ENABLED.load(Ordering::Relaxed) {
                    RUNNING.store(false, Ordering::SeqCst);
                    return;
                }
                std::thread::sleep(Duration::from_secs(10));
            }
            if !ENABLED.load(Ordering::Relaxed) {
                RUNNING.store(false, Ordering::SeqCst);
                return;
            }
            let (mem, cpu) = current_usage();
            let mem_thr = MEM_THRESHOLD.load(Ordering::Relaxed);
            let cpu_thr = CPU_THRESHOLD.load(Ordering::Relaxed);
            if mem >= mem_thr || cpu >= cpu_thr as f64 {
                notify(mem, cpu, mem_thr, cpu_thr);
                if AUTO_CLOSE.load(Ordering::Relaxed) {
                    do_auto_close();
                }
                if AUTO_CLEAN.load(Ordering::Relaxed) {
                    do_auto_clean();
                }
            }
        }
    });
}

pub fn stop() {
    ENABLED.store(false, Ordering::Relaxed);
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn current_usage() -> (u32, f64) {
    let mem = memory_load_percent();
    let cpu = cpu_usage_percent();
    (mem, cpu)
}

pub fn status() -> (bool, u32, f64, u32, u32) {
    let (mem, cpu) = current_usage();
    (
        ENABLED.load(Ordering::Relaxed),
        mem,
        cpu,
        MEM_THRESHOLD.load(Ordering::Relaxed),
        CPU_THRESHOLD.load(Ordering::Relaxed),
    )
}

// ---- helpers ----

fn memory_load_percent() -> u32 {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct MemoryStatusEx {
            dw_length: u32,
            dw_memory_load: u32,
            ull_total_phys: u64,
            ull_avail_phys: u64,
            ull_total_page_file: u64,
            ull_avail_page_file: u64,
            ull_total_virtual: u64,
            ull_avail_virtual: u64,
            ull_avail_extended_virtual: u64,
        }
        extern "system" {
            fn GlobalMemoryStatusEx(lpBuffer: *mut MemoryStatusEx) -> i32;
        }
        let mut ms = MemoryStatusEx {
            dw_length: std::mem::size_of::<MemoryStatusEx>() as u32,
            dw_memory_load: 0,
            ull_total_phys: 0,
            ull_avail_phys: 0,
            ull_total_page_file: 0,
            ull_avail_page_file: 0,
            ull_total_virtual: 0,
            ull_avail_virtual: 0,
            ull_avail_extended_virtual: 0,
        };
        unsafe {
            if GlobalMemoryStatusEx(&mut ms) != 0 {
                return ms.dw_memory_load;
            }
        }
    }
    0
}

fn cpu_usage_percent() -> f64 {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::GetSystemTimes;
    unsafe {
        let mut idle1 = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut kernel1 = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut user1 = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        if GetSystemTimes(&mut idle1, &mut kernel1, &mut user1) == 0 {
            return 0.0;
        }
        let i1 = (idle1.dwLowDateTime as u64) | ((idle1.dwHighDateTime as u64) << 32);
        let k1 = (kernel1.dwLowDateTime as u64) | ((kernel1.dwHighDateTime as u64) << 32);
        let u1 = (user1.dwLowDateTime as u64) | ((user1.dwHighDateTime as u64) << 32);
        std::thread::sleep(Duration::from_millis(200));
        let mut idle2 = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut kernel2 = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut user2 = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        if GetSystemTimes(&mut idle2, &mut kernel2, &mut user2) == 0 {
            return 0.0;
        }
        let i2 = (idle2.dwLowDateTime as u64) | ((idle2.dwHighDateTime as u64) << 32);
        let k2 = (kernel2.dwLowDateTime as u64) | ((kernel2.dwHighDateTime as u64) << 32);
        let u2 = (user2.dwLowDateTime as u64) | ((user2.dwHighDateTime as u64) << 32);
        let idle_delta = i2.saturating_sub(i1) as f64;
        let total_delta = (k2.saturating_sub(k1) + u2.saturating_sub(u1)) as f64;
        if total_delta <= 0.0 {
            return 0.0;
        }
        ((total_delta - idle_delta) / total_delta * 100.0).clamp(0.0, 100.0)
    }
}

fn notify(mem: u32, cpu: f64, mem_thr: u32, cpu_thr: u32) {
    let mut reasons = Vec::new();
    if mem >= mem_thr {
        reasons.push(format!("Memory {}% (limit {}%)", mem, mem_thr));
    }
    if cpu >= cpu_thr as f64 {
        reasons.push(format!("CPU {:.0}% (limit {}%)", cpu, cpu_thr));
    }
    let text = format!(
        "VEXORA: high resource usage — {}. Open VEXORA for safe actions (close a program you choose, or clean safe temp files).",
        reasons.join("; ")
    );
    // Fire-and-forget balloon notification via PowerShell.
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; $n=New-Object System.Windows.Forms.NotifyIcon; $n.Icon=[System.Drawing.SystemIcons]::Warning; $n.BalloonTipTitle='VEXORA'; $n.BalloonTipText='{}'; $n.Visible=$true; $n.ShowBalloonTip(6000); Start-Sleep -Seconds 7; $n.Dispose()",
        text.replace('\'', "''")
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .spawn();
}

fn do_auto_close() {
    let allowed: Vec<String> = ALLOWED_CLOSE.lock().map(|g| g.clone()).unwrap_or_default();
    if allowed.is_empty() {
        return;
    }
    let procs = crate::boost::process_list(500);
    for p in procs {
        if p.system {
            continue;
        }
        let lower = p.name.to_lowercase();
        if allowed.iter().any(|a| lower == a.to_lowercase() || lower.contains(&a.to_lowercase())) {
            let _ = crate::boost::safe_close(p.pid);
        }
    }
}

fn do_auto_clean() {
    // Clean only VEXORA's own temp folder — never the system temp.
    let dir = util::data_dir().join("Temp");
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() {
                let _ = std::fs::remove_file(&p);
            } else if p.is_dir() {
                let _ = std::fs::remove_dir_all(&p);
            }
        }
    }
}