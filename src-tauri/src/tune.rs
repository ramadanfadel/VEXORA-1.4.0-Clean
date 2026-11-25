//! One-click system tuning: performance, privacy and system tweaks.
//! Registry original values are recorded before applying so changes can be restored.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::regutil::{self, Hive};
use crate::util;

#[derive(Clone, Copy, Debug)]
pub enum RegVal {
    Dword(u32),
    Sz(&'static str),
}

pub struct RegOp {
    pub hive: Hive,
    pub key: &'static str,
    pub name: &'static str,
    pub value: RegVal,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrevRegVal {
    pub hive: String,
    pub key: String,
    pub name: String,
    pub existed: bool,
    pub kind: String, // dword | sz | none
    pub dword: Option<u32>,
    pub sz: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TweakChange {
    pub id: String,
    pub at: u64,
    pub action: String, // apply | restore
    pub on: bool,
    pub prev: Vec<PrevRegVal>,
    pub note: String,
}

fn changes_path() -> PathBuf {
    util::data_dir().join("tweak_changes.json")
}

fn load_changes() -> Vec<TweakChange> {
    fs::read_to_string(changes_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_changes(changes: &[TweakChange]) -> Result<(), String> {
    let p = changes_path();
    if let Some(parent) = p.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let s = serde_json::to_string_pretty(changes).map_err(|e| e.to_string())?;
    fs::write(p, s).map_err(|e| e.to_string())
}

fn record_change(change: TweakChange) {
    let mut all = load_changes();
    all.push(change);
    if all.len() > 300 { let drop = all.len()-300; all.drain(0..drop); }
    let _ = save_changes(&all);
}

pub fn change_log() -> Vec<TweakChange> {
    let mut all = load_changes();
    all.sort_by(|a,b| b.at.cmp(&a.at));
    all
}

fn capture_prev(op: &RegOp) -> PrevRegVal {
    let hive = match op.hive { Hive::Hkcu=>"HKEY_CURRENT_USER", Hive::Hklm=>"HKEY_LOCAL_MACHINE", Hive::Hkcr=>"HKEY_CLASSES_ROOT", Hive::Hku=>"HKEY_USERS" }.to_string();
    let full = full_key(op);
    let key_obj = regutil::open(&full);
    if let Some(k) = key_obj {
        if let Ok(v) = k.get_value::<u32, _>(op.name) {
            return PrevRegVal { hive, key: op.key.to_string(), name: op.name.to_string(), existed: true, kind: "dword".into(), dword: Some(v), sz: None };
        }
        if let Ok(v) = k.get_value::<String, _>(op.name) {
            return PrevRegVal { hive, key: op.key.to_string(), name: op.name.to_string(), existed: true, kind: "sz".into(), dword: None, sz: Some(v) };
        }
    }
    PrevRegVal { hive, key: op.key.to_string(), name: op.name.to_string(), existed: false, kind: "none".into(), dword: None, sz: None }
}

fn restore_prev(p: &PrevRegVal) -> bool {
    let full = format!("{}\\{}", p.hive, p.key);
    if !p.existed {
        return regutil::delete_value(&full, &p.name);
    }
    match regutil::create_key(&full) {
        Some(k) => match p.kind.as_str() {
            "dword" => k.set_value(&p.name, &p.dword.unwrap_or(0)).is_ok(),
            "sz" => k.set_value(&p.name, &p.sz.clone().unwrap_or_default()).is_ok(),
            _ => false,
        },
        None => false,
    }
}

pub struct Tweak {
    pub id: &'static str,
    pub ar: &'static str,
    pub en: &'static str,
    pub dar: &'static str,
    pub den: &'static str,
    pub cat: &'static str,
    pub admin: bool,
    pub risk: u8, // 0 safe, 1 caution, 2 advanced
    pub on: Vec<RegOp>,
    pub off: Vec<RegOp>,
    pub cmd_on: &'static [&'static str],
    pub cmd_off: &'static [&'static str],
}

fn d(key: &'static str, name: &'static str, v: u32) -> RegOp {
    RegOp {
        hive: Hive::Hkcu,
        key,
        name,
        value: RegVal::Dword(v),
    }
}

fn dm(key: &'static str, name: &'static str, v: u32) -> RegOp {
    RegOp {
        hive: Hive::Hklm,
        key,
        name,
        value: RegVal::Dword(v),
    }
}

fn s(key: &'static str, name: &'static str, v: &'static str) -> RegOp {
    RegOp {
        hive: Hive::Hkcu,
        key,
        name,
        value: RegVal::Sz(v),
    }
}

pub fn tweaks() -> Vec<Tweak> {
    vec![
        Tweak {
            id: "perf.visual_effects",
            ar: "أفضل أداء للرسوميات",
            en: "Best performance visuals",
            dar: "يوقف الظلال والحركات لتسريع الواجهة",
            den: "Turns off shadows and animations for a snappier UI",
            cat: "performance",
            admin: false,
            risk: 0,
            on: vec![d(
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects",
                "VisualFXSetting",
                2,
            )],
            off: vec![d(
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects",
                "VisualFXSetting",
                0,
            )],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "perf.menu_delay",
            ar: "إلغاء تأخير القوائم",
            en: "Instant menus",
            dar: "إزالة تأخير فتح القوائم (400 مللي ثانية)",
            den: "Remove the 400 ms menu delay",
            cat: "performance",
            admin: false,
            risk: 0,
            on: vec![
                d(r"Control Panel\Desktop", "MenuShowDelay", 0),
                d(r"Control Panel\Desktop", "AutoEndTasks", 1),
                d(r"Control Panel\Desktop", "HungAppTimeout", 1000),
                d(r"Control Panel\Desktop", "WaitToKillAppTimeout", 2000),
            ],
            off: vec![
                d(r"Control Panel\Desktop", "MenuShowDelay", 400),
                d(r"Control Panel\Desktop", "AutoEndTasks", 0),
                d(r"Control Panel\Desktop", "HungAppTimeout", 5000),
                d(r"Control Panel\Desktop", "WaitToKillAppTimeout", 20000),
            ],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "perf.animations",
            ar: "إيقاف حركات النوافذ",
            en: "Disable window animations",
            dar: "إيقاف تصغير/تكبير النوافذ واستعراض المهام",
            den: "Disable minimise/maximise and task view animations",
            cat: "performance",
            admin: false,
            risk: 0,
            on: vec![
                s(r"Control Panel\Desktop\WindowMetrics", "MinAnimate", "0"),
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                    "TaskbarAnimations",
                    0,
                ),
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                    "ListviewAlphaSelect",
                    0,
                ),
            ],
            off: vec![
                s(r"Control Panel\Desktop\WindowMetrics", "MinAnimate", "1"),
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                    "TaskbarAnimations",
                    1,
                ),
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                    "ListviewAlphaSelect",
                    1,
                ),
            ],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "perf.transparency",
            ar: "إيقاف الشفافية",
            en: "Disable transparency",
            dar: "إيقاف تأثير الزجاج في ويندوز 10/11",
            den: "Turn off acrylic/glass effects in Windows 10/11",
            cat: "performance",
            admin: false,
            risk: 0,
            on: vec![d(
                r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "EnableTransparency",
                0,
            )],
            off: vec![d(
                r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "EnableTransparency",
                1,
            )],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "perf.startup_delay",
            ar: "إلغاء تأخير بدء التشغيل",
            en: "No startup delay",
            dar: "تشغيل برامج بدء التشغيل فوراً بدون تأخير 10 ثوانٍ",
            den: "Launch startup apps immediately instead of after 10 s",
            cat: "performance",
            admin: false,
            risk: 0,
            on: vec![d(
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize",
                "StartupDelayInMSec",
                0,
            )],
            off: vec![],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "perf.power_plan",
            ar: "خطة الطاقة: أداء عالٍ",
            en: "High performance power plan",
            dar: "يمنع خفض سرعة المعالج ويحسّن الاستجابة",
            den: "Stops CPU throttling and improves responsiveness",
            cat: "performance",
            admin: true,
            risk: 1,
            on: vec![],
            off: vec![],
            cmd_on: &["powercfg", "/setactive", "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"],
            cmd_off: &["powercfg", "/setactive", "381b4222-f694-41f0-9685-ff5bb260df2e"],
        },
        Tweak {
            id: "perf.game_bar",
            ar: "إيقاف Game Bar و DVR",
            en: "Disable Game Bar / DVR",
            dar: "إيقاف تسجيل الألعاب في الخلفية",
            den: "Stop background game recording",
            cat: "performance",
            admin: false,
            risk: 0,
            on: vec![
                d(r"System\GameConfigStore", "GameDVR_Enabled", 0),
                d(r"Software\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled", 0),
                d(r"Software\Microsoft\GameBar", "AutoGameModeEnabled", 1),
            ],
            off: vec![
                d(r"System\GameConfigStore", "GameDVR_Enabled", 1),
                d(r"Software\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled", 1),
            ],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "perf.background_apps",
            ar: "إيقاف تطبيقات الخلفية",
            en: "Disable background apps",
            dar: "منع تطبيقات المتجر من العمل في الخلفية",
            den: "Stop Store apps from running in the background",
            cat: "performance",
            admin: false,
            risk: 1,
            on: vec![d(
                r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications",
                "GlobalUserDisabled",
                1,
            )],
            off: vec![d(
                r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications",
                "GlobalUserDisabled",
                0,
            )],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "privacy.telemetry",
            ar: "تقليل بيانات التشخيص",
            en: "Minimise telemetry",
            dar: "إرسال أقل قدر ممكن من بيانات الاستخدام لمايكروسوفت",
            den: "Send the minimum diagnostic data to Microsoft",
            cat: "privacy",
            admin: true,
            risk: 1,
            on: vec![dm(
                r"SOFTWARE\Policies\Microsoft\Windows\DataCollection",
                "AllowTelemetry",
                0,
            )],
            off: vec![dm(
                r"SOFTWARE\Policies\Microsoft\Windows\DataCollection",
                "AllowTelemetry",
                3,
            )],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "privacy.ad_id",
            ar: "إيقاف معرّف الإعلانات",
            en: "Disable advertising ID",
            dar: "منع التطبيقات من استخدام معرّفك الإعلاني",
            den: "Stop apps from using your advertising ID",
            cat: "privacy",
            admin: false,
            risk: 0,
            on: vec![d(
                r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo",
                "Enabled",
                0,
            )],
            off: vec![d(
                r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo",
                "Enabled",
                1,
            )],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "privacy.activity_history",
            ar: "إيقاف سجل النشاط",
            en: "Disable activity history",
            dar: "منع ويندوز من تسجيل التطبيقات والملفات المستخدمة",
            den: "Stop Windows from recording used apps and files",
            cat: "privacy",
            admin: false,
            risk: 0,
            on: vec![
                d(r"Software\Microsoft\Windows\CurrentVersion\Privacy", "PublishUserActivities", 0),
                d(r"Software\Microsoft\Windows\CurrentVersion\Privacy", "UploadUserActivities", 0),
            ],
            off: vec![
                d(r"Software\Microsoft\Windows\CurrentVersion\Privacy", "PublishUserActivities", 1),
                d(r"Software\Microsoft\Windows\CurrentVersion\Privacy", "UploadUserActivities", 1),
            ],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "privacy.web_search",
            ar: "إيقاف بحث الويب في القائمة",
            en: "Disable web search in Start",
            dar: "منع إرسال ما تكتبه في قائمة ابدأ إلى الإنترنت",
            den: "Stop Start menu searches from going online",
            cat: "privacy",
            admin: false,
            risk: 0,
            on: vec![d(
                r"Software\Policies\Microsoft\Windows\Explorer",
                "DisableSearchBoxSuggestions",
                1,
            )],
            off: vec![d(
                r"Software\Policies\Microsoft\Windows\Explorer",
                "DisableSearchBoxSuggestions",
                0,
            )],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "privacy.suggestions",
            ar: "إيقاف الاقتراحات والإعلانات",
            en: "Disable tips and suggestions",
            dar: "إيقاف اقتراحات ويندوز والإعلانات في شاشة القفل",
            den: "Turn off Windows tips and suggested content",
            cat: "privacy",
            admin: false,
            risk: 0,
            on: vec![
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    "SubscribedContent-338389Enabled",
                    0,
                ),
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    "SoftLandingEnabled",
                    0,
                ),
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    "SystemPaneSuggestionsEnabled",
                    0,
                ),
            ],
            off: vec![
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    "SubscribedContent-338389Enabled",
                    1,
                ),
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    "SoftLandingEnabled",
                    1,
                ),
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    "SystemPaneSuggestionsEnabled",
                    1,
                ),
            ],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "system.storage_sense",
            ar: "تفعيل Storage Sense",
            en: "Enable Storage Sense",
            dar: "تنظيف تلقائي للملفات المؤقتة وسلة المحذوفات",
            den: "Automatically clean temp files and the recycle bin",
            cat: "system",
            admin: false,
            risk: 0,
            on: vec![
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy",
                    "01",
                    1,
                ),
                d(
                    r"Software\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy",
                    "08",
                    1,
                ),
            ],
            off: vec![d(
                r"Software\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy",
                "01",
                0,
            )],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "system.hibernation",
            ar: "إيقاف الإسبات (Hibernation)",
            en: "Disable hibernation",
            dar: "حذف ملف hiberfil.sys وتحرير مساحة بحجم الذاكرة",
            den: "Delete hiberfil.sys and free RAM-sized disk space",
            cat: "system",
            admin: true,
            risk: 1,
            on: vec![],
            off: vec![],
            cmd_on: &["powercfg", "/hibernate", "off"],
            cmd_off: &["powercfg", "/hibernate", "on"],
        },
        Tweak {
            id: "system.long_paths",
            ar: "دعم المسارات الطويلة",
            en: "Long path support",
            dar: "السماح بمسارات أطول من 260 حرفاً",
            den: "Allow paths longer than 260 characters",
            cat: "system",
            admin: true,
            risk: 0,
            on: vec![dm(
                r"SYSTEM\CurrentControlSet\Control\FileSystem",
                "LongPathsEnabled",
                1,
            )],
            off: vec![dm(
                r"SYSTEM\CurrentControlSet\Control\FileSystem",
                "LongPathsEnabled",
                0,
            )],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "system.remote_assistance",
            ar: "إيقاف المساعدة عن بعد",
            en: "Disable Remote Assistance",
            dar: "إغلاق ثغرة شائعة يستخدمها المحتالون",
            den: "Close a common remote-access attack vector",
            cat: "system",
            admin: true,
            risk: 0,
            on: vec![dm(
                r"SYSTEM\CurrentControlSet\Control\Remote Assistance",
                "fAllowToGetHelp",
                0,
            )],
            off: vec![dm(
                r"SYSTEM\CurrentControlSet\Control\Remote Assistance",
                "fAllowToGetHelp",
                1,
            )],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "system.fast_shutdown",
            ar: "تسريع إغلاق النظام",
            en: "Faster shutdown",
            dar: "تقليل وقت انتظار البرامج عند الإغلاق",
            den: "Reduce the wait for apps during shutdown",
            cat: "system",
            admin: true,
            risk: 1,
            on: vec![dm(
                r"SYSTEM\CurrentControlSet\Control",
                "WaitToKillServiceTimeout",
                2000,
            )],
            off: vec![dm(
                r"SYSTEM\CurrentControlSet\Control",
                "WaitToKillServiceTimeout",
                5000,
            )],
            cmd_on: &[],
            cmd_off: &[],
        },
        Tweak {
            id: "network.tcp_tuning",
            ar: "تحسين شبكة TCP",
            en: "TCP auto-tuning",
            dar: "إعادة ضبط تحسين الشبكة إلى الوضع الطبيعي",
            den: "Reset TCP auto-tuning to the normal profile",
            cat: "system",
            admin: true,
            risk: 1,
            on: vec![],
            off: vec![],
            cmd_on: &["netsh", "int", "tcp", "set", "global", "autotuninglevel=normal"],
            cmd_off: &["netsh", "int", "tcp", "set", "global", "autotuninglevel=disabled"],
        },
    ]
}

fn full_key(op: &RegOp) -> String {
    let hive = match op.hive {
        Hive::Hkcu => "HKEY_CURRENT_USER",
        Hive::Hklm => "HKEY_LOCAL_MACHINE",
        Hive::Hkcr => "HKEY_CLASSES_ROOT",
        Hive::Hku => "HKEY_USERS",
    };
    format!("{hive}\\{}", op.key)
}

fn read_dword(op: &RegOp) -> Option<u32> {
    let k = regutil::open(&full_key(op))?;
    k.get_value::<u32, _>(op.name).ok()
}

fn read_sz(op: &RegOp) -> Option<String> {
    let k = regutil::open(&full_key(op))?;
    k.get_value::<String, _>(op.name).ok()
}

fn apply_op(op: &RegOp) -> bool {
    let full = full_key(op);
    let key = match regutil::create_key(&full) {
        Some(k) => k,
        None => return false,
    };
    match op.value {
        RegVal::Dword(v) => key.set_value(op.name, &v).is_ok(),
        RegVal::Sz(v) => key.set_value(op.name, &v.to_string()).is_ok(),
    }
}

fn op_matches(op: &RegOp) -> bool {
    match op.value {
        RegVal::Dword(v) => read_dword(op) == Some(v),
        RegVal::Sz(v) => read_sz(op).as_deref() == Some(v),
    }
}

fn hiberfil() -> String {
    format!("{}\\hiberfil.sys", std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into()))
}

fn power_plan_high() -> bool {
    match util::run_and_wait("powercfg", &["/getactivescheme"]) {
        Ok(text) => {
            let t = text.to_lowercase();
            t.contains("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c") || t.contains("high performance")
        }
        Err(_) => false,
    }
}

pub fn is_applied(id: &str) -> bool {
    let list = tweaks();
    let t = match list.iter().find(|t| t.id == id) {
        Some(t) => t,
        None => return false,
    };
    if id == "perf.power_plan" {
        return power_plan_high();
    }
    if id == "system.hibernation" {
        return !std::path::Path::new(&hiberfil()).exists();
    }
    if id == "network.tcp_tuning" {
        return false;
    }
    if t.on.is_empty() {
        return false;
    }
    t.on.iter().all(op_matches)
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TweakRow {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub cat: String,
    pub admin: bool,
    pub risk: u8,
    pub applied: bool,
}

pub fn rows() -> Vec<TweakRow> {
    let s = crate::settings::Settings::load();
    let ar = s.language == "ar";
    tweaks()
        .into_iter()
        .map(|t| TweakRow {
            id: t.id.to_string(),
            name: if ar { t.ar } else { t.en }.to_string(),
            desc: if ar { t.dar } else { t.den }.to_string(),
            cat: t.cat.to_string(),
            admin: t.admin,
            risk: t.risk,
            applied: is_applied(t.id),
        })
        .collect()
}

pub fn apply(id: &str, on: bool) -> Result<String, String> {
    let list = tweaks();
    let t = list
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| "unknown tweak".to_string())?;

    if t.admin && !util::is_admin() {
        return Err("admin required".into());
    }

    let ops = if on { &t.on } else { &t.off };
    let prev: Vec<PrevRegVal> = ops.iter().map(capture_prev).collect();
    let mut ok = true;
    for op in ops {
        ok &= apply_op(op);
    }

    let cmds = if on { t.cmd_on } else { t.cmd_off };
    let mut cmd_note = String::new();
    if !cmds.is_empty() {
        let program = cmds[0];
        let args: Vec<String> = cmds[1..].iter().map(|s| s.to_string()).collect();
        util::run_and_wait_vec(program, &args)?;
        cmd_note = format!("cmd:{} {}", program, args.join(" "));
    }

    if !ok && ops.is_empty() && cmds.is_empty() {
        return Err("nothing to do".into());
    }

    record_change(TweakChange {
        id: t.id.to_string(),
        at: util::now_unix(),
        action: "apply".into(),
        on,
        prev,
        note: cmd_note,
    });

    crate::history::add(
        "tweak",
        if on { "Tweak applied" } else { "Tweak reverted" },
        t.id,
        0,
        1,
    );
    Ok(String::from("ok"))
}

pub fn restore_last(id: &str) -> Result<String, String> {
    let mut all = load_changes();
    let pos = all.iter().rposition(|c| c.id == id && c.action == "apply" && !c.prev.is_empty());
    if let Some(i) = pos {
        let prev = all[i].prev.clone();
        let mut ok = true;
        for p in &prev { ok &= restore_prev(p); }
        all.push(TweakChange {
            id: id.to_string(),
            at: util::now_unix(),
            action: "restore".into(),
            on: false,
            prev: vec![],
            note: if ok { "restored previous registry values".into() } else { "restore completed with some failures".into() },
        });
        let _ = save_changes(&all);
        crate::history::add("tweak", "Tweak restored", id, 0, 1);
        if ok { return Ok("ok".into()); }
        return Err("restore completed with some failures".into());
    }
    let target_on = !is_applied(id);
    apply(id, target_on)
}
