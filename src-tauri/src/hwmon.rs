//! Hardware Monitor: temperatures, clock, fan and load.
//!
//! Data sources are deliberately limited to what a standard user can read:
//!   - WMI `Win32_Processor` (CPU load / clock)
//!   - WMI thermal-zone classes (`Win32_PerfFormattedData_Counters_ThermalZoneInformation`,
//!     then `MSAcpi_ThermalZoneTemperature` as fallback)
//!   - `nvidia-smi` for GPU temperature / fan / load when NVIDIA drivers expose it
//!
//! No kernel driver (LibreHardwareMonitorLib) and no Administrator rights are
//! required. A sensor the machine does not expose is rendered as `N/A` rather
//! than guessed.

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HwSensor {
    pub label: String,
    pub metric: String,
    pub value: String,
    pub available: bool,
    /// Raw numeric reading, when the sensor exists (None => N/A).
    pub raw: Option<f64>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub sensors: Vec<HwSensor>,
    pub gpu_name: String,
    pub sources: Vec<String>,
    pub note: String,
}

fn unavailable() -> HardwareInfo {
    let mut sensors = Vec::new();
    for (label, metric) in [
        ("CPU", "Load"),
        ("CPU", "Clock"),
        ("System", "Temperature"),
        ("GPU", "Temperature"),
        ("GPU", "Fan"),
        ("GPU", "Load"),
    ] {
        sensors.push(HwSensor {
            label: label.into(),
            metric: metric.into(),
            value: "N/A".into(),
            available: false,
            raw: None,
        });
    }
    HardwareInfo {
        sensors,
        gpu_name: String::new(),
        sources: Vec::new(),
        note: String::new(),
    }
}

/// Format a raw sensor reading. `None`, NaN and infinity all become `N/A`.
fn format_value(raw: Option<f64>, unit: &str) -> String {
    match raw {
        Some(n) if n.is_finite() => match unit {
            "C" => format!("{:.1} °C", (n * 10.0).round() / 10.0),
            "%" => format!("{:.0} %", n.round()),
            "MHz" => format!("{:.0} MHz", n.round()),
            "" => format!("{:.1}", (n * 10.0).round() / 10.0),
            _ => format!("{:.1} {}", (n * 10.0).round() / 10.0, unit),
        },
        _ => "N/A".into(),
    }
}

/// Parse the JSON produced by the PowerShell collector. Kept pure so the
/// formatting and fallback behaviour can be unit-tested.
fn parse_hardware(text: &str) -> HardwareInfo {
    let v: serde_json::Value = match serde_json::from_str(text.trim()) {
        Ok(x) => x,
        Err(_) => return unavailable(),
    };
    let gpu_name = v
        .get("gpuName")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let sources = v
        .get("sources")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let mut sensors = Vec::new();
    if let Some(arr) = v.get("sensors").and_then(|x| x.as_array()) {
        for item in arr {
            let label = item
                .get("label")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let metric = item
                .get("metric")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let unit = item
                .get("unit")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let raw = item.get("value").and_then(|x| x.as_f64());
            let available = raw.is_some();
            let value = format_value(raw, &unit);
            sensors.push(HwSensor {
                label,
                metric,
                value,
                available,
                raw,
            });
        }
    }
    if sensors.is_empty() {
        return unavailable();
    }
    HardwareInfo {
        sensors,
        gpu_name,
        sources,
        note: "N/A = sensor not exposed by this device or driver.".into(),
    }
}

const COLLECT_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
$sensors = New-Object System.Collections.ArrayList
$sources = New-Object System.Collections.ArrayList
function Add-Sensor([string]$label, [string]$metric, $value, [string]$unit) {
  [void]$sensors.Add([pscustomobject]@{ label=$label; metric=$metric; value=$value; unit=$unit })
}

$cpu = Get-CimInstance -ClassName Win32_Processor | Select-Object -First 1
$load = $null
$clk = $null
if ($cpu) {
  if ($null -ne $cpu.LoadPercentage) { $load = [double]$cpu.LoadPercentage }
  if ($cpu.CurrentClockSpeed -gt 0) { $clk = [double]$cpu.CurrentClockSpeed }
  [void]$sources.Add('Win32_Processor')
}
Add-Sensor 'CPU' 'Load' $load '%'
Add-Sensor 'CPU' 'Clock' $clk 'MHz'

$temp = $null
$perf = Get-CimInstance -Namespace root/cimv2 -ClassName Win32_PerfFormattedData_Counters_ThermalZoneInformation | Select-Object -First 1
if ($perf -and $perf.Temperature -gt 0) {
  $temp = [double]$perf.Temperature - 273.15
  [void]$sources.Add('ThermalZonePerformance')
}
if ($null -eq $temp) {
  $tz = Get-CimInstance -Namespace root/WMI -ClassName MSAcpi_ThermalZoneTemperature | Select-Object -First 1
  if ($tz -and $tz.CurrentTemperature -gt 0) {
    $temp = ([double]$tz.CurrentTemperature / 10.0) - 273.15
    [void]$sources.Add('ACPI')
  }
}
Add-Sensor 'System' 'Temperature' $temp 'C'

$gpuName = ''
$vc = Get-CimInstance -ClassName Win32_VideoController | Select-Object -First 1
if ($vc) { $gpuName = [string]$vc.Name }

$gTemp = $null
$gFan = $null
$gLoad = $null
if (Get-Command nvidia-smi -ErrorAction SilentlyContinue) {
  $nv = & nvidia-smi --query-gpu=temperature.gpu,fan.speed,utilization.gpu --format=csv,noheader,nounits 2>$null
  if ($nv) {
    $parts = @(@($nv)[0] -split ',') | ForEach-Object { $_.Trim() }
    if ($parts.Count -ge 1 -and $parts[0] -match '^\d+(\.\d+)?$') { $gTemp = [double]$parts[0] }
    if ($parts.Count -ge 2 -and $parts[1] -match '^\d+(\.\d+)?$') { $gFan = [double]$parts[1] }
    if ($parts.Count -ge 3 -and $parts[2] -match '^\d+(\.\d+)?$') { $gLoad = [double]$parts[2] }
    if ($null -ne $gTemp -or $null -ne $gFan -or $null -ne $gLoad) { [void]$sources.Add('nvidia-smi') }
  }
}
Add-Sensor 'GPU' 'Temperature' $gTemp 'C'
Add-Sensor 'GPU' 'Fan' $gFan '%'
Add-Sensor 'GPU' 'Load' $gLoad '%'

[pscustomobject]@{
  gpuName = $gpuName
  sources = @($sources)
  sensors = @($sensors)
} | ConvertTo-Json -Depth 4 -Compress
"#;

/// Read all exposed hardware sensors. Never fails: unavailable data becomes
/// `N/A`, so the UI always has a stable set of rows.
pub fn read() -> HardwareInfo {
    let text = match crate::util::run_and_wait(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            COLLECT_SCRIPT,
        ],
    ) {
        Ok(t) => t,
        Err(_) => return unavailable(),
    };
    parse_hardware(&text)
}

/// One point of the in-memory performance history. Nothing is written to disk.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HwSample {
    pub ts: u64,
    pub cpu_temp: Option<f64>,
    pub gpu_temp: Option<f64>,
    pub cpu_load: Option<f64>,
    pub mem_pct: Option<f64>,
}

const HISTORY_LIMIT: usize = 120;
static HISTORY: std::sync::Mutex<std::collections::VecDeque<HwSample>> =
    std::sync::Mutex::new(std::collections::VecDeque::new());

fn reading(info: &HardwareInfo, label: &str, metric: &str) -> Option<f64> {
    info.sensors
        .iter()
        .find(|s| s.label == label && s.metric == metric)
        .and_then(|s| s.raw)
}

/// Append a sample only when the user opens/refreshes Hardware Center.
/// No background polling is started by this module.
pub fn record(info: &HardwareInfo, mem_pct: Option<f64>) {
    let sample = HwSample {
        ts: crate::util::now_unix(),
        cpu_temp: reading(info, "CPU", "Temperature")
            .or_else(|| reading(info, "System", "Temperature")),
        gpu_temp: reading(info, "GPU", "Temperature"),
        cpu_load: reading(info, "CPU", "Load"),
        mem_pct,
    };
    if let Ok(mut h) = HISTORY.lock() {
        h.push_back(sample);
        while h.len() > HISTORY_LIMIT {
            h.pop_front();
        }
    }
}

pub fn history() -> Vec<HwSample> {
    HISTORY
        .lock()
        .map(|h| h.iter().cloned().collect())
        .unwrap_or_default()
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HwAlert {
    pub label: String,
    pub value: f64,
    pub limit: f64,
}

/// Thresholds are checked only from real readings; missing sensors produce no alert.
pub fn evaluate_alerts(info: &HardwareInfo, cpu_max: f64, gpu_max: f64) -> Vec<HwAlert> {
    let mut out = Vec::new();
    if let Some(v) = reading(info, "CPU", "Temperature")
        .or_else(|| reading(info, "System", "Temperature"))
    {
        if v >= cpu_max {
            out.push(HwAlert {
                label: "CPU".into(),
                value: v,
                limit: cpu_max,
            });
        }
    }
    if let Some(v) = reading(info, "GPU", "Temperature") {
        if v >= gpu_max {
            out.push(HwAlert {
                label: "GPU".into(),
                value: v,
                limit: gpu_max,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_and_fallbacks() {
        assert_eq!(format_value(None, "C"), "N/A");
        assert_eq!(format_value(Some(f64::NAN), "%"), "N/A");
        assert_eq!(format_value(Some(52.34), "C"), "52.3 °C");
        assert_eq!(format_value(Some(47.6), "%"), "48 %");
        assert_eq!(format_value(Some(3600.0), "MHz"), "3600 MHz");
    }

    #[test]
    fn parses_mixed_sensors() {
        let json = r#"{
          "gpuName":"NVIDIA GeForce RTX 3060",
          "sources":["Win32_Processor","nvidia-smi"],
          "sensors":[
            {"label":"CPU","metric":"Load","value":12.4,"unit":"%"},
            {"label":"CPU","metric":"Clock","value":3600.0,"unit":"MHz"},
            {"label":"System","metric":"Temperature","value":null,"unit":"C"},
            {"label":"GPU","metric":"Temperature","value":54.0,"unit":"C"},
            {"label":"GPU","metric":"Fan","value":null,"unit":"%"},
            {"label":"GPU","metric":"Load","value":3.0,"unit":"%"}
          ]
        }"#;
        let info = parse_hardware(json);
        assert_eq!(info.gpu_name, "NVIDIA GeForce RTX 3060");
        assert_eq!(info.sources.len(), 2);
        assert_eq!(info.sensors.len(), 6);
        assert_eq!(info.sensors[0].value, "12 %");
        assert_eq!(info.sensors[2].value, "N/A");
        assert!(!info.sensors[2].available);
        assert_eq!(info.sensors[3].value, "54.0 °C");
        assert!(info.sensors[3].available);
        assert!(!info.sensors[4].available);
    }

    #[test]
    fn alerts_use_only_real_readings() {
        let mut info = unavailable();
        if let Some(cpu) = info
            .sensors
            .iter_mut()
            .find(|s| s.label == "System" && s.metric == "Temperature")
        {
            cpu.raw = Some(91.0);
        }
        let alerts = evaluate_alerts(&info, 85.0, 85.0);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].label, "CPU");
        assert!(evaluate_alerts(&unavailable(), 85.0, 85.0).is_empty());
    }

    #[test]
    fn history_is_bounded() {
        let info = unavailable();
        for _ in 0..HISTORY_LIMIT + 25 {
            record(&info, Some(50.0));
        }
        assert_eq!(history().len(), HISTORY_LIMIT);
    }

    #[test]
    fn broken_json_yields_na_rows() {
        let info = parse_hardware("not json");
        assert_eq!(info.sensors.len(), 6);
        assert!(info.sensors.iter().all(|s| !s.available && s.value == "N/A"));
    }
}
