//! Declarative catalogue of everything the cleaner knows how to clean.

use crate::util::{appdata, local_appdata, program_data, temp_dir, user_profile, windir};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Files,
    Recycle,
    Command,
    Clipboard,
    RegistryValues,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Danger {
    Safe,
    Caution,
    Advanced,
}

#[derive(Clone, Debug)]
pub struct Cleaner {
    pub id: String,
    pub ar: String,
    pub en: String,
    pub dar: String,
    pub den: String,
    pub cat: String,
    pub paths: Vec<String>,
    pub patterns: Vec<String>,
    pub kind: Kind,
    pub cmd: Vec<String>,
    pub admin: bool,
    pub danger: Danger,
    pub default_on: bool,
    pub depth: Option<usize>,
    pub reg_key: Option<String>,
    pub reg_pattern: Option<String>,
    pub reg_keep: Vec<String>,
}

impl Default for Cleaner {
    fn default() -> Self {
        Self {
            id: String::new(),
            ar: String::new(),
            en: String::new(),
            dar: String::new(),
            den: String::new(),
            cat: "system".into(),
            paths: Vec::new(),
            patterns: vec!["*".into()],
            kind: Kind::Files,
            cmd: Vec::new(),
            admin: false,
            danger: Danger::Safe,
            default_on: true,
            depth: None,
            reg_key: None,
            reg_pattern: None,
            reg_keep: Vec::new(),
        }
    }
}

impl Cleaner {
    fn paths(mut self, p: &[&str]) -> Self {
        self.paths = p.iter().map(|s| s.to_string()).collect();
        self
    }
    fn patterns(mut self, p: &[&str]) -> Self {
        self.patterns = p.iter().map(|s| s.to_string()).collect();
        self
    }
    fn admin(mut self) -> Self {
        self.admin = true;
        self
    }
    fn danger(mut self, d: Danger) -> Self {
        self.danger = d;
        self
    }
    fn off(mut self) -> Self {
        self.default_on = false;
        self
    }
    fn kind(mut self, k: Kind) -> Self {
        self.kind = k;
        self
    }
    fn cmd(mut self, c: &[&str]) -> Self {
        self.cmd = c.iter().map(|s| s.to_string()).collect();
        self
    }
    fn depth(mut self, d: usize) -> Self {
        self.depth = Some(d);
        self
    }
    fn reg(mut self, key: &str, pattern: &str, keep: &[&str]) -> Self {
        self.reg_key = Some(key.to_string());
        self.reg_pattern = Some(pattern.to_string());
        self.reg_keep = keep.iter().map(|s| s.to_string()).collect();
        self
    }
}

macro_rules! cl {
    ($id:expr, $ar:expr, $en:expr, $cat:expr, $dar:expr, $den:expr) => {
        Cleaner {
            id: $id.to_string(),
            ar: $ar.to_string(),
            en: $en.to_string(),
            cat: $cat.to_string(),
            dar: $dar.to_string(),
            den: $den.to_string(),
            ..Default::default()
        }
    };
}

macro_rules! cat_raw {
    ($id:expr, $ar:expr, $en:expr, $cat:expr) => {
        Cleaner {
            id: $id.to_string(),
            ar: $ar.to_string(),
            en: $en.to_string(),
            cat: $cat.to_string(),
            ..Default::default()
        }
    };
}

pub fn categories() -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
    vec![
        ("system", "النظام", "System", "cpu"),
        ("windows", "ويندوز", "Windows", "window"),
        ("browsers", "المتصفحات", "Browsers", "browser"),
        ("apps", "البرامج", "Applications", "apps"),
        ("privacy", "الخصوصية", "Privacy", "shield"),
        ("advanced", "متقدم", "Advanced", "wrench"),
    ]
}

// ------------------------------------------------------------------ browsers
fn chromium(browser: &str, label_ar: &str, label_en: &str, root: &str) -> Vec<Cleaner> {
    let short = browser.to_string();
    let base = format!(r"{}\*\*", root);
    let mk = |id: &str, ar: &str, en: &str, dar: &str, den: &str, paths: Vec<String>| -> Cleaner {
        let mut c = cat_raw!(
            format!("browsers.{short}.{id}"),
            format!("{label_ar}: {ar}"),
            format!("{label_en}: {en}"),
            "browsers"
        );
        c.dar = dar.to_string();
        c.den = den.to_string();
        c.paths = paths;
        c
    };

    let mut out = Vec::new();
    // cache folders
    let caches = [
        ("cache", "ذاكرة التخزين المؤقت", "Cache", "Cache", vec![format!(r"{base}\Cache")]),
        (
            "code_cache",
            "ذاكرة الكود",
            "Code cache",
            "Code Cache",
            vec![format!(r"{base}\Code Cache")],
        ),
        (
            "gpu_cache",
            "ذاكرة كرت الشاشة",
            "GPU cache",
            "GPUCache",
            vec![
                format!(r"{base}\GPUCache"),
                format!(r"{base}\GrShaderCache"),
                format!(r"{base}\ShaderCache"),
                format!(r"{base}\DawnGraphiteCache"),
                format!(r"{base}\DawnWebGPUCache"),
            ],
        ),
        (
            "service_worker",
            "بيانات Service Worker",
            "Service worker data",
            "Service Worker/CacheStorage",
            vec![format!(r"{base}\Service Worker\CacheStorage")],
        ),
        (
            "media_cache",
            "ذاكرة الوسائط",
            "Media cache",
            "Media Cache",
            vec![format!(r"{base}\Media Cache")],
        ),
        (
            "storage",
            "تخزين المواقع المؤقت",
            "Site storage",
            "blob storage",
            vec![
                format!(r"{base}\blob_storage"),
                format!(r"{base}\File System"),
                format!(r"{base}\IndexedDB"),
            ],
        ),
        (
            "session",
            "الجلسة والتبويبات",
            "Session and tabs",
            "Sessions",
            vec![
                format!(r"{base}\Sessions"),
                format!(r"{base}\Session Storage"),
                format!(r"{base}\Current Session"),
                format!(r"{base}\Current Tabs"),
                format!(r"{base}\Last Session"),
                format!(r"{base}\Last Tabs"),
            ],
        ),
        (
            "history",
            "سجل التصفح",
            "Browsing history",
            "History",
            vec![format!(r"{base}\History"), format!(r"{base}\Top Sites"), format!(r"{base}\Visited Links")],
        ),
        (
            "cookies",
            "ملفات تعريف الارتباط",
            "Cookies",
            "Cookies",
            vec![format!(r"{base}\Network\Cookies*"), format!(r"{base}\Cookies*")],
        ),
        (
            "autofill",
            "البيانات المحفوظة في النماذج",
            "Autofill data",
            "Web Data",
            vec![
                format!(r"{base}\Web Data*"),
                format!(r"{base}\AutofillStrikeDatabase*"),
            ],
        ),
        (
            "passwords",
            "كلمات المرور المحفوظة",
            "Saved passwords",
            "Login Data",
            vec![format!(r"{base}\Login Data*"), format!(r"{base}\Login Data For Account*")],
        ),
    ];
    for (id, ar, en, den, paths) in caches {
        let mut c = mk(id, ar, en, ar, den, paths);
        match id {
            "cookies" => c = c.danger(Danger::Caution).off(),
            "passwords" => c = c.danger(Danger::Advanced).off(),
            "autofill" => c = c.danger(Danger::Caution),
            "history" => c = c.danger(Danger::Caution),
            _ => {}
        }
        out.push(c);
    }
    // crash reports / shader cache extras
    out.push(mk(
        "crash",
        "تقارير الأخطاء",
        "Crash reports",
        "Crashpad",
        "Crashpad reports",
        vec![format!(r"{base}\Crashpad\reports"), format!(r"{base}\Crashpad\completed")],
    ));
    out
}

fn firefox() -> Vec<Cleaner> {
    let lad = local_appdata();
    let rad = appdata();
    let mut out = Vec::new();
    let specs: [(&str, &str, &str, Vec<String>, Danger, bool); 9] = [
        (
            "cache",
            "ذاكرة التخزين المؤقت",
            "Cache",
            vec![format!(r"{lad}\Mozilla\Firefox\Profiles\*\cache2")],
            Danger::Safe,
            true,
        ),
        (
            "startup_cache",
            "ذاكرة بدء التشغيل",
            "Startup cache",
            vec![format!(r"{lad}\Mozilla\Firefox\Profiles\*\startupCache")],
            Danger::Safe,
            true,
        ),
        (
            "offline_cache",
            "ذاكرة الأوفلاين",
            "Offline cache",
            vec![format!(r"{lad}\Mozilla\Firefox\Profiles\*\OfflineCache")],
            Danger::Safe,
            true,
        ),
        (
            "history",
            "سجل التصفح",
            "Browsing history",
            vec![
                format!(r"{rad}\Mozilla\Firefox\Profiles\*\places.sqlite*"),
                format!(r"{rad}\Mozilla\Firefox\Profiles\*\favicons.sqlite*"),
            ],
            Danger::Caution,
            true,
        ),
        (
            "cookies",
            "ملفات تعريف الارتباط",
            "Cookies",
            vec![format!(r"{rad}\Mozilla\Firefox\Profiles\*\cookies.sqlite*")],
            Danger::Caution,
            false,
        ),
        (
            "formhistory",
            "سجل النماذج",
            "Form history",
            vec![format!(r"{rad}\Mozilla\Firefox\Profiles\*\formhistory.sqlite*")],
            Danger::Caution,
            true,
        ),
        (
            "downloads",
            "سجل التنزيلات",
            "Download history",
            vec![format!(r"{rad}\Mozilla\Firefox\Profiles\*\downloads.sqlite*")],
            Danger::Caution,
            true,
        ),
        (
            "session",
            "الجلسات",
            "Sessions",
            vec![
                format!(r"{rad}\Mozilla\Firefox\Profiles\*\sessionstore-backups"),
                format!(r"{rad}\Mozilla\Firefox\Profiles\*\sessionstore.jsonlz4"),
            ],
            Danger::Safe,
            true,
        ),
        (
            "passwords",
            "كلمات المرور المحفوظة",
            "Saved passwords",
            vec![
                format!(r"{rad}\Mozilla\Firefox\Profiles\*\logins.json"),
                format!(r"{rad}\Mozilla\Firefox\Profiles\*\key4.db"),
            ],
            Danger::Advanced,
            false,
        ),
    ];
    for (id, ar, en, paths, danger, on) in specs {
        let mut c = cat_raw!(
            format!("browsers.firefox.{id}"),
            format!("فايرفوكس: {ar}"),
            format!("Firefox: {en}"),
            "browsers"
        );
        c.paths = paths;
        c.danger = danger;
        c.default_on = on;
        out.push(c);
    }
    out
}

pub fn cleaners() -> Vec<Cleaner> {
    let win = windir();
    let lad = local_appdata();
    let rad = appdata();
    let pd = program_data();
    let tmp = temp_dir();
    let up = user_profile();
    let sysdrive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());

    let mut v: Vec<Cleaner> = Vec::new();

    // ------------------------------------------------------------- system
    v.push(
        cl!(
            "system.user_temp",
            "الملفات المؤقتة للمستخدم",
            "User temporary files",
            "system",
            "ملفات مؤقتة تتركها البرامج بعد إغلاقها",
            "Temporary files left behind by programs"
        )
        .paths(&[&tmp]),
    );
    v.push(
        cl!(
            "system.windows_temp",
            "مجلد Temp في ويندوز",
            "Windows temporary folder",
            "system",
            "مجلد Windows\\Temp (يتطلب صلاحيات)",
            "Windows\\Temp folder (needs admin)"
        )
        .paths(&[&format!(r"{win}\Temp")])
        .admin(),
    );
    v.push(
        cl!(
            "system.recycle_bin",
            "سلة المحذوفات",
            "Recycle bin",
            "system",
            "إفراغ سلة المحذوفات نهائياً",
            "Empty the recycle bin permanently"
        )
        .kind(Kind::Recycle)
        .danger(Danger::Caution)
        .off(),
    );
    v.push(
        cl!(
            "system.dns_cache",
            "ذاكرة أسماء النطاقات",
            "DNS cache",
            "system",
            "تصفير ذاكرة DNS وقد يحل مشاكل بعض المواقع",
            "Flush the DNS cache"
        )
        .kind(Kind::Command)
        .cmd(&["ipconfig", "/flushdns"]),
    );
    v.push(
        cl!(
            "system.clipboard",
            "محتوى الحافظة",
            "Clipboard",
            "system",
            "مسح آخر شيء تم نسخه",
            "Clear whatever was copied last"
        )
        .kind(Kind::Clipboard),
    );
    v.push(
        cl!(
            "system.thumbnails",
            "ذاكرة الصور المصغرة",
            "Thumbnail cache",
            "system",
            "ويندوز يعيد بناءها تلقائياً عند الحاجة",
            "Windows rebuilds these automatically"
        )
        .paths(&[&format!(r"{lad}\Microsoft\Windows\Explorer")])
        .patterns(&["thumbcache_*.db"])
        .depth(0),
    );
    v.push(
        cl!(
            "system.icon_cache",
            "ذاكرة الأيقونات",
            "Icon cache",
            "system",
            "حذف ذاكرة أيقونات المستكشف",
            "Delete Explorer icon cache"
        )
        .paths(&[&format!(r"{lad}\Microsoft\Windows\Explorer")])
        .patterns(&["iconcache_*.db"])
        .depth(0),
    );
    v.push(
        cl!(
            "system.crash_dumps",
            "ملفات تفريغ الذاكرة",
            "Crash dumps",
            "system",
            "ملفات كبيرة من البرامج المنهارة",
            "Large dumps from crashed applications"
        )
        .paths(&[
            &format!(r"{lad}\CrashDumps"),
            &format!(r"{pd}\Microsoft\Windows\WER\ReportQueue"),
            &format!(r"{pd}\Microsoft\Windows\WER\ReportArchive"),
        ]),
    );
    v.push(
        cl!(
            "system.error_reports",
            "تقارير الأخطاء",
            "Error reports",
            "system",
            "تقارير Windows Error Reporting",
            "Windows Error Reporting data"
        )
        .paths(&[
            &format!(r"{lad}\Microsoft\Windows\WER"),
            &format!(r"{pd}\Microsoft\Windows\WER\Temp"),
        ]),
    );
    v.push(
        cl!(
            "system.directx_cache",
            "ذاكرة DirectX",
            "DirectX shader cache",
            "system",
            "ذاكرة تظليل الألعاب — تُعاد بناؤها",
            "Game shader cache — rebuilt on demand"
        )
        .paths(&[&format!(r"{lad}\D3DSCache")]),
    );
    v.push(
        cl!(
            "system.nvidia_cache",
            "ذاكرة NVIDIA",
            "NVIDIA cache",
            "system",
            "ذاكرة DX و GL الخاصة بكرت NVIDIA",
            "NVIDIA DX/GL shader caches"
        )
        .paths(&[
            &format!(r"{lad}\NVIDIA\DXCache"),
            &format!(r"{lad}\NVIDIA\GLCache"),
            &format!(r"{lad}\NVIDIA Corporation\NV_Cache"),
        ]),
    );
    v.push(
        cl!(
            "system.amd_cache",
            "ذاكرة AMD",
            "AMD cache",
            "system",
            "ذاكرة تظليل كرت AMD",
            "AMD shader cache"
        )
        .paths(&[
            &format!(r"{lad}\AMD\DxCache"),
            &format!(r"{lad}\AMD\DxcCache"),
            &format!(r"{lad}\AMD\GLCache"),
        ]),
    );

    // ------------------------------------------------------------- windows
    v.push(
        cl!(
            "windows.prefetch",
            "ملفات Prefetch",
            "Prefetch data",
            "windows",
            "آثار تشغيل البرامج — يعيد ويندوز بناءها",
            "Startup traces — rebuilt by Windows"
        )
        .paths(&[&format!(r"{win}\Prefetch")])
        .patterns(&["*.pf"])
        .admin()
        .danger(Danger::Caution)
        .off(),
    );
    v.push(
        cl!(
            "windows.update_cache",
            "مخزن تحديثات ويندوز",
            "Windows Update cache",
            "windows",
            "حزم التحديثات المثبتة مسبقاً",
            "Already-installed update payloads"
        )
        .paths(&[&format!(r"{win}\SoftwareDistribution\Download")])
        .admin()
        .danger(Danger::Caution)
        .off(),
    );
    v.push(
        cl!(
            "windows.delivery_optimization",
            "تحسين توزيع التحديثات",
            "Delivery Optimization",
            "windows",
            "ملفات مشاركة التحديثات بين الأجهزة",
            "Peer-to-peer update cache"
        )
        .paths(&[&format!(
            r"{win}\ServiceProfiles\NetworkService\AppData\Local\Microsoft\Windows\DeliveryOptimization"
        )])
        .admin()
        .off(),
    );
    v.push(
        cl!(
            "windows.logs",
            "سجلات ويندوز",
            "Windows log files",
            "windows",
            "سجلات CBS و DISM وملفات ETL",
            "CBS, DISM and ETL logs"
        )
        .paths(&[&format!(r"{win}\Logs"), &format!(r"{win}\Panther")])
        .patterns(&["*.log", "*.etl", "*.cab", "*.old", "*.txt"])
        .admin()
        .danger(Danger::Caution)
        .off(),
    );
    v.push(
        cl!(
            "windows.minidump",
            "ملفات Minidump",
            "Minidumps",
            "windows",
            "ملفات أعطال النظام (الشاشة الزرقاء)",
            "Blue-screen crash dumps"
        )
        .paths(&[
            &format!(r"{win}\Minidump"),
            &format!(r"{win}\LiveKernelReports"),
        ])
        .admin()
        .off(),
    );
    v.push(
        cl!(
            "windows.font_cache",
            "ذاكرة الخطوط",
            "Font cache",
            "windows",
            "ملفات ذاكرة الخطوط القابلة لإعادة البناء",
            "Rebuildable font cache files"
        )
        .paths(&[&format!(
            r"{win}\ServiceProfiles\LocalService\AppData\Local\FontCache"
        )])
        .patterns(&["*.dat"])
        .admin()
        .danger(Danger::Caution)
        .off(),
    );
    v.push(
        cl!(
            "windows.store_cache",
            "ذاكرة متجر التطبيقات",
            "Microsoft Store cache",
            "windows",
            "بيانات مؤقتة لمتجر ويندوز",
            "Temporary Store app data"
        )
        .paths(&[
            &format!(r"{lad}\Packages\Microsoft.WindowsStore_8wekyb3d8bbwe\LocalCache"),
            &format!(r"{lad}\Packages\Microsoft.WindowsStore_8wekyb3d8bbwe\TempState"),
        ]),
    );
    v.push(
        cl!(
            "windows.app_temp_state",
            "بيانات تطبيقات المتجر المؤقتة",
            "Store apps TempState",
            "windows",
            "مجلدات TempState لجميع تطبيقات ويندوز",
            "TempState folders of packaged apps"
        )
        .paths(&[&format!(r"{lad}\Packages\*\TempState")]),
    );
    v.push(
        cl!(
            "windows.old_install",
            "نسخة ويندوز السابقة",
            "Previous Windows installation",
            "windows",
            "مجلد Windows.old — يحرر مساحة كبيرة لكنه يلغي إمكانية الرجوع",
            "Windows.old — frees a lot but blocks rollback"
        )
        .paths(&[&format!(r"{sysdrive}\Windows.old")])
        .admin()
        .danger(Danger::Advanced)
        .off(),
    );

    // ------------------------------------------------------------- privacy
    v.push(
        cl!(
            "privacy.recent_docs",
            "المستندات الحديثة",
            "Recent documents",
            "privacy",
            "اختصارات الملفات التي فتحتها مؤخراً",
            "Shortcuts of recently opened files"
        )
        .paths(&[&format!(r"{rad}\Microsoft\Windows\Recent")])
        .patterns(&["*.lnk", "*.url", "*.automaticDestinations-ms", "*.customDestinations-ms"])
        .danger(Danger::Caution),
    );
    v.push(
        cl!(
            "privacy.jump_lists",
            "قوائم المهام السريعة",
            "Jump lists",
            "privacy",
            "الملفات التي تظهر بالزر الأيمن على أيقونات شريط المهام",
            "Files shown when right-clicking taskbar icons"
        )
        .paths(&[&format!(r"{rad}\Microsoft\Windows\Recent\AutomaticDestinations"), &format!(r"{rad}\Microsoft\Windows\Recent\CustomDestinations")])
        .danger(Danger::Caution)
        .off(),
    );
    v.push(
        cl!(
            "privacy.run_history",
            "سجل مربع التشغيل",
            "Run dialog history",
            "privacy",
            "الأوامر المكتوبة في نافذة Run",
            "Commands typed into the Run dialog"
        )
        .kind(Kind::RegistryValues)
        .reg(
            r"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU",
            "*",
            &["MRUList"]
        )
        .danger(Danger::Caution),
    );
    v.push(
        cl!(
            "privacy.typed_paths",
            "المسارات المكتوبة",
            "Typed paths",
            "privacy",
            "المسارات المدخلة في شريط عنوان المستكشف",
            "Paths typed into Explorer's address bar"
        )
        .kind(Kind::RegistryValues)
        .reg(
            r"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths",
            "url*",
            &[]
        )
        .danger(Danger::Caution),
    );
    v.push(
        cl!(
            "privacy.notifications",
            "ذاكرة الإشعارات",
            "Notification cache",
            "privacy",
            "الإشعارات القديمة المحفوظة",
            "Stored notification history"
        )
        .paths(&[&format!(r"{lad}\Microsoft\Windows\Notifications")])
        .patterns(&["*.db", "*.db-wal", "*.db-shm"]),
    );
    v.push(
        cl!(
            "privacy.timeline",
            "سجل الأنشطة (Timeline)",
            "Activity timeline",
            "privacy",
            "سجل التطبيقات والملفات المستخدمة",
            "History of used apps and files"
        )
        .paths(&[&format!(r"{lad}\ConnectedDevicesPlatform\L.*\ActivitiesCache.db*")])
        .danger(Danger::Advanced)
        .off(),
    );
    v.push(
        cl!(
            "privacy.event_logs",
            "سجلات الأحداث",
            "Event logs",
            "privacy",
            "حذف سجلات أحداث ويندوز (لا رجعة)",
            "Clear Windows event logs (irreversible)"
        )
        .kind(Kind::Command)
        .cmd(&[
            "wevtutil", "el", "Application", "Security", "Setup", "System",
        ])
        .admin()
        .danger(Danger::Advanced)
        .off(),
    );

    // ------------------------------------------------------------- browsers
    v.extend(chromium(
        "chrome",
        "كروم",
        "Chrome",
        &format!(r"{lad}\Google\Chrome\User Data"),
    ));
    v.extend(chromium(
        "edge",
        "إيدج",
        "Edge",
        &format!(r"{lad}\Microsoft\Edge\User Data"),
    ));
    v.extend(chromium(
        "brave",
        "بريف",
        "Brave",
        &format!(r"{lad}\BraveSoftware\Brave-Browser\User Data"),
    ));
    v.extend(chromium(
        "vivaldi",
        "فيفالدي",
        "Vivaldi",
        &format!(r"{lad}\Vivaldi\User Data"),
    ));
    v.extend(chromium(
        "opera",
        "أوبرا",
        "Opera",
        &format!(r"{rad}\Opera Software\Opera Stable"),
    ));
    v.extend(chromium(
        "operagx",
        "أوبرا GX",
        "Opera GX",
        &format!(r"{rad}\Opera Software\Opera GX Stable"),
    ));
    v.extend(firefox());

    // ------------------------------------------------------------- apps
    let app_specs: Vec<(&str, &str, &str, &str, &str, Vec<String>, Danger, bool)> = vec![
        (
            "apps.office_recent",
            "أوفيس: الملفات الحديثة",
            "Office: recent files",
            "قائمة الملفات الحديثة في أوفيس",
            "Recent file list in Office",
            vec![
                format!(r"{rad}\Microsoft\Office\Recent"),
                format!(r"{lad}\Microsoft\Office\16.0\OfficeFileCache"),
                format!(r"{lad}\Microsoft\Office\15.0\OfficeFileCache"),
            ],
            Danger::Caution,
            true,
        ),
        (
            "apps.teams",
            "تيمز: الذاكرة المؤقتة",
            "Teams: cache",
            "ذاكرة مايكروسوفت تيمز",
            "Microsoft Teams caches",
            vec![
                format!(r"{rad}\Microsoft\Teams\Cache"),
                format!(r"{rad}\Microsoft\Teams\GPUCache"),
                format!(r"{rad}\Microsoft\Teams\Code Cache"),
                format!(r"{rad}\Microsoft\Teams\blob_storage"),
                format!(r"{lad}\Packages\MSTeams_8wekyb3d8bbwe\LocalCache"),
                format!(r"{lad}\Packages\MSTeams_8wekyb3d8bbwe\TempState"),
            ],
            Danger::Safe,
            true,
        ),
        (
            "apps.discord",
            "ديسكورد: الذاكرة المؤقتة",
            "Discord: cache",
            "ذاكرة ديسكورد",
            "Discord caches",
            vec![
                format!(r"{rad}\discord\Cache"),
                format!(r"{rad}\discord\Code Cache"),
                format!(r"{rad}\discord\GPUCache"),
                format!(r"{rad}\discord\Crashpad"),
            ],
            Danger::Safe,
            true,
        ),
        (
            "apps.slack",
            "سلاك: الذاكرة المؤقتة",
            "Slack: cache",
            "ذاكرة سلاك",
            "Slack caches",
            vec![
                format!(r"{rad}\Slack\Cache"),
                format!(r"{rad}\Slack\Code Cache"),
                format!(r"{rad}\Slack\GPUCache"),
            ],
            Danger::Safe,
            true,
        ),
        (
            "apps.zoom",
            "زووم: البيانات المؤقتة",
            "Zoom: temp data",
            "بيانات زووم المؤقتة",
            "Zoom temporary data",
            vec![
                format!(r"{rad}\Zoom\data"),
                format!(r"{rad}\Zoom\logs"),
            ],
            Danger::Safe,
            true,
        ),
        (
            "apps.vscode",
            "VS Code: الذاكرة المؤقتة",
            "VS Code: cache",
            "ذاكرة محرر VS Code",
            "VS Code caches",
            vec![
                format!(r"{rad}\Code\Cache"),
                format!(r"{rad}\Code\CachedData"),
                format!(r"{rad}\Code\GPUCache"),
                format!(r"{rad}\Code\logs"),
            ],
            Danger::Safe,
            true,
        ),
        (
            "apps.visualstudio",
            "Visual Studio: الذاكرة المؤقتة",
            "Visual Studio: caches",
            "ذاكرات ونسخ Visual Studio",
            "Visual Studio caches",
            vec![
                format!(r"{lad}\Microsoft\VisualStudio\*\ComponentModelCache"),
                format!(r"{lad}\Microsoft\VisualStudio\*\Extensions\*Cache*"),
                format!(r"{lad}\Temp\VSFeedbackIntelliCodeLogs"),
            ],
            Danger::Safe,
            false,
        ),
        (
            "apps.spotify",
            "سبوتيفاي: الذاكرة المؤقتة",
            "Spotify: cache",
            "ذاكرة سبوتيفاي",
            "Spotify caches",
            vec![
                format!(r"{lad}\Spotify\Storage"),
                format!(r"{lad}\Spotify\Data"),
            ],
            Danger::Safe,
            true,
        ),
        (
            "apps.steam",
            "ستيم: ذاكرة الويب",
            "Steam: web cache",
            "ذاكرة متصفح ستيم",
            "Steam browser cache",
            vec![
                format!(r"{lad}\Steam\htmlcache"),
                format!(r"{lad}\Steam\logs"),
            ],
            Danger::Safe,
            true,
        ),
        (
            "apps.uplay",
            "يوبلاي: السجلات",
            "Ubisoft Connect: logs",
            "سجلات يوبلاي",
            "Ubisoft Connect logs",
            vec![format!(r"{lad}\Ubisoft Game Launcher\logs")],
            Danger::Safe,
            false,
        ),
        (
            "apps.adobe_cache",
            "أدوبي: ذاكرة الوسائط",
            "Adobe: media cache",
            "ذاكرة الوسائط لأدوبي",
            "Adobe media cache",
            vec![
                format!(r"{lad}\Adobe\Common\Media Cache"),
                format!(r"{lad}\Adobe\Common\Media Cache Files"),
            ],
            Danger::Safe,
            true,
        ),
        (
            "apps.java_cache",
            "جافا: الذاكرة المؤقتة",
            "Java: cache",
            "ذاكرة جافا",
            "Java deployment cache",
            vec![format!(r"{lad}\Sun\Java\Deployment\cache")],
            Danger::Caution,
            true,
        ),
        (
            "apps.winrar_temp",
            "WinRAR: الملفات المؤقتة",
            "WinRAR: temp files",
            "ملفات WinRAR المؤقتة",
            "WinRAR temporary files",
            vec![format!(r"{rad}\WinRAR\Temp")],
            Danger::Safe,
            true,
        ),
        (
            "apps.pip_cache",
            "Python: ذاكرة pip",
            "Python: pip cache",
            "حزم بايثون المنزلة مسبقاً",
            "Previously downloaded Python packages",
            vec![format!(r"{lad}\pip\Cache")],
            Danger::Caution,
            false,
        ),
        (
            "apps.npm_cache",
            "Node: ذاكرة npm",
            "Node: npm cache",
            "حزم Node المنزلة مسبقاً",
            "Previously downloaded Node packages",
            vec![format!(r"{lad}\npm-cache")],
            Danger::Caution,
            false,
        ),
        (
            "apps.onedrive_logs",
            "ون درايف: السجلات",
            "OneDrive: logs",
            "سجلات مزامنة ون درايف",
            "OneDrive sync logs",
            vec![format!(r"{lad}\Microsoft\OneDrive\logs")],
            Danger::Safe,
            true,
        ),
        (
            "apps.google_drive",
            "جوجل درايف: الذاكرة",
            "Google Drive: cache",
            "ذاكرة جوجل درايف",
            "Google Drive cache",
            vec![format!(r"{lad}\Google\DriveFS\*\content_cache")],
            Danger::Caution,
            false,
        ),
        (
            "apps.edgewebview",
            "تطبيقات WebView2",
            "WebView2 apps",
            "ذاكرة تطبيقات تعتمد على WebView2",
            "WebView2 runtime caches",
            vec![format!(r"{lad}\Microsoft\EdgeWebView\Application\*\EBWebView\Default\Cache")],
            Danger::Safe,
            true,
        ),
        (
            "apps.whatsapp",
            "واتساب: الذاكرة المؤقتة",
            "WhatsApp: cache",
            "ذاكرة واتساب لسطح المكتب",
            "WhatsApp desktop cache",
            vec![format!(r"{rad}\WhatsApp\Cache"), format!(r"{rad}\WhatsApp\GPUCache")],
            Danger::Safe,
            true,
        ),
    ];
    for (id, ar, en, dar, den, paths, danger, on) in app_specs {
        let mut c = cat_raw!(id, ar, en, "apps");
        c.dar = dar.to_string();
        c.den = den.to_string();
        c.paths = paths;
        c.danger = danger;
        c.default_on = on;
        v.push(c);
    }

    // ------------------------------------------------------------- advanced
    v.push(
        cl!(
            "advanced.all_user_temp",
            "مجلدات Temp لكل المستخدمين",
            "All users temp folders",
            "advanced",
            "مجلدات Temp لكل حسابات ويندوز",
            "Temp folders of every Windows account"
        )
        .paths(&[&format!(r"{sysdrive}\Users\*\AppData\Local\Temp")])
        .admin()
        .danger(Danger::Advanced)
        .off(),
    );
    v.push(
        cl!(
            "advanced.shellbags",
            "ShellBags (سجل المجلدات)",
            "ShellBags",
            "advanced",
            "آثار فتح المجلدات في المستكشف",
            "Explorer folder access traces"
        )
        .kind(Kind::RegistryValues)
        .reg(
            r"HKEY_CURRENT_USER\Software\Microsoft\Windows\Shell\BagMRU",
            "*",
            &[]
        )
        .danger(Danger::Advanced)
        .off(),
    );
    v.push(
        cl!(
            "advanced.public_temp",
            "مجلد Temp العام",
            "Public temp folder",
            "advanced",
            "ملفات مؤقتة مشتركة بين المستخدمين",
            "Shared temporary files"
        )
        .paths(&[&format!(r"{pd}\Temp"), &format!(r"{up}\..\Public\Documents")])
        .danger(Danger::Caution)
        .off(),
    );

    v
}

pub fn find(id: &str) -> Option<Cleaner> {
    cleaners().into_iter().find(|c| c.id == id)
}
