//! Persistent user settings stored as JSON under %APPDATA%.

use serde::{Deserialize, Serialize};

use crate::util;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Schedule {
    pub enabled: bool,
    pub frequency: String,
    pub time: String,
    pub last_run: u64,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: "weekly".into(),
            time: "03:00".into(),
            last_run: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub language: String,
    pub theme: String,
    pub confirm_clean: bool,
    pub preview_first: bool,
    pub recycle_bin: bool,
    pub shred: bool,
    pub shred_passes: u32,
    pub exclusions: Vec<String>,
    pub disabled_cleaners: Vec<String>,
    pub schedule: Schedule,
    pub last_page: String,
    pub window_w: u32,
    pub window_h: u32,
    pub first_run: bool,
    pub auto_elevate: bool,
    pub restore_before_clean: bool,
    pub restore_before_tweaks: bool,
    pub last_restore: u64,
    pub update_url: String,
    /// GitHub repository used as the update source, e.g. `ramadanfadel/VEXORA`.
    pub github_repo: String,
    // ---- Appearance Studio (Phase 1: stored, previewed, not yet applied) ----
    pub appearance_layout: String,
    pub appearance_accent: String,
    pub appearance_mode: String,
    pub appearance_glow: bool,
    pub appearance_animation: bool,
    /// Independent migration flag; never derived from `first_run`.
    pub appearance_migrated: bool,
    pub minimize_to_tray: bool,
    pub show_bubble: bool,
    // ---- Background resource monitoring ----
    pub bg_monitor_enabled: bool,
    pub bg_monitor_interval_min: u32,
    pub bg_monitor_mem_threshold: u32,
    pub bg_monitor_cpu_threshold: u32,
    pub bg_monitor_auto_close: bool,
    pub bg_monitor_auto_clean_temp: bool,
    pub bg_monitor_allowed_close: Vec<String>,
    pub bg_monitor_run_with_windows: bool,
    // ---- Battery health & smart charge guard ----
    pub battery_monitor_enabled: bool,
    pub battery_upper_threshold: u32,
    pub battery_lower_threshold: u32,
    // ---- VEXORA Smart Floating Bubble ----
    pub widget_enabled: bool,
    pub widget_run_with_app: bool,
    pub widget_run_with_windows: bool,
    pub widget_always_on_top: bool,
    pub widget_auto_hide: bool,
    pub widget_opacity: f64,
    pub widget_scale: f64,
    pub widget_refresh_ms: u32,
    pub widget_snap: bool,
    pub widget_snap_distance: u32,
    pub widget_hide_delay_ms: u32,
    pub widget_monitor: i32,
    pub widget_x: i32,
    pub widget_y: i32,
    pub widget_side: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "ar".into(),
            theme: "dark".into(),
            confirm_clean: true,
            preview_first: true,
            recycle_bin: true,
            shred: false,
            shred_passes: 1,
            exclusions: Vec::new(),
            disabled_cleaners: vec![
                "privacy.event_logs".into(),
            ],
            schedule: Schedule::default(),
            last_page: "dashboard".into(),
            window_w: 1200,
            window_h: 800,
            first_run: true,
            auto_elevate: true,
            restore_before_clean: true,
            restore_before_tweaks: false,
            last_restore: 0,
            update_url: String::new(),
            github_repo: "ramadanfadel/VEXORA".into(),
            appearance_layout: "classic".into(),
            appearance_accent: "cyan".into(),
            appearance_mode: "dark".into(),
            appearance_glow: true,
            appearance_animation: true,
            appearance_migrated: false,
            minimize_to_tray: true,
            show_bubble: true,
            bg_monitor_enabled: false,
            bg_monitor_interval_min: 30,
            bg_monitor_mem_threshold: 80,
            bg_monitor_cpu_threshold: 80,
            bg_monitor_auto_close: false,
            bg_monitor_auto_clean_temp: false,
            bg_monitor_allowed_close: Vec::new(),
            bg_monitor_run_with_windows: false,
            battery_monitor_enabled: false,
            battery_upper_threshold: 95,
            battery_lower_threshold: 20,
            widget_enabled: true,
            widget_run_with_app: true,
            widget_run_with_windows: false,
            widget_always_on_top: true,
            widget_auto_hide: true,
            widget_opacity: 0.96,
            widget_scale: 1.0,
            widget_refresh_ms: 1500,
            widget_snap: true,
            widget_snap_distance: 28,
            widget_hide_delay_ms: 700,
            widget_monitor: -1,
            widget_x: i32::MIN,
            widget_y: i32::MIN,
            widget_side: "none".into(),
        }
    }
}

fn path() -> std::path::PathBuf {
    util::data_dir().join("settings.json")
}

/// Accept either `owner/repo` or a full GitHub URL, and keep only the
/// `owner/repo` part. Anything else falls back to the default repository.
pub fn normalize_github_repo(input: &str) -> String {
    let mut s = input.trim();
    for prefix in [
        "https://github.com/",
        "http://github.com/",
        "github.com/",
        "https://www.github.com/",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest;
            break;
        }
    }
    s = s.trim().trim_end_matches('/');
    // Drop a path such as /releases/tag/... if a release URL was pasted.
    if let Some(idx) = s.find('/') {
        let tail = &s[idx + 1..];
        if tail.starts_with("releases/") || tail.starts_with("issues/") {
            s = &s[..idx];
        }
    }
    let valid = s.split('/').count() == 2
        && s.split('/').all(|p| {
            !p.is_empty()
                && p.len() <= 100
                && p.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        });
    if valid {
        s.to_string()
    } else {
        "ramadanfadel/VEXORA".into()
    }
}

/// One-time appearance migration. `has_explicit_layout` is true only when the
/// stored JSON actually contains an `appearanceLayout` key. `first_run` is
/// deliberately not used here.
fn migrate_appearance(s: &mut Settings, has_explicit_layout: bool) {
    if s.appearance_migrated {
        return;
    }
    if !has_explicit_layout {
        s.appearance_layout = "grid".into();
    }
    s.appearance_migrated = true;
}

impl Settings {
    pub fn load() -> Self {
        match std::fs::read_to_string(path()) {
            Ok(text) => {
                let has_layout = serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| v.get("appearanceLayout").cloned())
                    .is_some();
                let mut s: Self = serde_json::from_str(&text).unwrap_or_default();
                s.sanitize();
                let was_migrated = s.appearance_migrated;
                migrate_appearance(&mut s, has_layout);
                if !was_migrated {
                    let _ = s.save();
                }
                s
            }
            Err(_) => {
                let mut s = Self::default();
                migrate_appearance(&mut s, false);
                s
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let mut s = self.clone();
        s.sanitize();
        let text = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
        let target = path();
        let tmp = target.with_extension("json.tmp");
        std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &target).map_err(|e| e.to_string())
    }

    pub fn is_cleaner_enabled(&self, id: &str) -> bool {
        !self.disabled_cleaners.iter().any(|d| d == id)
    }

    pub fn sanitize(&mut self) {
        if self.language != "ar" && self.language != "en" { self.language = "ar".into(); }
        if self.theme != "dark" && self.theme != "light" { self.theme = "dark".into(); }
        self.shred_passes = self.shred_passes.clamp(1, 3);
        self.bg_monitor_interval_min = self.bg_monitor_interval_min.clamp(1, 1440);
        self.bg_monitor_mem_threshold = self.bg_monitor_mem_threshold.clamp(1, 100);
        self.bg_monitor_cpu_threshold = self.bg_monitor_cpu_threshold.clamp(1, 100);
        self.widget_opacity = self.widget_opacity.clamp(0.4, 1.0);
        self.widget_scale = self.widget_scale.clamp(0.8, 1.5);
        self.widget_refresh_ms = self.widget_refresh_ms.clamp(1000, 10000);
        self.widget_snap_distance = self.widget_snap_distance.clamp(0, 200);
        self.widget_hide_delay_ms = self.widget_hide_delay_ms.clamp(0, 5000);
        self.window_w = self.window_w.clamp(980, 10000);
        self.window_h = self.window_h.clamp(660, 10000);
        self.battery_upper_threshold = self.battery_upper_threshold.clamp(50, 100);
        self.battery_lower_threshold = self.battery_lower_threshold.clamp(5, 50);
        // sanitize exclusions: drop empty and traversal paths
        self.exclusions = self.exclusions.iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.contains("..") && !s.contains('\0'))
            .collect();
        self.bg_monitor_allowed_close = self.bg_monitor_allowed_close.iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        self.disabled_cleaners = self.disabled_cleaners.iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if self.last_page.is_empty() { self.last_page = "dashboard".into(); }
        if self.update_url.len() > 500 { self.update_url.truncate(500); }
        self.github_repo = normalize_github_repo(&self.github_repo);
        const LAYOUTS: &[&str] = &["classic", "dock", "grid", "glass"];
        const ACCENTS: &[&str] = &["cyan", "purple", "blue", "emerald", "pink", "amber"];
        if !LAYOUTS.contains(&self.appearance_layout.as_str()) {
            self.appearance_layout = "classic".into();
        }
        if !ACCENTS.contains(&self.appearance_accent.as_str()) {
            self.appearance_accent = "cyan".into();
        }
        if self.appearance_mode != "dark" && self.appearance_mode != "light" {
            self.appearance_mode = "dark".into();
        }
    }

    pub fn is_excluded(&self, path: &str) -> bool {
        self.exclusions
            .iter()
            .any(|ex| util::path_matches(ex, path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_github_repo() {
        assert_eq!(normalize_github_repo("ramadanfadel/VEXORA"), "ramadanfadel/VEXORA");
        assert_eq!(
            normalize_github_repo("https://github.com/ramadanfadel/VEXORA/"),
            "ramadanfadel/VEXORA"
        );
        assert_eq!(
            normalize_github_repo("https://github.com/ramadanfadel/VEXORA/releases/tag/VEXORA_1.3.7_x64"),
            "ramadanfadel/VEXORA"
        );
        assert_eq!(normalize_github_repo("bad input"), "ramadanfadel/VEXORA");
    }

    #[test]
    fn appearance_migration_is_one_time_and_respects_explicit_choice() {
        let mut s = Settings::default();
        migrate_appearance(&mut s, false);
        assert_eq!(s.appearance_layout, "grid");
        assert!(s.appearance_migrated);
        // Already migrated: never overwritten again.
        s.appearance_layout = "glass".into();
        migrate_appearance(&mut s, false);
        assert_eq!(s.appearance_layout, "glass");

        let mut old = Settings::default();
        old.appearance_layout = "classic".into();
        migrate_appearance(&mut old, true);
        assert_eq!(old.appearance_layout, "classic");
        assert!(old.appearance_migrated);
    }

    #[test]
    fn appearance_values_are_sanitized() {
        let mut s = Settings::default();
        s.appearance_layout = "malicious()".into();
        s.appearance_accent = "url(javascript:...)".into();
        s.appearance_mode = "neon".into();
        s.sanitize();
        assert_eq!(s.appearance_layout, "classic");
        assert_eq!(s.appearance_accent, "cyan");
        assert_eq!(s.appearance_mode, "dark");
    }
}
