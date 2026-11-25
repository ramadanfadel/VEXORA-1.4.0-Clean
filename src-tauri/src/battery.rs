//! Battery health & smart charge guard.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

static ENABLED: AtomicBool = AtomicBool::new(false);
static UPPER: AtomicU32 = AtomicU32::new(95);
static LOWER: AtomicU32 = AtomicU32::new(20);
static RUNNING: AtomicBool = AtomicBool::new(false);
static ALERTED_UPPER: AtomicBool = AtomicBool::new(false);
static ALERTED_LOWER: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BatteryInfo {
    pub present: bool,
    pub charge_pct: i32,
    pub power_online: bool,
    pub charging: bool,
    pub discharging: bool,
    pub health_pct: i32,
    pub design_capacity: i32,
    pub full_capacity: i32,
    pub cycle_count: i32,
    pub charge_rate_w: f64,
    pub discharge_rate_w: f64,
    pub time_remaining_h: f64,
}

pub fn status() -> BatteryInfo {
    read_battery()
}

pub fn apply(enabled: bool, upper: u32, lower: u32) {
    ENABLED.store(enabled, Ordering::Relaxed);
    UPPER.store(upper.clamp(50, 100), Ordering::Relaxed);
    LOWER.store(lower.clamp(5, 50), Ordering::Relaxed);
    if enabled {
        start();
    } else {
        stop();
    }
}

pub fn start() {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    ENABLED.store(true, Ordering::Relaxed);
    std::thread::spawn(|| {
        loop {
            if !ENABLED.load(Ordering::Relaxed) {
                RUNNING.store(false, Ordering::SeqCst);
                return;
            }
            check_and_alert();
            // sleep 30s in small chunks
            for _ in 0..30 {
                if !ENABLED.load(Ordering::Relaxed) {
                    RUNNING.store(false, Ordering::SeqCst);
                    return;
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    });
}

pub fn stop() {
    ENABLED.store(false, Ordering::Relaxed);
}

fn read_battery() -> BatteryInfo {
    let script = r#"
$b = Get-CimInstance -Namespace root/cimv2 -ClassName Win32_Battery -ErrorAction SilentlyContinue | Select-Object -First 1
$bs = Get-CimInstance -Namespace root/WMI -ClassName BatteryStatus -ErrorAction SilentlyContinue | Select-Object -First 1
$sd = Get-CimInstance -Namespace root/WMI -ClassName BatteryStaticData -ErrorAction SilentlyContinue | Select-Object -First 1
$fc = Get-CimInstance -Namespace root/WMI -ClassName BatteryFullChargedCapacity -ErrorAction SilentlyContinue | Select-Object -First 1
$present = ($null -ne $b) -or ($null -ne $bs)
$charge = 0
if ($fc -and $fc.FullChargedCapacity -gt 0 -and $bs -and $bs.RemainingCapacity -ge 0) {
  $charge = [int](100.0 * $bs.RemainingCapacity / $fc.FullChargedCapacity)
} elseif ($b) {
  $charge = [int]$b.EstimatedChargeRemaining
}
$charge = [Math]::Min(100, [Math]::Max(0, $charge))
$powerOnline = $false; $charging = $false; $discharging = $false
$chargeRate = 0.0; $dischargeRate = 0.0; $remaining = 0
if ($bs) {
  if ($null -ne $bs.PowerOnline) { $powerOnline = [bool]$bs.PowerOnline }
  if ($null -ne $bs.Charging) { $charging = [bool]$bs.Charging }
  if ($null -ne $bs.Discharging) { $discharging = [bool]$bs.Discharging }
  if ($null -ne $bs.ChargeRate) { $chargeRate = [double]$bs.ChargeRate }
  if ($null -ne $bs.DischargeRate) { $dischargeRate = [double]$bs.DischargeRate }
  if ($null -ne $bs.RemainingCapacity) { $remaining = [int]$bs.RemainingCapacity }
}
$design = 0; $full = 0; $cycles = 0
if ($sd) {
  if ($null -ne $sd.DesignedCapacity) { $design = [int]$sd.DesignedCapacity }
  if ($null -ne $sd.CycleCount) { $cycles = [int]$sd.CycleCount }
}
if ($fc -and $null -ne $fc.FullChargedCapacity) { $full = [int]$fc.FullChargedCapacity }
$health = 0
if ($design -gt 0 -and $full -gt 0) { $health = [int](100.0 * $full / $design) }
$health = [Math]::Min(100, [Math]::Max(0, $health))
$timeRem = 0.0
if ($discharging -and $dischargeRate -gt 0 -and $remaining -gt 0) {
  $timeRem = [double]$remaining / [double]$dischargeRate
}
[pscustomobject]@{
  present = $present
  chargePct = $charge
  powerOnline = $powerOnline
  charging = $charging
  discharging = $discharging
  healthPct = $health
  designCapacity = $design
  fullCapacity = $full
  cycleCount = $cycles
  chargeRateW = [Math]::Round($chargeRate / 1000.0, 2)
  dischargeRateW = [Math]::Round($dischargeRate / 1000.0, 2)
  timeRemainingH = [Math]::Round($timeRem, 2)
} | ConvertTo-Json -Compress
"#;
    let text = match crate::util::run_and_wait(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script],
    ) {
        Ok(t) => t,
        Err(_) => return BatteryInfo::default(),
    };
    let trimmed = text.trim();
    if trimmed.is_empty() { return BatteryInfo::default(); }
    let v: serde_json::Value = match serde_json::from_str(trimmed) { Ok(x) => x, Err(_) => return BatteryInfo::default() };
    let b = |k: &str| v.get(k).and_then(|x| x.as_bool()).unwrap_or(false);
    let i = |k: &str| v.get(k).and_then(|x| x.as_i64()).unwrap_or(0) as i32;
    let f = |k: &str| v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0);
    BatteryInfo {
        present: b("present"),
        charge_pct: i("chargePct"),
        power_online: b("powerOnline"),
        charging: b("charging"),
        discharging: b("discharging"),
        health_pct: i("healthPct"),
        design_capacity: i("designCapacity"),
        full_capacity: i("fullCapacity"),
        cycle_count: i("cycleCount"),
        charge_rate_w: f("chargeRateW"),
        discharge_rate_w: f("dischargeRateW"),
        time_remaining_h: f("timeRemainingH"),
    }
}

fn check_and_alert() {
    let info = read_battery();
    if !info.present { return; }
    let upper = UPPER.load(Ordering::Relaxed) as i32;
    let lower = LOWER.load(Ordering::Relaxed) as i32;

    // Upper threshold: plugged in and reached upper.
    if info.power_online && info.charge_pct >= upper {
        if !ALERTED_UPPER.swap(true, Ordering::SeqCst) {
            notify_upper(info.charge_pct, upper);
        }
    } else if info.charge_pct < upper - 5 || !info.power_online {
        ALERTED_UPPER.store(false, Ordering::Relaxed);
    }

    // Lower threshold: discharging and reached lower.
    if info.discharging && info.charge_pct <= lower {
        if !ALERTED_LOWER.swap(true, Ordering::SeqCst) {
            notify_lower(info.charge_pct, lower);
        }
    } else if info.charge_pct > lower + 5 || info.power_online {
        ALERTED_LOWER.store(false, Ordering::Relaxed);
    }
}

fn notify_upper(charge: i32, upper: i32) {
    let text = format!("Battery at {}% (limit {}%). Unplug the charger to preserve battery health.", charge, upper);
    fire_toast("VEXORA Battery", &text, true);
}

fn notify_lower(charge: i32, lower: i32) {
    let text = format!("Battery at {}% (limit {}%). Plug in the charger soon.", charge, lower);
    fire_toast("VEXORA Battery", &text, true);
}

fn fire_toast(title: &str, text: &str, sound: bool) {
    let snd = if sound {
        "Add-Type -AssemblyName System.Media; [System.Media.SystemSounds]::Exclamation.Play();"
    } else { "" };
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; {} $n=New-Object System.Windows.Forms.NotifyIcon; $n.Icon=[System.Drawing.SystemIcons]::Warning; $n.BalloonTipTitle='{}'; $n.BalloonTipText='{}'; $n.Visible=$true; $n.ShowBalloonTip(8000); Start-Sleep -Seconds 9; $n.Dispose()",
        snd, title.replace('\'', "''"), text.replace('\'', "''")
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .spawn();
}
