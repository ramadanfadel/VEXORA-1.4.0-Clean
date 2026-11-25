/* VEXORA — UI logic (vanilla JS over the Tauri IPC bridge) */
"use strict";

const TAURI = window.__TAURI__ || {};
const invoke = TAURI.core ? TAURI.core.invoke : async () => {
  throw new Error("Tauri bridge unavailable");
};
const listen = TAURI.event ? TAURI.event.listen : async () => () => {};

/* --------------------------------------------------------------------- i18n */
const DICT = {
  ar: {
    restoreCenter: "مركز الاستعادة",
    restoreCenterHint: "يعرض التغييرات القابلة للتراجع. الحذف النهائي/الكتابة فوق البيانات لا يمكن التراجع عنه.",
    noReversibleChanges: "لا توجد تغييرات قابلة للتراجع مسجلة",
    restoreBeforeTweaksHint: "اختياري. قد يستغرق وقتاً ويحتاج صلاحيات المسؤول وحماية النظام مفعلة. لا يُنشئ نقطة إن تعذّر ذلك.",
    details: "تفاصيل",
    excludePath: "استبعاد المسار",
    excluded: "تم الاستبعاد",
    analysisFor: "نتائج تحليل",
    risk: "الخطر",
    selectedForClean: "محدد للتنظيف",
    noItems: "لا توجد عناصر",
    restorePointBeforeTweaks: "إنشاء نقطة استعادة قبل التعديلات (عند توفرها)",
    changeLog: "سجل التغييرات",
    restoreOriginal: "استعادة الحالة السابقة",
    noChanges: "لا توجد تغييرات مسجلة",
    changeApplied: "تم تطبيق تعديل",
    changeRestored: "تمت استعادة تعديل",
    reversible: "قابل للتراجع",
    notReversible: "غير قابل للتراجع تلقائياً",
    sensitivitySafe: "آمن",
    sensitivityReview: "يحتاج مراجعة",
    sensitivitySensitive: "حساس",
    riskLevel: "مستوى الخطورة",
    reason: "السبب",
    selectForClean: "تضمين في التنظيف",
    analysisResults: "نتائج التحليل",
    pathCol: "المسار",
    partialClean: "تنظيف جزئي",
    failedItems: "عناصر فشلت",
    confirm: "تأكيد",
    "app.title": "VEXORA",
    "app.tagline": "تنظيف أذكى، وأداء أسرع",
    "nav.dashboard": "الرئيسية",
    "nav.clean": "التنظيف",
    "nav.tune": "التسريع والضبط",
    "nav.registry": "الريجستري",
    "nav.startup": "بدء التشغيل",

  "nav.bgprograms": "برامج الخلفية",
    "nav.uninstall": "إلغاء التثبيت",
    "nav.tools": "الأدوات",
    "nav.history": "السجل",
    "nav.settings": "الإعدادات",
    analyze: "تحليل",
    quickClean: "تنظيف سريع",
    clean: "تنظيف",
    scanning: "جارٍ التحليل…",
    cleaning: "جارٍ التنظيف…",
    junkFound: "ملفات يمكن تنظيفها",
    freeSpace: "مساحة متاحة",
    usedSpace: "مستخدم",
    health: "صحة الجهاز",
    lastClean: "آخر تنظيف",
    totalCleaned: "إجمالي ما تم تنظيفه",
    quickActions: "إجراءات سريعة",
    tempClean: "الملفات المؤقتة",
    recycleBin: "سلة المحذوفات",
    dnsFlush: "تصفير DNS",
    boost: "تحرير الذاكرة",
    privacy: "خصوصية",
    welcome: "مرحباً بك",
    subtitle: "حافظ على جهازك سريعاً ونظيفاً وآمناً",
    selectAll: "تحديد الكل",
    selectNone: "إلغاء التحديد",
    clearAll: "إلغاء التحديد",
    selected: "المحدد",
    nothingSelected: "لم تحدد أي عنصر",
    items: "العناصر",
    path: "المسار",
    unknown: "غير معروف",
    nothingFound: "لا توجد ملفات لتنظيفها",
    done: "تم",
    cleanResult: "تم حذف {n} عنصراً وتحرير {size}",
    skipped: "تم تجاوز {n} عنصراً",
    preview: "معاينة التنظيف",
    previewHint: "سيتم حذف هذه العناصر:",
    confirmClean: "هل تريد حذف {n} عنصراً بحجم {size}؟",
    registryScan: "فحص الريجستري",
    registryFix: "إصلاح المحدد",
    backup: "نسخة احتياطية",
    issues: "مشكلة",
    registryClean: "الريجستري نظيف — لا توجد مشاكل",
    fixConfirm: "سيتم إصلاح {n} مشكلة بعد أخذ نسخة احتياطية. متابعة؟",
    tweakConfirm: "سيتم تغيير إعداد داخل ويندوز. هل تريد المتابعة؟ يمكنك التراجع لاحقًا من سجل التغييرات.",
    fixed: "تم إصلاح {n} مشكلة",
    backupAt: "النسخة الاحتياطية: {path}",
    startupTitle: "برامج بدء التشغيل",
    enabled: "مُفعّل",
    disabled: "متوقف",
    delete: "حذف",
    enable: "تفعيل",
    disable: "تعطيل",
    confirmDelete: "هل تريد الحذف نهائياً؟",
    confirmDisable: "هل تريد تعطيل هذا العنصر؟",
    safeToDisable: "آمن للتعطيل",
    essential: "أساسي",
    selectSafe: "تعطيل المقترحات آمنة",
    disableSelected: "تعطيل المحدد",
    confirmDisableBatch: "تعطيل برامج بدء التشغيل المحددة؟",
    programs: "البرامج المثبتة",
    uninstall: "إلغاء التثبيت",
    removeEntry: "حذف المدخل",
    leftovers: "فحص البقايا",
    noLeftovers: "لا توجد بقايا",
    publisher: "الناشر",
    version: "الإصدار",
    installedOn: "تاريخ التثبيت",
    programDetails: "تفاصيل البرنامج",
    search: "بحث…",
    dupes: "الملفات المكررة",
    similarImages: "الصور المتشابهة",
    toolSimilarDesc: "البحث عن الصور المتطابقة والمتشابهة بصرياً",
    bigFiles: "الملفات الكبيرة",
    emptyDirs: "المجلدات الفارغة",
    diskMap: "تحليل المساحة",
    drives: "فحص الأقراص",
    toolDrivesDesc: "افحص الأقراص الصلبة وفلاشات USB: السعة، الصحة، بيانات SMART، وفحص أخطاء للقراءة فقط.",
    driveHealth: "الحالة",
    driveFree: "المتاح",
    driveTotal: "الإجمالي",
    driveType: "النوع",
    driveModel: "الطراز",
    driveScan: "فحص الأخطاء",
    driveScanHint: "فحص للقراءة فقط (chkdsk /scan) — لا يُصلح أو يعدّل شيئًا. قد يحتاج صلاحيات المدير.",
    smartInfo: "SMART",
    removable: "فلاشة USB",
    fixedDisk: "قرص داخلي",
    cloudDisk: "قرص سحابي",
    diskHealth: "صحة القرص",
    driveTemp: "الحرارة",
    driveWear: "الاستهلاك",
    drivePowerOn: "ساعات التشغيل",
    driveStatusClean: "سليم — لا أخطاء",
    driveStatusErrors: "توجد أخطاء — انسخ بياناتك ثم أصلح",
    driveStatusCleaned: "تم تنظيف عناصر",
    driveStatusUnavailable: "تعذّر الفحص (يحتاج صلاحيات المدير أو القرص مستخدم)",
    driveStatusUnknown: "نتيجة غير معروفة",
    driveAdminHint: "بعض بيانات SMART وفحص الأخطاء تحتاج صلاحيات المدير.",
    driveRaw: "تفاصيل الفحص",
    sysRepair: "إصلاح ملفات النظام",
    toolSysRepairDesc: "تشغيل DISM و SFC لإصلاح ملفات النظام التالفة (قد يستغرق 10–30 دقيقة).",
    repairStart: "بدء الإصلاح",
    repairCancel: "إلغاء",
    repairNeedsAdmin: "يتطلب صلاحيات المدير",
    repairClean: "اكتمل — النظام سليم",
    repairErrors: "اكتمل مع أخطاء — راجع التفاصيل",
    repairCancelled: "تم الإلغاء",
    repairStageCheck: "فحص صحة الصورة",
    repairStageRestore: "استعادة صحة الصورة",
    repairStageSfc: "فحص ملفات النظام",
    repairLog: "سجل",
    spooler: "إصلاح طابور الطباعة",
    toolSpoolerDesc: "إيقاف Spooler، حذف المهام المعلقة، إعادة تشغيله.",
    spoolerPending: "المهام المعلقة",
    spoolerFix: "إصلاح الآن",
    spoolerFixed: "تم الإصلاح",
    spoolerFixFailed: "فشل الإصلاح",
    contextMenu: "مدير قائمة كليك يمين",
    toolContextMenuDesc: "تعطيل/تفعيل إضافات قائمة كليك يمين، والتبديل بين قائمة Windows 11 والكلاسيكية.",
    ctxScope: "النطاق",
    ctxName: "العنصر",
    ctxFriendly: "الاسم",
    ctxEnable: "تفعيل",
    ctxDisable: "تعطيل",
    ctxWin11Classic: "قائمة كليك يمين كلاسيكية",
    ctxWin11ClassicHint: "إظهار القائمة الكاملة (Windows 10) في Windows 11.",
    ctxScopeFile: "ملف",
    ctxScopeFolder: "مجلد",
    ctxScopeBackground: "خلفية مجلد",
    shredder: "مُدمّر الملفات المستعصية",
    toolShredderDesc: "فك قفل الملفات (Unlocker) + فرم آمن 1/3 ممرات.",
    unlocker: "فك القفل",
    unlockerScan: "فحص القفل",
    lockingProcesses: "العمليات المانعة",
    killAndDelete: "إنهاء وحذف",
    shredFile: "فرم آمن",
    shredPasses: "ممرات",
    shredderWarn: "لا يمكن استعادة الملف بعد الفرم.",
    fonts: "مدير الخطوط",
    toolFontsDesc: "قراءة/معاينة/نسخ احتياطي/حذف الخطوط، واكتشاف المكررات والمسارات التالفة.",
    fontPreview: "معاينة",
    fontBackup: "نسخ احتياطي",
    fontDelete: "حذف",
    fontBroken: "مسار تالف",
    fontDuplicate: "مكرر",
    fontSystem: "نظام",
    fontUser: "مستخدم",
    ramOptimizer: "محسن ذاكرة RAM والكاش",
    toolRamOptimizerDesc: "تفريغ Working Set والـ Standby List، وعرض الذاكرة المحررة.",
    ramTotal: "الإجمالي",
    ramUsed: "المستخدم",
    ramAvailable: "المتاح",
    ramFreed: "تم تحرير",
    ramClean: "تحسين الآن",
    ramAutoClean: "تنظيف تلقائي عند 85%",
    battery: "صحة البطارية والتنبيه",
    toolBatteryDesc: "قراءة صحة البطارية وتنبيه الشحن (95%/20%) مع صوت.",
    batteryPresent: "البطارية",
    batteryCharge: "نسبة الشحن",
    batteryHealth: "صحة البطارية",
    batteryDesign: "السعة التصميمية",
    batteryFull: "السعة الفعلية",
    batteryCycles: "دورات الشحن",
    batteryRate: "معدل الشحن/التفريغ",
    batteryTime: "الوقت المتبقي",
    batteryUpper: "حد التنبيه (علوي)",
    batteryLower: "حد التنبيه (سفلي)",
    batteryMonitor: "تفعيل المراقبة",
    batteryStatusCharging: "قيد الشحن",
    batteryStatusDischarging: "قيد التفريغ",
    batteryStatusAC: "يعمل على الطاقة المباشرة (AC)",
    batteryNoBattery: "لا توجد بطارية (جهاز مكتبي)",
    adobeCache: "منظف كاش برامج التصميم",
    toolAdobeCacheDesc: "فحص وحذف كاش Adobe ومصغرات Windows (مسارات آمنة).",
    adobeRunning: "برنامج Adobe مفتوح — أغلقه قبل التنظيف",
    adobeScan: "فحص",
    adobeClean: "تنظيف المحدد",
    runtimes: "التعريفات والبرامج الأساسية",
    hwmon: "مراقب الأجهزة (الحرارة والسرعة)",
    toolHwmonDesc: "حرارة المعالج والكرت وسرعتهما من WMI و nvidia-smi — بدون صلاحيات إدارية.",
    hwmonRefresh: "تحديث القراءات",
    hwmonGpu: "كرت الشاشة",
    hwmonSources: "المصادر",
    hwmonMetric: "القياس",
    hwmonValue: "القراءة",
    hwmonNoSensors: "لا توجد مستشعرات متاحة على هذا الجهاز.",
    githubRepo: "مستودع GitHub للتحديثات",
    githubRepoHint: "مثال: ramadanfadel/VEXORA-1.4.0-Clean — يقبل أي اسم إصدار أو ملف تثبيت.",
    githubToken: "رمز GitHub (اختياري)",
    githubTokenHint: "يستخدم فقط لتفادي حد 60 طلباً/ساعة. رمز بصلاحية public_repo. يُحفظ محلياً.",
    githubPage: "صفحتي على GitHub",
    updateToast: "إصدار جديد متاح",
    updateToastBody: "تحديث VEXORA متاح للتثبيت",
    typedConfirmHint: "اكتب {word} للتأكيد",
    typedConfirmMismatch: "الكتابة غير مطابقة — لم يتم تنفيذ أي شيء",
    ctxAdminDenied: "فشل التغيير — يتطلب صلاحيات Administrator. شغّل VEXORA كمسؤول.",
    appearance: "استوديو المظهر",
    appearanceDesc: "اختر تصميم الواجهة ولونها ووضعها قبل اعتمادها — التغيير يظهر في المعاينة مباشرة.",
    appearanceLayout: "تصميم الواجهة",
    appearanceAccent: "اللون الرئيسي",
    appearanceMode: "وضع العرض",
    appearanceGlow: "إضاءة خفيفة",
    appearanceAnimation: "انتقالات ناعمة",
    appearancePreview: "معاينة مباشرة",
    appearancePreviewHint: "جرّب بحرية — لا شيء يُحفظ قبل الضغط على تطبيق.",
    appearanceApply: "تطبيق",
    appearanceReset: "استعادة الافتراضي",
    myStyle: "مظهري",
    layoutClassic: "كلاسيكي",
    layoutDock: "شريط الأيقونات",
    layoutGrid: "شبكة البطاقات",
    layoutGlass: "زجاج نيون",
    accentCyan: "سيان",
    accentPurple: "بنفسجي",
    accentBlue: "أزرق",
    accentEmerald: "زمردي",
    accentPink: "وردي",
    accentAmber: "ذهبي",
    appearanceApplied: "تم تطبيق المظهر",
    appearanceResetDone: "تمت استعادة المظهر الافتراضي",
    toolRuntimesDesc: "فحص/نسخ احتياطي للتعريفات، والتحقق من VC++/.NET/DirectX مع تثبيت via winget.",
    driverBackup: "نسخ احتياطي للتعريفات",
    driverBackupFolder: "مجلد الحفظ",
    driverBackupRun: "نسخ الآن",
    runtimeInstalled: "مثبت",
    runtimeMissing: "ناقص",
    runtimeInstall: "تثبيت",
    runtimeVersion: "الإصدار",
    wipe: "مسح المساحة الحرة",
    start: "ابدأ",
    stop: "إيقاف",
    scan: "فحص",
    chooseFolder: "اختر مجلداً",
    minSize: "أصغر حجم",
    groups: "مجموعة",
    reclaim: "يمكن توفير",
    deleteSelected: "حذف المحدد",
    wipePasses: "عدد المرات",
    wipeRunning: "جارٍ الكتابة فوق المساحة الحرة…",
    wipeDone: "تم مسح المساحة الحرة ({size})",
    processes: "العمليات",
    endTask: "إنهاء",
    ramFreed: "تم تحرير {size} من الذاكرة",
    diskHealth: "صحة الأقراص",
    tweaksPerf: "الأداء",
    tweaksPrivacy: "الخصوصية",
    tweaksSystem: "النظام",
    adminNeeded: "يتطلب مسؤول",
    applied: "مُطبّق",
    notApplied: "غير مُطبّق",
    applyFailed: "تعذّر التطبيق — شغّل البرنامج كمسؤول",
    historyEmpty: "لا يوجد سجل بعد",
    clearHistory: "مسح السجل",
    language: "اللغة",
    theme: "المظهر",
    dark: "داكن",
    light: "فاتح",
    general: "عام",
    confirmBeforeClean: "تأكيد قبل التنظيف",
    previewFirst: "معاينة الملفات قبل الحذف",
    useRecycle: "استخدام سلة المحذوفات",
    recycleHint: "عند الإيقاف يُحذف نهائياً",
    shred: "حذف آمن (كتابة فوق البيانات)",
    shredPasses: "عدد مرات الكتابة",
    exclusions: "الاستثناءات",
    exclusionsHint: "مسارات لن يتم لمسها أبداً",
    add: "إضافة",
    remove: "إزالة",
    schedule: "الجدولة",
    scheduleEnable: "تنظيف تلقائي مجدول",
    daily: "يومياً",
    weekly: "أسبوعياً",
    monthly: "شهرياً",
    time: "الوقت",
    about: "حول البرنامج",
    version: "الإصدار",
    dataFolder: "مجلد البيانات",
    backupsFolder: "مجلد النسخ الاحتياطية",
    openFolder: "فتح",
    relaunchAdmin: "إعادة التشغيل كمسؤول",
    adminOn: "مسؤول",
    adminOff: "مستخدم عادي",
    saved: "تم الحفظ",
    error: "حدث خطأ",
    danger: "متقدم",
    caution: "بحذر",
    close: "إغلاق",
    cancel: "إلغاء",
    ok: "موافق",
    never: "لم يتم بعد",
    uptime: "زمن التشغيل",
    cpu: "المعالج",
    memory: "الذاكرة",
    startupCount: "بدء التشغيل",
    disk: "القرص",
    reveal: "إظهار",
    open: "فتح",
    noAdmin: "بعض العناصر تحتاج صلاحيات مسؤول",
    scheduleNote: "يتم إنشاء مهمة في مجدول المهام باسم VEXORAAutoClean",
    notNow: "لاحقاً",
    yesDelete: "حذف",
    shredDelete: "حذف آمن",
    scanFirst: "اضغط تحليل أولاً",
    emptyDirsHint: "مجلدات لا تحتوي على أي ملفات — يمكن حذفها لتوفير مساحة. تجنب حذف مجلدات النظام.",
    monitor: "مراقب التثبيت",
    monitorStart: "بدء المراقبة",
    monitorFinish: "إنهاء وحفظ",
    monitorUninstall: "إلغاء تثبيت (مراقب)",
    monitorHint: "1) اضغط «بدء المراقبة» 2) ثبّت البرنامج 3) اضغط «إنهاء وحفظ»",
    monitorNoLogs: "لا توجد سجلات مراقبة",
    services: "الخدمات والمهام",
    serviceStart: "تشغيل",
    serviceStop: "إيقاف",
    serviceEnable: "تفعيل",
    serviceDisable: "تعطيل",
    serviceName: "الاسم",
    serviceDisplay: "العرض",
    serviceStatus: "الحالة",
    serviceStartup: "البدء",
    taskName: "اسم المهمة",
    taskStatus: "الحالة",
    taskNextRun: "التشغيل التالي",
    specs: "مواصفات الجهاز",
    specCpu: "المعالج",
    specRam: "الذاكرة",
    specGpu: "بطاقة الشاشة",
    specStorage: "التخزين",
    specOs: "نظام التشغيل",
    specMotherboard: "اللوحة الأم",
    specBios: "BIOS",
    specHostname: "اسم الجهاز",
    specUsername: "المستخدم",
    specCores: "النوى",
    specThreads: "الخيوط",
    specSpeed: "السرعة",
    specSlots: "المنافذ",
    specVram: "الذاكرة المرئية",
    specDriver: "التعريف",
    specModel: "الموديل",
    specTotal: "الإجمالي",
    specFree: "متاح",
    specKind: "النوع",
    minimizeToTray: "إغلاق إلى شريط المهام",
    minimizeToTrayHint: "عند الضغط على × يبقى VEXORA يعمل في الخلفية",
    bgMonitorTitle: "مراقبة الموارد في الخلفية",
    bgMonitorHint: "اختياري. يراقب استخدام الذاكرة والمعالج ويُنبّهك عند تجاوز الحد. لا يُغلق أي برنامج تلقائياً إلا إذا فعّلت ذلك صراحة.",
    bgMonitorEnabled: "تفعيل المراقبة",
    bgMonitorEnabledHint: "تعمل في الخلفية حسب الفترة المحددة",
    bgMonitorInterval: "فترة الفحص",
    bgMonitorMemThreshold: "حد استخدام الذاكرة",
    bgMonitorCpuThreshold: "حد استخدام المعالج",
    bgMonitorAutoClose: "إغلاق تلقائي للبرامج المسموح بها",
    bgMonitorAutoCloseHint: "يُغلق فقط البرامج المذكورة أدناه",
    bgMonitorAutoClean: "تنظيف تلقائي لملفات VEXORA المؤقتة",
    bgMonitorAutoCleanHint: "ينظف مجلد VEXORA المؤقت فقط — لا يمس ملفات النظام",
    bgMonitorAllowed: "البرامج المسموح بإغلاقها (اسم لكل سطر)",
    bgMonitorAllowedHint: "أدخل أسماء العمليات (مثل chrome.exe). يُستخدم فقط مع الإغلاق التلقائي.",
    bgMonitorRunWin: "تشغيل مع Windows",
    bgMonitorRunWinHint: "يفتح VEXORA عند بدء تشغيل الجهاز لتبدأ المراقبة تلقائياً (اختياري)",
    bgMonitorCheck: "فحص الآن",
    minutes: "دقائق",
    custom: "مخصص",
    showBubble: "إظهار الفقاعة العائمة",
    showBubbleHint: "فقاعة صغيرة على سطح المكتب (CPU/RAM + اختصارات)",
    browsers: "المتصفحات",
    browserClean: "تنظيف المتصفحات",
    browserCache: "الذاكرة المؤقتة",
    browserCookies: "ملفات تعريف الارتباط",
    browserHistory: "السجل",
    browserSessions: "الجلسات",
    browserSiteData: "بيانات المواقع",
    browserCleanAll: "تنظيف الكل",
    junkFiles: "ملفات غير مرغوبة",
    junkScan: "فحص الملفات غير المرغوبة",
    junkDelete: "حذف المحدد",
    junkEmptyFolders: "مجلدات فارغة",
    junkTotal: "إجمالي",
    forcedUninstall: "إلغاء تثبيت إجباري",
    forcedUninstallHint: "إلغاء تثبيت إجباري: يبحث عن جميع آثار البرنامج (ريجستري + مجلدات) — استخدمه بعد إلغاء التثبيت العادي لتنظيف من الجذور، أو عندما يكون المُلغي مكسوراً/مفقوداً",
    forcedUninstallBtn: "إلغاء تثبيت إجباري",
    hunterMode: "وضع الصياد (Hunter Mode)",
    hunterModeHint: "وجّه المؤشر إلى نافذة أي برنامج ثم اضغط «التقاط» لتحديده وإلغاء تثبيته",
    captureTarget: "التقاط الهدف",
    noMatch: "لم يتم العثور على برنامج مطابق",
    deepScanLevel: "مستوى الفحص العميق",
    safe: "آمن",
    review: "مراجعة",
    sensitive: "حساس",
    progSizeMeasured: "الحجم المقاس فعلياً",
    progInstallType: "نوع التثبيت",
    progSource: "مصدر التسجيل",
    progSafety: "تقييم الأمان",
    typeWin32: "Win32",
    typeMsi: "MSI",
    typeStore: "Microsoft Store",
    typePortable: "محمول (Portable)",
    typeUnknown: "غير معروف",
    moderate: "متوسط",
    advanced: "متقدم",
    credit: "تصميم وإعداد أ. رمضان فاضل مصطفى (أبو مريم)",
    creditShort: "أ. رمضان فاضل مصطفى (أبو مريم)",
    designer: "التصميم والإعداد",
    update: "التحديثات",
    checkUpdate: "التحقق من التحديثات",
    upToDate: "لديك أحدث إصدار",
    updateAvailable: "يتوفر إصدار جديد {version}",
    updateNow: "تحديث الآن",
    updateNotes: "ملاحظات الإصدار",
    downloading: "جارٍ تنزيل التحديث والتحقق منه…",
    updateStarted: "تم التحقق من التحديث وسيبدأ التثبيت الآن",
    updateFailed: "تعذّر التحديث",
    updateNoManifest: "لم يُعثر على ملف latest.json بجوار البرنامج. ضعه بجوار الملف التنفيذي أو حدّد رابطاً أدناه.",
    currentVersion: "الإصدار الحالي",
    latestVersion: "أحدث إصدار",
    updateSource: "مصدر التحديث (latest.json)",
    updateSourceHint: "اتركه فارغاً لاستخدام latest.json الموجود بجوار البرنامج",
    shaVerified: "يتم التحقق من بصمة SHA-256 قبل التثبيت",
    safety: "الأمان والثقة",
    autoElevate: "طلب صلاحيات المسؤول تلقائياً",
    autoElevateHint: "يعيد تشغيل البرنامج كمسؤول عند البدء للوصول إلى كل الميزات",
    restore: "نقاط الاستعادة",
    restoreHint: "أنشئ نقطة استعادة قبل التنظيف العميق لحماية النظام",
    restoreBeforeClean: "إنشاء نقطة استعادة قبل التنظيف",
    restoreBeforeCleanHint: "موصى به. يتطلب صلاحيات مسؤول وتفعيل حماية النظام.",
    createRestore: "إنشاء نقطة استعادة",
    openRestore: "فتح استعادة النظام",
    restorePoints: "النقاط المتاحة",
    restoreCreated: "تم إنشاء نقطة استعادة بنجاح",
    restoreNeedsAdmin: "يلزم تشغيل البرنامج كمسؤول لإنشاء نقطة استعادة",
    restoreFailed: "تعذّر إنشاء نقطة استعادة",
    restorePointDesc: "تُنشأ نقطة استعادة تلقائياً قبل كل تغيير، يمكنك التراجع عنه في أي وقت.",
    creatingRestore: "جارٍ إنشاء نقطة الاستعادة...",
    restoreCreatedShort: "تم إنشاء نقطة استعادة قبل التنظيف",
    lastRestore: "آخر نقطة استعادة",
    backupRestore: "النسخ الاحتياطي والتصدير",
    exportReport: "تصدير التقرير",
    exportHtml: "تقرير HTML",
    exportCsv: "جدول CSV",
    exportSettings: "نسخ الإعدادات احتياطياً",
    importSettings: "استيراد الإعدادات",
    settingsExported: "تم تصدير الإعدادات: {path}",
    settingsImported: "تم استيراد الإعدادات بنجاح",
    reportSaved: "تم حفظ التقرير: {path}",
    reportsFolder: "مجلد التقارير",
    versionShort: "الإصدار",
    name: "الاسم",
    size: "الحجم",
    viewList: "قائمة",
    viewGrid: "أيقونات",
    viewDetails: "تفاصيل",
    sortBy: "ترتيب حسب",
    sortName: "الاسم",
    sortSize: "الحجم",
    sortDate: "تاريخ التثبيت",
    sortPublisher: "الناشر",
    batchUninstall: "إلغاء تثبيت المحدد",
    batchHint: "سيتم تشغيل أداة إزالة كل برنامج على حدة. تأكد من حفظ عملك.",
    system: "نظام",
    memory: "الذاكرة",
    closeGraceful: "إغلاق طبيعي",
    forceKill: "إنهاء قسري",
    closeSelected: "إغلاق المحدد",
    freeMemory: "تحرير الذاكرة",
    showSystem: "إظهار عمليات النظام",
    bgProgramsHint: "حدد البرامج (غير النظام) وأغلقها بزر واحد. الإغلاق الطبيعي يحاول حفظ العمل.",
    confirmCloseBg: "إغلاق البرامج المحددة؟ قد يفقد عمل غير محفوظ.",
    confirmForceKill: "إنهاء قسري قد يفقد البيانات. متابعة؟",
    iconsLoading: "جارٍ تحميل الأيقونات…",
    copyName: "نسخ الاسم",
    progIconHint: "الأيقونات تُحمّل تلقائياً من البرامج المثبّتة",
    toolDupesDesc: "البحث عن ملفات مكررة حسب الحجم والبصمة",
    toolBigDesc: "أكبر الملفات على القرص",
    toolEmptyDesc: "مجلدات فارغة قابلة للحذف",
    toolMapDesc: "خريطة استخدام القرص",
    toolWipeDesc: "مسح المساحة الحرة",
    toolBrowsersDesc: "تنظيف بيانات المتصفحات",
    toolJunkDesc: "ملفات مؤقتة ومخلفات",
    toolServicesDesc: "الخدمات والمهام المجدولة",
    toolMonitorDesc: "مراقبة تثبيت البرامج",
    cleanupPlanHint: "راجع القائمة واختر ما تريد إزالته. العناصر المحجوبة محمية ولا يمكن حذفها.",
    quarantineNote: "تُنقل العناصر إلى الحجر الصحي ويمكن استعادتها لاحقاً.",
    restoreQuarantine: "استعادة المحذوفات",
    emptyQuarantine: "تفريغ الحجر الصحي",
    leftoverScan: "فحص البقايا",
    leftoverScanHint: "بحث تلقائي شامل عن مفاتيح إلغاء تثبيت تالفة ومجلدات أيتام فيها أدوات إلغاء تثبيت واختصارات مكسورة — تُعرض النتائج للمراجعة قبل أي حذف",
    leftoversReview: "مراجعة الملفات المتبقية بعد إلغاء التثبيت",
    uninstalling: "جارٍ إلغاء التثبيت…",
    uninstallDone: "تم إلغاء التثبيت بنجاح",
  noLeftoversHint: "لا توجد بقايا مكتشفة. لتنظيف أعمق استخدم «إلغاء تثبيت إجباري» (يبحث عن جميع الآثار).",
    uninstall_stillRegistered: "انتهى المُلغي لكن البرنامج لا يزال مسجّلاً — راجع نافذة المُلغي وأعد المحاولة",
    uninstall_launchFailed: "تعذّر تشغيل أداة إلغاء التثبيت",
    uninstall_noCommand: "لا يوجد أمر إلغاء تثبيت مسجّل لهذا البرنامج",
    uninstall_timeout: "انتهت مهلة الانتظار — قد تكون أداة الإلغاء لا تزال تعمل",
  },
  en: {
    restoreCenter: "Restore Center",
    restoreCenterHint: "Shows reversible changes. Permanent deletion/shredding cannot be undone.",
    noReversibleChanges: "No reversible changes recorded",
    restoreBeforeTweaksHint: "Optional. May take time and requires admin rights and System Protection. If unavailable, tweak still applies with a warning.",
    details: "Details",
    excludePath: "Exclude path",
    excluded: "Excluded",
    analysisFor: "Analysis for",
    risk: "Risk",
    selectedForClean: "Selected for clean",
    noItems: "No items",
    restorePointBeforeTweaks: "Create restore point before tweaks (when available)",
    changeLog: "Change log",
    restoreOriginal: "Restore previous state",
    noChanges: "No recorded changes",
    changeApplied: "Tweak applied",
    changeRestored: "Tweak restored",
    reversible: "Reversible",
    notReversible: "Not automatically reversible",
    sensitivitySafe: "Safe",
    sensitivityReview: "Review required",
    sensitivitySensitive: "Sensitive",
    riskLevel: "Risk level",
    reason: "Reason",
    selectForClean: "Include in clean",
    analysisResults: "Analysis results",
    pathCol: "Path",
    partialClean: "Partial clean",
    failedItems: "Failed items",
    confirm: "Confirm",
    "app.title": "VEXORA",
    "app.tagline": "Clean Smarter. Run Faster.",
    "nav.dashboard": "Dashboard",
    "nav.clean": "Cleaner",
    "nav.tune": "Speed & Tuning",
    "nav.registry": "Registry",
    "nav.startup": "Startup",

  "nav.bgprograms": "Background programs",
    "nav.uninstall": "Uninstaller",
    "nav.tools": "Tools",
    "nav.history": "History",
    "nav.settings": "Settings",
    analyze: "Analyze",
    quickClean: "Quick clean",
    clean: "Clean",
    scanning: "Analyzing…",
    cleaning: "Cleaning…",
    junkFound: "Junk files found",
    freeSpace: "Free space",
    usedSpace: "Used",
    health: "System health",
    lastClean: "Last clean",
    totalCleaned: "Total cleaned",
    quickActions: "Quick actions",
    tempClean: "Temp files",
    recycleBin: "Recycle bin",
    dnsFlush: "Flush DNS",
    boost: "Free memory",
    privacy: "Privacy",
    welcome: "Welcome",
    subtitle: "Keep your PC fast, clean and private",
    selectAll: "Select all",
    selectNone: "Select none",
    clearAll: "Clear all",
    selected: "Selected",
    nothingSelected: "Nothing selected",
    items: "Items",
    path: "Path",
    unknown: "Unknown",
    nothingFound: "Nothing to clean",
    done: "Done",
    cleanResult: "Removed {n} items, freed {size}",
    skipped: "Skipped {n} items",
    preview: "Cleaning preview",
    previewHint: "These items will be deleted:",
    confirmClean: "Delete {n} items ({size})?",
    registryScan: "Scan registry",
    registryFix: "Fix selected",
    backup: "Backup",
    issues: "issues",
    registryClean: "Registry is clean — no issues found",
    fixConfirm: "Fix {n} issues after creating a backup?",
    tweakConfirm: "This will change a Windows setting. Continue? You can undo it later from the change log.",
    fixed: "Fixed {n} issues",
    backupAt: "Backup: {path}",
    startupTitle: "Startup programs",
    enabled: "Enabled",
    disabled: "Disabled",
    delete: "Delete",
    enable: "Enable",
    disable: "Disable",
    confirmDelete: "Delete permanently?",
    confirmDisable: "Disable this item?",
    safeToDisable: "Safe to disable",
    essential: "Essential",
    selectSafe: "Disable safe suggestions",
    disableSelected: "Disable selected",
    confirmDisableBatch: "Disable selected startup items?",
    programs: "Installed programs",
    uninstall: "Uninstall",
    removeEntry: "Remove entry",
    leftovers: "Scan leftovers",
    noLeftovers: "No leftovers found",
    publisher: "Publisher",
    version: "Version",
    installedOn: "Installed",
    programDetails: "Program details",
    search: "Search…",
    dupes: "Duplicate files",
    similarImages: "Similar images",
    toolSimilarDesc: "Find identical and visually similar images",
    bigFiles: "Large files",
    emptyDirs: "Empty folders",
    diskMap: "Disk analyzer",
    drives: "Disk check",
    toolDrivesDesc: "Check hard disks and USB flash drives: capacity, health, SMART data, and a read-only error scan.",
    driveHealth: "Health",
    driveFree: "Free",
    driveTotal: "Total",
    driveType: "Type",
    driveModel: "Model",
    driveScan: "Scan for errors",
    driveScanHint: "Read-only scan (chkdsk /scan) — it never repairs or changes anything. May require administrator rights.",
    smartInfo: "SMART",
    removable: "USB flash",
    fixedDisk: "Internal disk",
    cloudDisk: "Cloud disk",
    diskHealth: "Disk health",
    driveTemp: "Temp",
    driveWear: "Wear",
    drivePowerOn: "Power-on hours",
    driveStatusClean: "Healthy — no errors",
    driveStatusErrors: "Errors found — back up your data, then repair",
    driveStatusCleaned: "Some items were cleaned",
    driveStatusUnavailable: "Could not check (needs admin or drive in use)",
    driveStatusUnknown: "Unknown result",
    driveAdminHint: "Some SMART data and the error scan require administrator rights.",
    driveRaw: "Scan details",
    sysRepair: "System file repair",
    toolSysRepairDesc: "Run DISM and SFC to repair corrupted system files (may take 10–30 minutes).",
    repairStart: "Start repair",
    repairCancel: "Cancel",
    repairNeedsAdmin: "Requires administrator",
    repairClean: "Completed — system is healthy",
    repairErrors: "Completed with errors — review details",
    repairCancelled: "Cancelled",
    repairStageCheck: "Check image health",
    repairStageRestore: "Restore image health",
    repairStageSfc: "Scan system files",
    repairLog: "Log",
    spooler: "Print spooler fixer",
    toolSpoolerDesc: "Stop Spooler, clear pending jobs, restart it.",
    spoolerPending: "Pending jobs",
    spoolerFix: "Fix now",
    spoolerFixed: "Fixed",
    spoolerFixFailed: "Fix failed",
    contextMenu: "Context menu manager",
    toolContextMenuDesc: "Enable/disable context menu handlers and toggle Windows 11 classic menu.",
    ctxScope: "Scope",
    ctxName: "Item",
    ctxFriendly: "Name",
    ctxEnable: "Enable",
    ctxDisable: "Disable",
    ctxWin11Classic: "Classic context menu",
    ctxWin11ClassicHint: "Show the full (Windows 10) menu in Windows 11.",
    ctxScopeFile: "File",
    ctxScopeFolder: "Folder",
    ctxScopeBackground: "Folder background",
    shredder: "File shredder & unlocker",
    toolShredderDesc: "Unlock locked files and securely shred (1/3 passes).",
    unlocker: "Unlocker",
    unlockerScan: "Scan lock",
    lockingProcesses: "Locking processes",
    killAndDelete: "Kill & delete",
    shredFile: "Secure shred",
    shredPasses: "Passes",
    shredderWarn: "File cannot be recovered after shredding.",
    fonts: "Font manager",
    toolFontsDesc: "List/preview/backup/delete fonts; detect duplicates and broken paths.",
    fontPreview: "Preview",
    fontBackup: "Backup",
    fontDelete: "Delete",
    fontBroken: "Broken path",
    fontDuplicate: "Duplicate",
    fontSystem: "System",
    fontUser: "User",
    ramOptimizer: "RAM & cache optimizer",
    toolRamOptimizerDesc: "Trim working sets and standby list, show freed memory.",
    ramTotal: "Total",
    ramUsed: "Used",
    ramAvailable: "Available",
    ramFreed: "Freed",
    ramClean: "Optimize now",
    ramAutoClean: "Auto-clean at 85%",
    battery: "Battery health & charge alarm",
    toolBatteryDesc: "Read battery health and alert on charge (95%/20%) with sound.",
    batteryPresent: "Battery",
    batteryCharge: "Charge",
    batteryHealth: "Health",
    batteryDesign: "Design capacity",
    batteryFull: "Full capacity",
    batteryCycles: "Cycle count",
    batteryRate: "Charge/discharge rate",
    batteryTime: "Time remaining",
    batteryUpper: "Upper alert",
    batteryLower: "Lower alert",
    batteryMonitor: "Enable monitoring",
    batteryStatusCharging: "Charging",
    batteryStatusDischarging: "Discharging",
    batteryStatusAC: "On AC power",
    batteryNoBattery: "No battery (desktop PC)",
    adobeCache: "Design cache cleaner",
    toolAdobeCacheDesc: "Scan and clean Adobe caches and Windows thumbnails (safe paths only).",
    adobeRunning: "An Adobe app is open — close it before cleaning",
    adobeScan: "Scan",
    adobeClean: "Clean selected",
    runtimes: "Drivers & runtimes",
    hwmon: "Hardware monitor (temps & speeds)",
    toolHwmonDesc: "CPU/GPU temperatures, clock, fan and load via WMI and nvidia-smi — no admin rights.",
    hwmonRefresh: "Refresh readings",
    hwmonGpu: "Graphics card",
    hwmonSources: "Sources",
    hwmonMetric: "Metric",
    hwmonValue: "Reading",
    hwmonNoSensors: "No sensors are exposed on this device.",
    githubRepo: "GitHub repo for updates",
    githubRepoHint: "Example: ramadanfadel/VEXORA-1.4.0-Clean — any release tag or installer name works.",
    githubToken: "GitHub token (optional)",
    githubTokenHint: "Only used to avoid the 60 requests/hour API limit. A public_repo token is enough. Stored locally.",
    githubPage: "My GitHub page",
    updateToast: "New version available",
    updateToastBody: "A VEXORA update is ready to install",
    typedConfirmHint: "Type {word} to confirm",
    typedConfirmMismatch: "Text did not match — nothing was changed",
    ctxAdminDenied: "Change failed — Administrator rights are required. Run VEXORA as administrator.",
    appearance: "Appearance Studio",
    appearanceDesc: "Choose the layout, accent and mode — changes appear in the preview instantly.",
    appearanceLayout: "Interface layout",
    appearanceAccent: "Accent color",
    appearanceMode: "Display mode",
    appearanceGlow: "Subtle glow",
    appearanceAnimation: "Smooth animations",
    appearancePreview: "Live preview",
    appearancePreviewHint: "Experiment freely — nothing is saved until you press Apply.",
    appearanceApply: "Apply",
    appearanceReset: "Reset to default",
    myStyle: "My Style",
    layoutClassic: "Classic",
    layoutDock: "Icon Dock",
    layoutGrid: "Icon Grid",
    layoutGlass: "Glass Neon",
    accentCyan: "Cyber Cyan",
    accentPurple: "Neon Purple",
    accentBlue: "Electric Blue",
    accentEmerald: "Emerald",
    accentPink: "Hot Pink",
    accentAmber: "Amber Glow",
    appearanceApplied: "Appearance applied",
    appearanceResetDone: "Default appearance restored",
    toolRuntimesDesc: "Backup drivers, and verify VC++/.NET/DirectX with winget install.",
    driverBackup: "Backup drivers",
    driverBackupFolder: "Destination folder",
    driverBackupRun: "Backup now",
    runtimeInstalled: "Installed",
    runtimeMissing: "Missing",
    runtimeInstall: "Install",
    runtimeVersion: "Version",
    wipe: "Free-space wipe",
    start: "Start",
    stop: "Stop",
    scan: "Scan",
    chooseFolder: "Choose folder",
    minSize: "Minimum size",
    groups: "groups",
    reclaim: "Can save",
    deleteSelected: "Delete selected",
    wipePasses: "Passes",
    wipeRunning: "Overwriting free space…",
    wipeDone: "Free space wiped ({size})",
    processes: "Processes",
    endTask: "End task",
    ramFreed: "Freed {size} of memory",
    diskHealth: "Disk health",
    tweaksPerf: "Performance",
    tweaksPrivacy: "Privacy",
    tweaksSystem: "System",
    adminNeeded: "Admin required",
    applied: "Applied",
    notApplied: "Not applied",
    applyFailed: "Could not apply — run the app as administrator",
    historyEmpty: "No history yet",
    clearHistory: "Clear history",
    language: "Language",
    theme: "Theme",
    dark: "Dark",
    light: "Light",
    general: "General",
    confirmBeforeClean: "Confirm before cleaning",
    previewFirst: "Preview files before deleting",
    useRecycle: "Use recycle bin",
    recycleHint: "When off files are deleted permanently",
    shred: "Secure delete (overwrite)",
    shredPasses: "Overwrite passes",
    exclusions: "Exclusions",
    exclusionsHint: "Paths that are never touched",
    add: "Add",
    remove: "Remove",
    schedule: "Schedule",
    scheduleEnable: "Scheduled cleaning",
    daily: "Daily",
    weekly: "Weekly",
    monthly: "Monthly",
    time: "Time",
    about: "About",
    version: "Version",
    dataFolder: "Data folder",
    backupsFolder: "Backups folder",
    openFolder: "Open",
    relaunchAdmin: "Restart as administrator",
    adminOn: "Administrator",
    adminOff: "Standard user",
    saved: "Saved",
    error: "Something went wrong",
    danger: "Advanced",
    caution: "Caution",
    close: "Close",
    cancel: "Cancel",
    ok: "OK",
    never: "Never",
    uptime: "Uptime",
    cpu: "CPU",
    memory: "Memory",
    startupCount: "Startup apps",
    disk: "Disk",
    reveal: "Reveal",
    open: "Open",
    noAdmin: "Some items require administrator rights",
    scheduleNote: "A task named VEXORAAutoClean is created",
    notNow: "Not now",
    yesDelete: "Delete",
    shredDelete: "Secure delete",
    scanFirst: "Press Analyze first",
    emptyDirsHint: "Folders containing no files — can be deleted to save space. Avoid deleting system folders.",
    monitor: "Install monitor",
    monitorStart: "Start monitoring",
    monitorFinish: "Finish & save",
    monitorUninstall: "Uninstall (monitored)",
    monitorHint: "1) Click «Start monitoring» 2) Install the program 3) Click «Finish & save»",
    monitorNoLogs: "No monitored installs",
    services: "Services & tasks",
    serviceStart: "Start",
    serviceStop: "Stop",
    serviceEnable: "Enable",
    serviceDisable: "Disable",
    serviceName: "Name",
    serviceDisplay: "Display",
    serviceStatus: "Status",
    serviceStartup: "Startup",
    taskName: "Task name",
    taskStatus: "Status",
    taskNextRun: "Next run",
    specs: "System specs",
    specCpu: "CPU",
    specRam: "Memory",
    specGpu: "Graphics",
    specStorage: "Storage",
    specOs: "Operating system",
    specMotherboard: "Motherboard",
    specBios: "BIOS",
    specHostname: "Hostname",
    specUsername: "User",
    specCores: "Cores",
    specThreads: "Threads",
    specSpeed: "Speed",
    specSlots: "Slots",
    specVram: "VRAM",
    specDriver: "Driver",
    specModel: "Model",
    specTotal: "Total",
    specFree: "Free",
    specKind: "Type",
    minimizeToTray: "Minimize to tray",
    minimizeToTrayHint: "Closing with × keeps VEXORA running in the background",
    bgMonitorTitle: "Background resource monitoring",
    bgMonitorHint: "Optional. Monitors memory and CPU usage and notifies you when limits are exceeded. No program is closed automatically unless you explicitly enable it.",
    bgMonitorEnabled: "Enable monitoring",
    bgMonitorEnabledHint: "Runs in the background at the chosen interval",
    bgMonitorInterval: "Check interval",
    bgMonitorMemThreshold: "Memory usage limit",
    bgMonitorCpuThreshold: "CPU usage limit",
    bgMonitorAutoClose: "Auto-close allowed programs",
    bgMonitorAutoCloseHint: "Closes only the programs listed below",
    bgMonitorAutoClean: "Auto-clean VEXORA temp files",
    bgMonitorAutoCleanHint: "Cleans only VEXORA's temp folder — never system files",
    bgMonitorAllowed: "Allowed programs to close (one name per line)",
    bgMonitorAllowedHint: "Enter process names (e.g. chrome.exe). Used only with auto-close.",
    bgMonitorRunWin: "Run with Windows",
    bgMonitorRunWinHint: "Starts VEXORA at boot so monitoring begins automatically (optional)",
    bgMonitorCheck: "Check now",
    minutes: "minutes",
    custom: "Custom",
    showBubble: "Show floating bubble",
    showBubbleHint: "Small desktop bubble (CPU/RAM + shortcuts)",
    browsers: "Browsers",
    browserClean: "Browser cleaner",
    browserCache: "Cache",
    browserCookies: "Cookies",
    browserHistory: "History",
    browserSessions: "Sessions",
    browserSiteData: "Site data",
    browserCleanAll: "Clean all",
    junkFiles: "Junk files",
    junkScan: "Scan junk files",
    junkDelete: "Delete selected",
    junkEmptyFolders: "Empty folders",
    junkTotal: "Total",
    forcedUninstall: "Forced uninstall",
    forcedUninstallHint: "Forced uninstall: searches for all traces of a program (registry + folders) — use it after a normal uninstall to clean from the roots, or when the uninstaller is broken/missing",
    forcedUninstallBtn: "Forced uninstall",
    hunterMode: "Hunter mode",
    hunterModeHint: "Point the cursor at any program window, then click Capture to identify and uninstall it",
    captureTarget: "Capture target",
    noMatch: "No matching program found",
    deepScanLevel: "Deep scan level",
    safe: "Safe",
  review: "Review",
  sensitive: "Sensitive",
  progSizeMeasured: "Measured on disk",
  progInstallType: "Install type",
  progSource: "Registry source",
  progSafety: "Safety rating",
  typeWin32: "Win32",
  typeMsi: "MSI",
  typeStore: "Microsoft Store",
  typePortable: "Portable",
  typeUnknown: "Unknown",
    moderate: "Moderate",
    advanced: "Advanced",
    credit: "Designed & developed by Ramadan Fadel Mostafa (Abu Maryam)",
    creditShort: "Ramadan Fadel Mostafa (Abu Maryam)",
    designer: "Designed & developed by",
    update: "Updates",
    checkUpdate: "Check for updates",
    upToDate: "You have the latest version",
    updateAvailable: "Version {version} is available",
    updateNow: "Update now",
    updateNotes: "Release notes",
    downloading: "Downloading and verifying the update…",
    updateStarted: "Update verified — the installer will start now",
    updateFailed: "Update failed",
    updateNoManifest: "No latest.json found next to the app. Place it beside the executable or set a URL below.",
    currentVersion: "Current version",
    latestVersion: "Latest version",
    updateSource: "Update source (latest.json)",
    updateSourceHint: "Leave empty to use latest.json next to the executable",
    shaVerified: "SHA-256 is verified before installing",
    safety: "Safety & trust",
    autoElevate: "Request administrator rights automatically",
    autoElevateHint: "Relaunches as administrator at startup so every feature works",
    restore: "Restore points",
    restoreHint: "Create a restore point before deep cleaning to protect the system",
    restoreBeforeClean: "Create a restore point before cleaning",
    restoreBeforeCleanHint: "Recommended. Requires administrator rights and System Protection.",
    createRestore: "Create restore point",
    openRestore: "Open System Restore",
    restorePoints: "Available points",
    restoreCreated: "Restore point created",
    restoreNeedsAdmin: "Run the app as administrator to create a restore point",
    restoreFailed: "Could not create a restore point",
    restorePointDesc: "A restore point is created automatically before each change, so you can undo it at any time.",
    creatingRestore: "Creating restore point...",
    restoreCreatedShort: "Restore point created before cleaning",
    lastRestore: "Last restore point",
    backupRestore: "Backup & export",
    exportReport: "Export report",
    exportHtml: "HTML report",
    exportCsv: "CSV table",
    exportSettings: "Back up settings",
    importSettings: "Import settings",
    settingsExported: "Settings exported: {path}",
    settingsImported: "Settings imported",
    reportSaved: "Report saved: {path}",
    reportsFolder: "Reports folder",
    versionShort: "Version",
    name: "Name",
    size: "Size",
    viewList: "List",
    viewGrid: "Icons",
    viewDetails: "Details",
    sortBy: "Sort by",
    sortName: "Name",
    sortSize: "Size",
    sortDate: "Install date",
    sortPublisher: "Publisher",
    batchUninstall: "Uninstall selected",
    batchHint: "Each selected program's uninstaller will be launched one by one. Save your work first.",
    system: "System",
    memory: "Memory",
    closeGraceful: "Close gracefully",
    forceKill: "Force kill",
    closeSelected: "Close selected",
    freeMemory: "Free memory",
    showSystem: "Show system processes",
    bgProgramsHint: "Select non-system programs and close them with one button. Graceful close tries to save work.",
    confirmCloseBg: "Close selected programs? Unsaved work may be lost.",
    confirmForceKill: "Force kill may lose data. Continue?",
    iconsLoading: "Loading icons…",
    copyName: "Copy name",
    progIconHint: "Icons are loaded automatically from installed programs",
    toolDupesDesc: "Find duplicate files by size and hash",
    toolBigDesc: "Largest files on disk",
    toolEmptyDesc: "Empty folders that can be removed",
    toolMapDesc: "Disk usage map",
    toolWipeDesc: "Wipe free space",
    toolBrowsersDesc: "Clean browser data",
    toolJunkDesc: "Temp files and leftovers",
    toolServicesDesc: "Services and scheduled tasks",
    toolMonitorDesc: "Monitor program installations",
    cleanupPlanHint: "Review the list and choose what to remove. Blocked items are protected and cannot be deleted.",
    quarantineNote: "Items are moved to quarantine and can be restored later.",
    restoreQuarantine: "Restore quarantine",
    emptyQuarantine: "Empty quarantine",
    leftoverScan: "Scan leftovers",
    leftoverScanHint: "Automatic full search for broken uninstall entries, orphan folders with uninstallers and dead shortcuts left by installs/uninstalls — results are shown for review before anything is deleted",
    leftoversReview: "Review leftovers after uninstalling",
    uninstalling: "Uninstalling…",
    uninstallDone: "Uninstalled successfully",
  noLeftoversHint: "No leftovers detected. For a deeper clean use 'Forced uninstall' (searches all traces).",
    uninstall_stillRegistered: "The uninstaller finished but the program is still registered — check its window and retry",
    uninstall_launchFailed: "Could not start the uninstaller",
    uninstall_noCommand: "No uninstall command is registered for this program",
    uninstall_timeout: "Timed out waiting — the uninstaller may still be running",
  },
};

const state = {
  lang: "ar",
  theme: "dark",
  page: "dashboard",
  settings: null,
  catalog: null,
  info: null,
  selected: new Set(),
  scan: {},          // id -> {size, count, note, items}
  analyzed: false,
  tweaks: [],
  bgProcs: null,
  bgSel: null,
  bgShowSystem: false,
  bgIcons: {},
  progs: [],
  progQuery: "",
  progView: localStorage.getItem("vexora_prog_view") || "list",
  progSort: localStorage.getItem("vexora_prog_sort") || "name",
  progSel: new Set(),
  progIcons: (() => { try { const o = JSON.parse(localStorage.getItem("vexora_prog_icons") || "{}"); if (Date.now() - (o.__t || 0) > 7 * 864e5) return {}; delete o.__t; return o; } catch (e) { return {}; } })(),
  startup: [],
  startupExtras: { icons: {}, publishers: {} },
  issues: [],
  cleanQuery: "",
  busyCount: 0,
  health: null,
  totals: [0, 0],
  lastClean: 0,
  tools: { tab: "dupes", root: "C:\\", results: {}, browserCats: ["cache", "cookies", "history", "sessions"], junk: null, services: null, tasks: null, monitorName: "", monitorActive: false, monitorLogs: null, similar: null, wipe: null },
};

/* ------------------------- Appearance Studio (Phase 1) ------------------- */
const VEX_LAYOUTS = ["classic", "dock", "grid", "glass"];
const VEX_ACCENTS = {
  cyan: { hex: "#16CDE3", label: "accentCyan" },
  purple: { hex: "#A855F7", label: "accentPurple" },
  blue: { hex: "#3478FF", label: "accentBlue" },
  emerald: { hex: "#10B981", label: "accentEmerald" },
  pink: { hex: "#EC4899", label: "accentPink" },
  amber: { hex: "#F59E0B", label: "accentAmber" },
};
const VEX_APPEARANCE_DEFAULT = { layout: "grid", accent: "cyan", mode: "dark", glow: true, animation: true };
let vexAppearanceDraft = null;

function vexDraftFromSettings() {
  const s = state.settings || {};
  return {
    layout: VEX_LAYOUTS.includes(s.appearanceLayout) ? s.appearanceLayout : "classic",
    accent: VEX_ACCENTS[s.appearanceAccent] ? s.appearanceAccent : "cyan",
    mode: s.appearanceMode === "light" ? "light" : "dark",
    glow: s.appearanceGlow !== false,
    animation: s.appearanceAnimation !== false,
  };
}

function vexShade(hex, amount) {
  const n = parseInt(hex.slice(1), 16);
  const mix = amount < 0 ? 0 : 255;
  const p = Math.abs(amount);
  const ch = (v) => Math.round(v + (mix - v) * p).toString(16).padStart(2, "0");
  return "#" + ch((n >> 16) & 255) + ch((n >> 8) & 255) + ch(n & 255);
}

function vexPreviewVars(draft) {
  const hex = (VEX_ACCENTS[draft.accent] || VEX_ACCENTS.cyan).hex;
  const dark = draft.mode === "dark";
  const textForAccent = dark ? hex : vexShade(hex, -0.45);
  return [
    `--vex-accent:${hex}`,
    `--vex-accent-soft:${hex}22`,
    `--vex-accent-text:${textForAccent}`,
    `--vex-bg:${dark ? "#070c16" : "#f2f6fb"}`,
    `--vex-card:${dark ? "#0d1829" : "#ffffff"}`,
    `--vex-text:${dark ? "#e8f1fb" : "#0f172a"}`,
    `--vex-muted:${dark ? "#8fa6c0" : "#64748b"}`,
    `--vex-line:${dark ? "#1e3452" : "#d8e2ee"}`,
    `--vex-shadow:${draft.glow ? `0 0 18px ${hex}40` : "none"}`,
    `--vex-motion:${draft.animation ? ".18s" : "0s"}`,
  ].join(";");
}

function vexPreviewMarkup(draft) {
  const items = [
    ["apps", t("dashboard")],
    ["shield", t("nav.clean")],
    ["zap", t("nav.tune")],
    ["file", t("nav.tools")],
  ];
  if (draft.layout === "grid") {
    return `<div class="vex-pv-grid">${items
      .map(
        ([ic, label], i) =>
          `<div class="vex-pv-card ${i === 0 ? "active" : ""}">` +
          `<span class="vex-pv-ico">${svg(ic, 20)}</span>` +
          `<b>${esc(label)}</b><small>${esc(t("appearancePreviewHint").slice(0, 34))}</small></div>`
      )
      .join("")}</div>`;
  }
  if (draft.layout === "dock") {
    return `<div class="vex-pv-dockrow"><div class="vex-pv-rail">${items
      .map(([ic, label], i) => `<div class="vex-pv-railitem ${i === 0 ? "active" : ""}">${svg(ic, 16)}<span>${esc(label)}</span></div>`)
      .join("")}</div><div class="vex-pv-content"><div class="vex-pv-hero"></div><div class="vex-pv-line"></div><div class="vex-pv-line short"></div></div></div>`;
  }
  if (draft.layout === "glass") {
    return `<div class="vex-pv-glass"><div class="vex-pv-glow"></div>${items
      .map(
        ([ic, label], i) =>
          `<div class="vex-pv-gcard ${i === 0 ? "active" : ""}">${svg(ic, 18)}<b>${esc(label)}</b></div>`
      )
      .join("")}</div>`;
  }
  return `<div class="vex-pv-classic"><div class="vex-pv-crail">${items
    .map(([ic, label], i) => `<div class="vex-pv-cri ${i === 0 ? "active" : ""}">${svg(ic, 15)}<span>${esc(label)}</span></div>`)
    .join("")}</div><div class="vex-pv-content"><div class="vex-pv-hero"></div><div class="vex-pv-line"></div></div></div>`;
}

function vexRenderAppearancePreview(rebuild) {
  const d = vexAppearanceDraft;
  const box = $("#vexPreview");
  if (!box || !d) return;
  const changedLayout = box.dataset.layout !== d.layout;
  box.setAttribute("style", vexPreviewVars(d));
  // Only rebuild the preview DOM when the layout actually changes. Colour,
  // mode and toggle changes just update CSS variables + active classes, so
  // the studio stays responsive even on slower WebView2 installations.
  if (rebuild !== false || changedLayout || !box.firstChild) {
    box.dataset.layout = d.layout;
    box.innerHTML =
      `<div class="vex-pv-head"><span class="vex-pv-logo">V</span>` +
      `<span>${esc(t("appearancePreview"))} · ${esc(t("layout" + d.layout.charAt(0).toUpperCase() + d.layout.slice(1)))}</span></div>` +
      vexPreviewMarkup(d);
  }
  // Scope to the studio card: `body[data-vex-layout]` also matches these
  // selectors and its click handler would otherwise reset the draft.
  $$("#vexAppearance [data-vex-layout]").forEach((b) => b.classList.toggle("active", b.dataset.vexLayout === d.layout));
  $$("#vexAppearance [data-vex-accent]").forEach((b) => b.classList.toggle("active", b.dataset.vexAccent === d.accent));
  $$("#vexAppearance [data-vex-mode]").forEach((b) => b.classList.toggle("active", b.dataset.vexMode === d.mode));
  $$("#vexAppearance [data-vex-toggle]").forEach((b) => b.classList.toggle("on", !!d[b.dataset.vexToggle]));
}

function vexUpdateMyStyleLabel() {
  const el = $("#vexMyStyleLabel");
  if (!el || !state.settings) return;
  const d = vexDraftFromSettings();
  el.textContent =
    `${t("layout" + d.layout.charAt(0).toUpperCase() + d.layout.slice(1))} · ` +
    `${t("accent" + d.accent.charAt(0).toUpperCase() + d.accent.slice(1))}`;
  const dot = $("#vexMyStyleDot");
  if (dot) dot.style.background = (VEX_ACCENTS[d.accent] || VEX_ACCENTS.cyan).hex;
}

/// Apply the committed appearance to the real shell. The floating bubble is a
/// separate document and is intentionally unaffected.
function vexApplyAppearance() {
  if (!state.settings) return;
  const d = vexDraftFromSettings();
  const hex = (VEX_ACCENTS[d.accent] || VEX_ACCENTS.cyan).hex;
  const root = document.documentElement;
  root.dataset.vexMode = d.mode;
  document.body.dataset.vexLayout = d.layout;
  document.body.dataset.vexGlow = d.glow ? "on" : "off";
  document.body.dataset.vexAnimation = d.animation ? "on" : "off";
  const accent = d.mode === "light" ? vexShade(hex, -0.4) : hex;
  root.style.setProperty("--accent", accent);
  root.style.setProperty("--accent-soft", hex + (d.mode === "light" ? "1f" : "24"));
  root.style.setProperty("--accent-2", vexShade(accent, -0.15));
  root.style.setProperty("--accent-text", d.mode === "light" ? vexShade(hex, -0.45) : hex);
}

const t = (key, params) => {
  let s = (DICT[state.lang] && DICT[state.lang][key]) || DICT.en[key] || key;
  if (params) {
    for (const [k, v] of Object.entries(params)) s = s.replaceAll(`{${k}}`, v);
  }
  return s;
};

/* -------------------------------------------------------------------- icons */
const ICONS = {
  home: '<path d="M3 10.6 12 3l9 7.6"/><path d="M5 9.8V20h14V9.8"/><path d="M9.5 20v-5.5h5V20"/>',
  broom: '<path d="M14 3l7 7"/><path d="M11.5 5.5 18.5 12.5"/><path d="M8 8l8 8-4.5 5H6l-1.5-5.5L8 8z"/>',
  zap: '<path d="M13 2 4 14h6l-1 8 9-12h-6l1-8z"/>',
  database: '<ellipse cx="12" cy="5.5" rx="8" ry="3"/><path d="M4 5.5v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6"/><path d="M4 11.5v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6"/>',
  rocket: '<path d="M5 15c-1.5 1.5-2 5-2 5s3.5-.5 5-2"/><path d="M9 13c-1-3 1-7 5-9 2-1 5-2 6-2 0 1-1 4-2 6-2 4-6 6-9 5z"/><circle cx="14.5" cy="8.5" r="1.6"/>',
  trash: '<path d="M4 7h16"/><path d="M9 7V4h6v3"/><path d="M6 7l1 13h10l1-13"/><path d="M10 11v5M14 11v5"/>',
  tools: '<path d="M14.5 4.5a4 4 0 1 0 5 5L21 21H3l9.5-11.5z"/>',
  clock: '<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3 2"/>',
  settings: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-2.9 1.2 2 2 0 1 1-4 0 1.7 1.7 0 0 0-2.9-1.2l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1A1.7 1.7 0 0 0 3 15a2 2 0 1 1 0-4 1.7 1.7 0 0 0 1.5-2.9l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1A1.7 1.7 0 0 0 10 4a2 2 0 1 1 4 0 1.7 1.7 0 0 0 2.9 1.4l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1A1.7 1.7 0 0 0 21 11a2 2 0 1 1 0 4z"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m16.5 16.5 4.5 4.5"/>',
  play: '<path d="M8 4.5 19 12 8 19.5z"/>',
  check: '<path d="m4 12.5 5 5L20 6.5"/>',
  x: '<path d="M6 6l12 12M18 6 6 18"/>',
  alert: '<path d="M12 3 2 20h20L12 3z"/><path d="M12 9v5"/><circle cx="12" cy="17.2" r=".9"/>',
  disk: '<rect x="3" y="5" width="18" height="14" rx="2.5"/><path d="M3 13h18"/><circle cx="7.5" cy="16" r="1"/>',
  folder: '<path d="M3 7.5a2 2 0 0 1 2-2h4l2 2.5h8a2 2 0 0 1 2 2V18a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  file: '<path d="M7 3h7l5 5v13H7z"/><path d="M14 3v5h5"/>',
  cpu: '<rect x="7" y="7" width="10" height="10" rx="2"/><path d="M10 3v4M14 3v4M10 17v4M14 17v4M3 10h4M3 14h4M17 10h4M17 14h4"/>',
  chip: '<rect x="6" y="6" width="12" height="12" rx="3"/><path d="M9 9h6v6H9z"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.5 1.5M17.5 17.5 19 19M19 5l-1.5 1.5M6.5 17.5 5 19"/>',
  moon: '<path d="M20 14.5A8.5 8.5 0 1 1 10 4a7 7 0 0 0 10 10.5z"/>',
  globe: '<circle cx="12" cy="12" r="8.5"/><path d="M3.5 12h17"/><path d="M12 3.5c2.5 2.4 3.8 5.4 3.8 8.5s-1.3 6.1-3.8 8.5c-2.5-2.4-3.8-5.4-3.8-8.5S9.5 5.9 12 3.5z"/>',
  refresh: '<path d="M20 12a8 8 0 1 1-2.4-5.7"/><path d="M20 4v4.5h-4.5"/>',
  shield: '<path d="M12 3l7 2.5V12c0 4.5-3 7.6-7 9-4-1.4-7-4.5-7-9V5.5z"/><path d="m9 12 2 2 4-4"/>',
  window: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18"/><circle cx="6.5" cy="6.5" r=".8"/>',
  browser: '<circle cx="12" cy="12" r="8.5"/><path d="M3.5 9h17M3.5 15h17"/><path d="M12 3.5c2 2.6 3 5.4 3 8.5s-1 5.9-3 8.5c-2-2.6-3-5.4-3-8.5s1-5.9 3-8.5z"/>',
  apps: '<rect x="3" y="3" width="7" height="7" rx="1.6"/><rect x="14" y="3" width="7" height="7" rx="1.6"/><rect x="3" y="14" width="7" height="7" rx="1.6"/><rect x="14" y="14" width="7" height="7" rx="1.6"/>',
  listview: '<path d="M4 6h16M4 12h16M4 18h16"/>',
  gridview: '<rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/>',
  detailview: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 8h10M7 12h10M7 16h6"/>',

  wrench: '<path d="M15.5 3.5a5 5 0 0 0-6 7.7L3.5 17.3a2 2 0 0 0 2.8 2.8l6-6a5 5 0 0 0 7.7-6l-3.3 3.3-2.8-2.8 3.3-3.3z"/>',
  chevron: '<path d="m9 6 6 6-6 6"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  info: '<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5.5"/><circle cx="12" cy="7.8" r=".9"/>',
  gauge: '<path d="M4 15a8 8 0 1 1 16 0"/><path d="m12 15 4-4"/><circle cx="12" cy="15" r="1.3"/>',
  memory: '<rect x="3" y="7" width="18" height="10" rx="2"/><path d="M7 7v10M12 7v10M17 7v10"/>',
};

const svg = (name, size = 18) =>
  `<svg viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">${ICONS[name] || ""}</svg>`;

/* ------------------------------------------------------------------ helpers */
const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));
const esc = (s) =>
  String(s == null ? "" : s).replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c])
  );

function fmtBytes(n) {
  n = Number(n) || 0;
  const units = ["بايت", "ك.ب", "م.ب", "ج.ب", "ت.ب"];
  const unitsEn = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  while (Math.abs(n) >= 1024 && i < 4) {
    n /= 1024;
    i += 1;
  }
  const label = (state.lang === "ar" ? units : unitsEn)[i];
  return i === 0 ? `${Math.round(n)} ${label}` : `${n.toFixed(1)} ${label}`;
}

function fmtDate(ts) {
  if (!ts) return t("never");
  return new Date(ts * 1000).toLocaleString(state.lang === "ar" ? "ar-EG" : "en-GB");
}

function toast(msg, type = "ok") {
  const box = $("#toasts");
  const el = document.createElement("div");
  el.className = `toast ${type === "error" ? "error" : type === "warn" ? "warn" : ""}`;
  el.textContent = msg;
  box.appendChild(el);
  setTimeout(() => el.remove(), 4200);
}

async function openAppearanceStudio() {
  await go("settings");
  const el = $("#vexAppearance");
  if (el) el.scrollIntoView({ behavior: "smooth", block: "center" });
}

function toastUpdate(info) {
  const box = $("#toasts");
  if (!box || !info || !info.available) return;
  const el = document.createElement("div");
  el.className = "toast warn";
  el.style.display = "flex";
  el.style.alignItems = "center";
  el.style.gap = "8px";
  el.style.maxWidth = "520px";
  const txt = document.createElement("span");
  txt.textContent = `${t("updateToast")} ${info.latest || ""} — ${t("updateToastBody")}`;
  const btn = document.createElement("button");
  btn.className = "btn small primary";
  btn.textContent = t("updateNow");
  btn.onclick = async () => {
    btn.disabled = true;
    try {
      await call("install_update", { url: info.url, sha256: info.sha256 }, t("downloading"));
      toast(t("updateStarted"));
      setTimeout(() => invoke("quit_app").catch(() => {}), 1600);
    } catch (e) {
      toast(String(e), "error");
      btn.disabled = false;
    }
  };
  el.appendChild(txt);
  el.appendChild(btn);
  box.appendChild(el);
  setTimeout(() => el.remove(), 15000);
}

function busy(on, text) {
  state.busyCount = Math.max(0, state.busyCount + (on ? 1 : -1));
  const el = $("#busy");
  if (state.busyCount > 0 && text) $("#busyText").textContent = text;
  el.classList.toggle("hidden", state.busyCount === 0);
}

async function call(cmd, args = {}, busyText) {
  if (busyText) busy(true, busyText);
  try {
    return await invoke(cmd, args);
  } catch (e) {
    toast(String(e), "error");
    throw e;
  } finally {
    if (busyText) busy(false);
  }
}

/** Normalise an IPC/JS error into a readable string. */
function friendlyError(e) {
  if (e == null) return "";
  if (typeof e === "string") return e;
  if (e.message) return String(e.message);
  try { return JSON.stringify(e); } catch (_) { return String(e); }
}

/* -------------------------------------------------------------------- modal */
function modal({ title, body, buttons = [], width, onClose }) {
  const root = $("#modalRoot");
  $("#modalTitle").textContent = title;
  $("#modalBody").innerHTML = body;
  const foot = $("#modalFoot");
  foot.innerHTML = "";
  if (width) $(".modal").style.width = width;
  let closed = false;
  const close = (reason = "close") => {
    if (closed) return;
    closed = true;
    root.classList.add("hidden");
    $(".modal").style.width = "";
    document.removeEventListener("keydown", onKey);
    if (onClose) onClose(reason);
  };
  const onKey = (e) => { if (e.key === "Escape") close("escape"); };
  for (const b of buttons) {
    const btn = document.createElement("button");
    btn.className = `btn ${b.cls || ""}`;
    btn.innerHTML = b.html || esc(b.label || "");
    btn.onclick = async () => {
      if (b.close !== false) close(b.reason || "button");
      if (b.onClick) await b.onClick();
    };
    foot.appendChild(btn);
  }
  if (!buttons.length) {
    const btn = document.createElement("button");
    btn.className = "btn";
    btn.textContent = t("close");
    btn.onclick = () => close("close");
    foot.appendChild(btn);
  }
  root.classList.remove("hidden");
  $("#modalClose").onclick = () => close("x");
  root.querySelector(".modal-backdrop").onclick = () => close("backdrop");
  document.addEventListener("keydown", onKey);
  return { close, body: $("#modalBody"), foot };
}

function confirmBox(message, okLabel) {
  return new Promise((resolve) => {
    let settled = false;
    const done = (v) => { if (settled) return; settled = true; resolve(v); };
    modal({
      title: t("confirm") || (state.lang === "ar" ? "تأكيد" : "Confirm"),
      body: `<p style="margin:4px 0;line-height:1.7">${esc(message)}</p>`,
      // modal() closes (and calls onClose with the button reason) BEFORE it
      // runs onClick, so onClose is the only reliable place to read the answer.
      onClose: (reason) => done(reason === "ok"),
      buttons: [
        { label: t("cancel"), cls: "ghost", reason: "cancel" },
        { label: okLabel || t("ok"), cls: "primary", reason: "ok" },
      ],
    });
  });
}

/// Irreversible actions require typing a word before the OK button works.
function typedConfirmBox(message, okLabel, expected) {
  return new Promise((resolve) => {
    let settled = false;
    let approved = false;
    const done = (v) => { if (settled) return; settled = true; resolve(v); };
    const tryConfirm = () => {
      const input = $("#typedConfirm");
      if (!input || (input.value || "").trim() !== expected) {
        toast(t("typedConfirmMismatch"), "warn");
        return;
      }
      approved = true;
      close("ok");
      done(true);
    };
    const { close } = modal({
      title: t("confirm") || (state.lang === "ar" ? "تأكيد" : "Confirm"),
      body: `<p style="margin:4px 0;line-height:1.7">${esc(message)}</p>
        <div class="field" style="margin-top:10px">
          <label>${esc(t("typedConfirmHint", { word: expected }))}</label>
          <input type="text" id="typedConfirm" autocomplete="off" spellcheck="false" style="direction:ltr" />
        </div>`,
      onClose: () => done(approved),
      buttons: [
        { label: t("cancel"), cls: "ghost", reason: "cancel" },
        { label: okLabel || t("ok"), cls: "primary", reason: "ok", close: false, onClick: tryConfirm },
      ],
    });
    const input = $("#typedConfirm");
    if (input) {
      input.focus();
      input.onkeydown = (e) => { if (e.key === "Enter") { e.preventDefault(); tryConfirm(); } };
    }
  });
}

function deleteResultToast(out, fallbackTitle) {
  const removed = (out && out.removed) || 0;
  const freed = (out && out.freed) || 0;
  const failed = (out && out.failed) || 0;
  const msg = t("cleanResult", { n: removed, size: fmtBytes(freed) });
  if (failed > 0) toast(`${msg} • ${t("failedItems")}: ${failed}`, "warn");
  else toast(msg, failed ? "warn" : "ok");
  return out;
}

function cleanErrorsText(out) {
  const errs = [];
  for (const r of (out && out.rows) || []) {
    for (const e of (r.errors || [])) { if (errs.length < 8) errs.push(e); }
  }
  if ((out && out.failed) > 0) errs.push(`${t("failedItems")}: ${out.failed}`);
  return errs;
}

function notifyRestore(out) {
  if (!out) return;
  if (out.restore === "ok") toast(t("restoreCreatedShort"));
  else if (out.restore === "admin") toast(t("restoreNeedsAdmin"), "warn");
}

/* --------------------------------------------------------------------- ring */
function ring(pct, value, label) {
  const r = 82;
  const c = 2 * Math.PI * r;
  const off = c * (1 - Math.max(0.02, Math.min(1, pct / 100)));
  return `
  <div class="ring">
    <svg width="190" height="190" viewBox="0 0 190 190">
      <circle cx="95" cy="95" r="${r}" fill="none" stroke="var(--card-3)" stroke-width="13"/>
      <circle cx="95" cy="95" r="${r}" fill="none" stroke="var(--accent)" stroke-width="13"
        stroke-linecap="round" stroke-dasharray="${c.toFixed(1)}" stroke-dashoffset="${off.toFixed(1)}"/>
    </svg>
    <div class="ring-center">
      <div class="ring-value">${value}</div>
      <div class="ring-label">${esc(label)}</div>
    </div>
  </div>`;
}

/* ------------------------------------------------------------------ sidebar */
const NAV = [
  ["dashboard", "home"],
  ["clean", "broom"],
  ["tune", "zap"],
  ["registry", "database"],
  ["startup", "rocket"],
  ["bgprograms", "apps"],
  ["uninstall", "trash"],
  ["tools", "tools"],
  ["history", "clock"],
  ["settings", "settings"],
];

const VEX_NAV_HINT = {
  dashboard: ["نظرة عامة سريعة", "Quick overview"],
  clean: ["تنظيف الملفات", "Clean files"],
  tune: ["تحسين الأداء", "Boost performance"],
  registry: ["إصلاح السجل", "Registry health"],
  startup: ["إدارة بدء التشغيل", "Startup items"],
  bgprograms: ["برامج الخلفية", "Background apps"],
  uninstall: ["إزالة البرامج", "Remove programs"],
  tools: ["أدوات متقدمة", "Advanced tools"],
  history: ["سجل العمليات", "Activity log"],
  settings: ["إعدادات VEXORA", "VEXORA settings"],
};

function renderNav() {
  const nav = $("#nav");
  nav.innerHTML = NAV.map(
    ([id, ic]) => `
    <button class="nav-item ${state.page === id ? "active" : ""}" data-nav="${id}"
      data-hint="${esc((VEX_NAV_HINT[id] || ["", ""])[state.lang === "ar" ? 0 : 1])}">
      ${svg(ic)}<span>${esc(t(`nav.${id}`))}</span>
    </button>`
  ).join("");
  $$("[data-nav]", nav).forEach((b) =>
    b.addEventListener("click", () => go(b.dataset.nav))
  );
  const badge = $("#adminBadge");
  badge.classList.toggle("hidden", !(state.info && state.info.admin));
  badge.textContent = t("adminOn");
  $("#themeBtn").innerHTML = svg(state.theme === "dark" ? "sun" : "moon");
  $("#themeBtn").title = t("theme");
  $("#langBtn").innerHTML = svg("globe");
  $("#langBtn").title = t("language");
}

async function go(page) {
  state.page = page;
  invoke("save_last_page", { page }).catch(() => {});
  renderNav();
  const view = $("#view");
  view.innerHTML = `<div class="empty">${svg("refresh", 42)}<div>${esc(t("scanning"))}</div></div>`;
  try {
    if (page === "dashboard") await renderDashboard();
    else if (page === "clean") await renderClean();
    else if (page === "tune") await renderTune();
    else if (page === "registry") await renderRegistry();
    else if (page === "startup") await renderStartup();
    else if (page === "bgprograms") await renderBgPrograms();
    else if (page === "uninstall") await renderUninstall();
    else if (page === "tools") await renderTools();
    else if (page === "history") await renderHistory();
    else if (page === "settings") await renderSettings();
  } catch (e) {
    view.innerHTML = `<div class="empty">${svg("alert", 42)}<div>${esc(String(e))}</div></div>`;
  }
}

function pageHead(title, subtitle, actions = "") {
  return `<div class="page-head">
    <div><h1>${esc(title)}</h1><p>${esc(subtitle)}</p></div>
    <div class="head-actions">${actions}</div>
  </div>`;
}

/* --------------------------------------------------------------- DASHBOARD */
async function renderDashboard() {
  const [health, totals, hist] = await Promise.all([
    call("get_health"),
    call("history_totals"),
    call("history_list", { limit: 1 }),
  ]);
  state.health = health;
  state.totals = totals;
  state.lastClean = hist.length ? hist[0].ts : 0;

  const junk = state.scan && Object.values(state.scan).length
    ? Object.values(state.scan).reduce((a, s) => a + (s.size || 0), 0)
    : 0;
  const analyzed = Object.values(state.scan).some((s) => s.count > 0);

  const memPct = health.memTotal ? (health.memUsed / health.memTotal) * 100 : 0;
  const sysDisk = health.disks[0] || { total: 0, used: 0, free: 0 };
  const diskPct = sysDisk.total ? (sysDisk.used / sysDisk.total) * 100 : 0;

  const diskBars = health.disks
    .map((d) => {
      const pct = d.total ? (d.used / d.total) * 100 : 0;
      return `<div class="bar-row">
        <span class="nm">${esc(d.path)}</span>
        <span class="bar-track"><i style="width:${pct.toFixed(1)}%"></i></span>
        <span class="vl">${fmtBytes(d.free)} ${esc(state.lang === "ar" ? "متاح" : "free")}</span>
      </div>`;
    })
    .join("");

  const quick = [
    ["temp", "broom", t("tempClean"), "system.user_temp"],
    ["recycle", "trash", t("recycleBin"), "system.recycle_bin"],
    ["dns", "globe", t("dnsFlush"), null],
    ["boost", "zap", t("boost"), null],
    ["privacy", "shield", t("privacy"), "privacy"],
    ["startup", "rocket", t("nav.startup"), null],
  ]
    .map(
      ([id, ic, label, sub]) => `
      <div class="card action-card" data-quick="${id}">
        <div class="action-ico">${svg(ic, 20)}</div>
        <div><div class="t">${esc(label)}</div><div class="s">${esc(
        sub === "privacy" ? (state.lang === "ar" ? "بصمات التصفح" : "browsing traces") : ""
      )}</div></div>
      </div>`
    )
    .join("");

  const healthPct = Math.round(100 - memPct * 0.45 - Math.max(0, diskPct - 70) * 0.9);
  const healthCls = healthPct > 80 ? "ok" : healthPct > 55 ? "warn" : "bad";

  $("#view").innerHTML =
    pageHead(
      `${t("welcome")}, ${health.user || health.host}`,
      `${esc(health.os || "Windows")} ${esc(health.osBuild || "")} · ${t("subtitle")}`,
      `<button class="btn primary" id="dashAnalyze">${svg("search")}${esc(t("analyze"))}</button>
       <button class="btn" id="dashClean">${svg("broom")}${esc(t("quickClean"))}</button>`
    ) +
    `<div class="hero">
      <div class="card hero-main">
        ${ring(
          analyzed ? Math.min(100, (junk / (2 * 1024 ** 3)) * 100) : 0,
          fmtBytes(junk),
          analyzed ? t("junkFound") : t("analyze")
        )}
        <div class="muted tiny">${esc(analyzed ? `${Object.values(state.scan).reduce((a, s) => a + (s.count || 0), 0)} ${state.lang === "ar" ? "ملف" : "files"}` : t("subtitle"))}</div>
      </div>

      <div class="grid" style="gap:14px">
        <div class="card flat">
          <div class="card-title">${svg("gauge")}${esc(t("health"))} <span class="pill ${healthCls}" style="margin-inline-start:auto">${healthPct}%</span></div>
          <div class="stat-row"><span class="label">${svg("memory", 15)}${esc(t("memory"))}</span><span class="value">${fmtBytes(health.memUsed)} / ${fmtBytes(health.memTotal)}</span></div>
          <div class="progress ${memPct > 85 ? "danger" : memPct > 65 ? "warn" : ""}"><i style="width:${memPct.toFixed(1)}%"></i></div>
          <div class="stat-row" style="margin-top:10px"><span class="label">${svg("cpu", 15)}${esc(t("cpu"))}</span><span class="value">${(health.cpu || 0).toFixed(0)}%</span></div>
          <div class="progress ${health.cpu > 80 ? "warn" : ""}"><i style="width:${Math.min(100, health.cpu || 0).toFixed(1)}%"></i></div>
        </div>
        <div class="card flat">
          <div class="card-title">${svg("disk")}${esc(t("disk"))}</div>
          <div class="bars">${diskBars}</div>
        </div>
      </div>
    </div>

    <div class="grid cols-4" style="margin-top:16px">
      <div class="card flat"><div class="tiny muted">${esc(t("freeSpace"))}</div><div style="font-size:20px;font-weight:700;margin-top:4px">${fmtBytes(sysDisk.free)}</div></div>
      <div class="card flat"><div class="tiny muted">${esc(t("startupCount"))}</div><div style="font-size:20px;font-weight:700;margin-top:4px">${health.startupCount}</div></div>
      <div class="card flat"><div class="tiny muted">${esc(t("totalCleaned"))}</div><div style="font-size:20px;font-weight:700;margin-top:4px">${fmtBytes(totals[0])}</div></div>
      <div class="card flat"><div class="tiny muted">${esc(t("lastClean"))}</div><div style="font-size:13px;font-weight:600;margin-top:8px">${fmtDate(state.lastClean)}</div></div>
    </div>

    <h2 style="font-size:15px;margin:22px 0 10px">${esc(t("quickActions"))}</h2>
    <div class="grid cols-3">${quick}</div>`;

  $("#dashAnalyze").onclick = async () => {
    const ids = defaultCleanerIds();
    const res = await call("scan_cleaners", { ids }, t("scanning"));
    state.scan = {};
    for (const r of res) state.scan[r.id] = r;
    state.analyzed = true;
    toast(t("done"));
    await renderDashboard();
  };

  $("#dashClean").onclick = async () => {
    const ids = defaultCleanerIds();
    if (!ids.length) return toast(t("nothingSelected"), "warn");
    const res = await call("scan_cleaners", { ids }, t("scanning"));
    const total = res.reduce((a, r) => a + r.size, 0);
    const count = res.reduce((a, r) => a + r.count, 0);
    if (!count) return toast(t("nothingFound"));
    if (!(await confirmBox(t("confirmClean", { n: count, size: fmtBytes(total) }), t("clean")))) return;
    const out = await call("clean_cleaners", { ids }, t("cleaning"));
    state.scan = {};
    state.analyzed = false;
    toast(t("cleanResult", { n: out.removed, size: fmtBytes(out.freed) }));
    notifyRestore(out);
    await renderDashboard();
  };

  $$("[data-quick]").forEach((el) =>
    el.addEventListener("click", async () => {
      const kind = el.dataset.quick;
      if (kind === "dns") {
        await call("flush_dns");
        return toast(t("done"));
      }
      if (kind === "boost") {
        const [freed] = await call("free_memory", {}, t("scanning"));
        return toast(t("ramFreed", { size: fmtBytes(freed) }));
      }
      if (kind === "startup") return go("startup");
      if (kind === "privacy") {
        const ids = (state.catalog ? state.catalog.cleaners : [])
          .filter((c) => c.id.startsWith("privacy.") || c.category === "browsers")
          .map((c) => c.id);
        const res = await call("scan_cleaners", { ids }, t("scanning"));
        const total = res.reduce((a, r) => a + r.size, 0);
        const count = res.reduce((a, r) => a + r.count, 0);
        if (!count) return toast(t("nothingFound"));
        if (!(await confirmBox(t("confirmClean", { n: count, size: fmtBytes(total) }), t("clean")))) return;
        const out = await call("clean_cleaners", { ids }, t("cleaning"));
        notifyRestore(out);
        return toast(t("cleanResult", { n: out.removed, size: fmtBytes(out.freed) }));
      }
      const ids = [kind === "temp" ? "system.user_temp" : "system.recycle_bin"];
      if (!(await confirmBox(t("confirmClean", { n: 1, size: "—" }), t("clean")))) return;
      const out = await call("clean_cleaners", { ids }, t("cleaning"));
      toast(t("cleanResult", { n: out.removed, size: fmtBytes(out.freed) }));
      notifyRestore(out);
    })
  );
}

function cleanerSensitivity(c) {
  if (c && c.sensitivity) return c.sensitivity;
  if (c && c.danger === "advanced") return "sensitive";
  if (c && c.danger === "caution") return "review";
  return "safe";
}
function canSelectCleaner(c) {
  const disabled = new Set((state.settings && state.settings.disabledCleaners) || []);
  if (disabled.has(c.id)) return false;
  if (c.admin && !(state.info && state.info.admin)) return false;
  return true;
}
function defaultCleanerIds() {
  if (!state.catalog) return [];
  return state.catalog.cleaners
    .filter(canSelectCleaner)
    .filter((c) => c.defaultOn && cleanerSensitivity(c) === "safe")
    .map((c) => c.id);
}

/* ------------------------------------------------------------------- CLEANER */
async function renderClean() {
  const catIcons = { system: "cpu", windows: "window", browsers: "browser", apps: "apps", privacy: "shield", advanced: "wrench" };
  const cats = state.catalog.categories;
  const cleaners = state.catalog.cleaners;

  const totalSelected = [...state.selected].reduce(
    (a, id) => a + ((state.scan[id] && state.scan[id].size) || 0),
    0
  );

  const body = cats
    .map((cat) => {
      const list = cleaners.filter(
        (c) =>
          c.category === cat.id &&
          (!state.cleanQuery ||
            c.name.toLowerCase().includes(state.cleanQuery) ||
            c.desc.toLowerCase().includes(state.cleanQuery))
      );
      if (!list.length) return "";
      const onCount = list.filter((c) => state.selected.has(c.id)).length;
      return `<section class="cat-block">
        <div class="cat-head">
          <span class="action-ico" style="width:30px;height:30px;border-radius:9px">${svg(catIcons[cat.id] || "folder", 16)}</span>
          <h2>${esc(cat.name)}</h2>
          <span class="count">${onCount}/${list.length}</span>
          <span class="spacer"></span>
          <button class="btn link" data-cat-toggle="${cat.id}">${esc(t("selectAll"))}</button>
        </div>
        <div class="cleaner-list">
          ${list
            .map((c) => {
              const on = state.selected.has(c.id);
              const s = state.scan[c.id];
              const sizeHtml = s
                ? `<span class="sz ${s.size ? "" : "zero"}">${s.size ? fmtBytes(s.size) : s.count ? `${s.count}` : "—"}</span>`
                : "";
              const sens = cleanerSensitivity(c);
              const sensChip = sens === "sensitive"
                ? `<span class="chip danger">${esc(t("sensitivitySensitive"))}</span>`
                : sens === "review"
                ? `<span class="chip warn">${esc(t("sensitivityReview"))}</span>`
                : `<span class="chip safe">${esc(t("sensitivitySafe"))}</span>`;
              const chips = [
                sensChip,
                c.admin ? `<span class="chip admin">${esc(t("adminNeeded"))}</span>` : "",
              ].join("");
              const pathsLine = (c.paths||[]).slice(0,2).map(p=>esc(p)).join("<br>");
              const more = (c.paths||[]).length > 2 ? ` <span class="muted">+${(c.paths||[]).length-2}</span>` : "";
              return `<div class="cleaner ${on ? "on" : ""}" data-cleaner="${esc(c.id)}">
                <span class="check">${svg("check", 13)}</span>
                <div class="txt">
                  <div class="n">${esc(c.name)} ${chips}</div>
                  <div class="d">${esc(c.desc)}</div>
                  <div class="tiny muted" style="direction:ltr;text-align:start;margin-top:2px">${pathsLine}${more}</div>
                </div>
                ${sizeHtml}
              </div>`;
            })
            .join("")}
        </div>
      </section>`;
    })
    .join("");

  $("#view").innerHTML =
    pageHead(
      t("nav.clean"),
      t("subtitle"),
      `<span class="pill">${esc(t("selected"))}: <b id="selTotal">${fmtBytes(totalSelected)}</b></span>
       <button class="btn" id="cleanAnalyze">${svg("search")}${esc(t("analyze"))}</button>
       <button class="btn primary" id="cleanRun">${svg("broom")}${esc(t("clean"))}</button>`
    ) +
    `<div class="toolbar">
      <div class="search">${svg("search")}<input id="cleanSearch" placeholder="${esc(t("search"))}" value="${esc(state.cleanQuery)}"/></div>
      <button class="btn ghost small" id="selAll">${esc(t("selectAll"))}</button>
      <button class="btn ghost small" id="selNone">${esc(t("clearAll"))}</button>
    </div>
    <div id="cleanList">${body || `<div class="empty">${svg("search", 42)}<div>${esc(t("nothingFound"))}</div></div>`}</div>`;

  const search = $("#cleanSearch");
  search.oninput = () => {
    state.cleanQuery = search.value.toLowerCase();
    renderClean();
  };
  search.focus();
  search.setSelectionRange(search.value.length, search.value.length);

  $$("[data-cleaner]").forEach((el) =>
    el.addEventListener("click", () => {
      const id = el.dataset.cleaner;
      if (state.selected.has(id)) {
        state.selected.delete(id);
        el.classList.remove("on");
      } else {
        state.selected.add(id);
        el.classList.add("on");
      }
      // update the per-category counter badge
      const catBlock = el.closest(".cat-block");
      if (catBlock) {
        const head = catBlock.querySelector(".cat-head .count");
        if (head) {
          const catId = catBlock.querySelector("[data-cat-toggle]")?.dataset.catToggle;
          const list = state.catalog.cleaners.filter((c) => c.category === catId);
          const onCount = list.filter((c) => state.selected.has(c.id)).length;
          head.textContent = onCount + "/" + list.length;
        }
      }
      // keep the header total in sync with the new selection
      const totalEl = $("#selTotal");
      if (totalEl) {
        const sum = [...state.selected].reduce(
          (a, sid) => a + ((state.scan[sid] && state.scan[sid].size) || 0),
          0
        );
        totalEl.textContent = fmtBytes(sum);
      }
    })
  );
  $$("[data-cat-toggle]").forEach((b) =>
    b.addEventListener("click", () => {
      const cat = b.dataset.catToggle;
      const list = cleaners.filter((c) => c.category === cat);
      const allOn = list.every((c) => state.selected.has(c.id));
      list.forEach((c) => (allOn ? state.selected.delete(c.id) : state.selected.add(c.id)));
      go("clean");
    })
  );
  $("#selAll").onclick = () => {
    cleaners.forEach((c) => state.selected.add(c.id));
    go("clean");
  };
  $("#selNone").onclick = () => {
    state.selected.clear();
    go("clean");
  };

  $("#cleanAnalyze").onclick = async () => {
    const ids = [...state.selected];
    if (!ids.length) return toast(t("nothingSelected"), "warn");
    const res = await call("scan_cleaners", { ids }, t("scanning"));
    state.scan = {};
    for (const r of res) state.scan[r.id] = r;
    state.analyzed = true;
    toast(t("done"));
    go("clean");
  };

  $("#cleanRun").onclick = async () => {
    const ids = [...state.selected];
    if (!ids.length) return toast(t("nothingSelected"), "warn");
    let res = Object.values(state.scan);
    if (!state.analyzed || !res.length) {
      res = await call("scan_cleaners", { ids }, t("scanning"));
      state.scan = {};
      for (const r of res) state.scan[r.id] = r;
      state.analyzed = true;
    }
    const total = res.filter((r) => ids.includes(r.id)).reduce((a, r) => a + r.size, 0);
    const count = res.filter((r) => ids.includes(r.id)).reduce((a, r) => a + r.count, 0);
    if (!count) return toast(t("nothingFound"));

    const s = state.settings;
    if (s.previewFirst) {
      const files = res
        .filter((r) => ids.includes(r.id))
        .flatMap((r) => r.items.map((i) => ({ ...i, cleaner: r.id })))
        .slice(0, 260);
      const html = `<p class="muted" style="margin-top:0">${esc(t("previewHint"))} (${count})</p>
        <div class="filelist">${files
          .map(
            (f) => `<div class="f"><span>${esc(f.path)}</span><span>${fmtBytes(f.size)}</span></div>`
          )
          .join("")}</div>`;
      const go0 = await new Promise((resolve) => {
        modal({
          title: t("preview"),
          body: html,
          buttons: [
            { label: t("cancel"), cls: "ghost", onClick: () => resolve(false) },
            {
              label: `${t("clean")} (${fmtBytes(total)})`,
              cls: "primary",
              onClick: () => resolve(true),
            },
          ],
        });
      });
      if (!go0) return;
    } else if (s.confirmClean) {
      if (!(await confirmBox(t("confirmClean", { n: count, size: fmtBytes(total) }), t("clean")))) return;
    }

    const out = await call("clean_cleaners", { ids }, t("cleaning"));
    state.scan = {};
    state.analyzed = false;
    const errs = cleanErrorsText(out);
    modal({
      title: errs.length ? t("partialClean") : t("done"),
      body: `<div style="text-align:center;padding:10px 0">
          <div style="width:64px;height:64px;border-radius:50%;background:var(--accent-soft);color:var(--accent);display:flex;align-items:center;justify-content:center;margin:0 auto 14px">${svg(errs.length?"alert":"check", 32)}</div>
          <div style="font-size:16px;font-weight:700">${esc(t("cleanResult", { n: out.removed, size: fmtBytes(out.freed) }))}</div>
          ${out.skipped ? `<div class="muted tiny" style="margin-top:6px">${esc(t("skipped", { n: out.skipped }))}</div>` : ""}
          ${out.restore === "ok" ? `<div class="tiny" style="margin-top:8px;color:var(--accent);display:flex;align-items:center;justify-content:center;gap:6px">${svg("shield", 14)}${esc(t("restoreCreatedShort"))}</div>` : ""}
          ${errs.length ? `<div class="tiny" style="margin-top:10px;color:#f59e0b;text-align:start;direction:ltr;max-height:120px;overflow:auto">${errs.map(e=>`<div>• ${esc(String(e))}</div>`).join("")}</div>` : ""}
        </div>`,
      buttons: [{ label: t("ok"), cls: "primary" }],
    });
  };
}

/* ---------------------------------------------------------------------- TUNE */
async function renderTune() {
  const [tweaks, health, procs, disks, changes] = await Promise.all([
    call("tweaks_list"),
    call("get_health"),
    call("processes", { limit: 12 }),
    call("disk_health"),
    call("tweak_change_log").catch(() => []),
  ]);
  state.tweaks = tweaks;
  state.health = health;

  const memPct = health.memTotal ? (health.memUsed / health.memTotal) * 100 : 0;
  const groups = [
    ["performance", t("tweaksPerf"), "zap"],
    ["privacy", t("tweaksPrivacy"), "shield"],
    ["system", t("tweaksSystem"), "settings"],
  ];

  const tweaksHtml = groups
    .map(([cat, label, ic]) => {
      const list = tweaks.filter((x) => x.cat === cat);
      if (!list.length) return "";
      return `<div class="card flat" style="margin-bottom:14px">
        <div class="card-title">${svg(ic)}${esc(label)}</div>
        ${list
          .map(
            (x) => `<div class="setting">
              <div class="txt">
                <div class="t">${esc(x.name)} ${x.admin ? `<span class="chip admin">${esc(t("adminNeeded"))}</span>` : ""} ${x.risk >= 1 ? `<span class="chip warn">${esc(t("caution"))}</span>` : ""}</div>
                <div class="s">${esc(x.desc)}</div>
              </div>
              <div class="toggle ${x.applied ? "on" : ""}" data-tweak="${esc(x.id)}"></div>
            </div>`
          )
          .join("")}
      </div>`;
    })
    .join("");

  const procRows = procs
    .map(
      (p) => `<tr>
      <td><div style="font-weight:600">${esc(p.name)}</div><div class="tiny muted ellipsis" style="max-width:360px;direction:ltr;text-align:start">${esc(p.exe||"")}</div></td>
      <td class="num">${p.pid}</td>
      <td class="num">${fmtBytes(p.mem)}</td>
      <td style="text-align:end">${p.system ? `<span class="pill warn">${esc(state.lang==="ar"?"محمي":"Protected")}</span>` : `<button class="btn small danger" data-kill="${p.pid}">${esc(t("endTask"))}</button>`}</td>
    </tr>`
    )
    .join("");

  const diskRows = disks.length
    ? disks
        .map(
          (d) => {
            const h = String(d.health||"");
            const cls = h === "Unknown" ? "warn" : (h.startsWith("SMART:") || h.toLowerCase().includes("healthy")) ? "" : "warn";
            return `<tr><td>${esc(d.name)}</td><td class="num">${esc(String(d.media))}</td>
          <td><span class="pill ${cls}">${esc(h)}</span></td>
          <td class="num">${fmtBytes(d.size)}</td></tr>`;
          }
        )
        .join("")
    : `<tr><td colspan="4" class="muted">${esc(t("unknown"))}</td></tr>`;

  $("#view").innerHTML =
    pageHead(t("nav.tune"), t("subtitle"), `<button class="btn" id="refreshTune">${svg("refresh")}</button>`) +
    `<div class="grid cols-2">
      <div class="card flat">
        <div class="card-title">${svg("memory")}${esc(t("memory"))}</div>
        <div style="display:flex;align-items:baseline;gap:8px;margin:8px 0">
          <div style="font-size:24px;font-weight:700">${fmtBytes(health.memUsed)}</div>
          <div class="muted tiny">/ ${fmtBytes(health.memTotal)}</div>
        </div>
        <div class="progress ${memPct > 85 ? "danger" : memPct > 65 ? "warn" : ""}"><i style="width:${memPct.toFixed(1)}%"></i></div>
        <div class="tiny muted" style="margin-top:6px">${memPct.toFixed(0)}% ${esc(state.lang === "ar" ? "مستخدم" : "used")}</div>
        <button class="btn primary" id="boostBtn" style="margin-top:14px;width:100%;justify-content:center">${svg("zap")}${esc(t("boost"))}</button>
      </div>
      <div class="card flat">
        <div class="card-title">${svg("disk")}${esc(t("diskHealth"))}</div>
        <table class="table"><thead><tr><th>${esc(t("disk"))}</th><th>Type</th><th>Health</th><th>${esc(t("size"))}</th></tr></thead>
        <tbody>${diskRows}</tbody></table>
      </div>
    </div>
    <div style="height:16px"></div>
    ${tweaksHtml}
    <div class="card flat" style="margin-bottom:14px">
      <div class="card-title">${svg("clock")}${esc(t("changeLog"))}</div>
      ${(changes||[]).length ? `<div class="filelist">${(changes||[]).slice(0,12).map(c=>`<div class="f" style="align-items:flex-start;gap:10px">
        <span style="min-width:0;flex:1">
          <div class="tiny" style="font-weight:600">${esc(c.id)}</div>
          <div class="tiny muted">${esc(new Date((c.at||0)*1000).toLocaleString())} • ${esc(c.action)} • ${c.on ? "on" : "off"} ${c.note? "• "+esc(c.note):""}</div>
        </span>
        <button class="btn small ghost" data-tweak-restore="${esc(c.id)}">${esc(t("restoreOriginal"))}</button>
      </div>`).join("")}</div>` : `<div class="tiny muted">${esc(t("noChanges"))}</div>`}
      <div class="tiny muted" style="margin-top:8px">${esc(t("restorePointDesc"))}</div>
    </div>
    <div class="card flat">
      <div class="card-title">${svg("cpu")}${esc(t("processes"))}</div>
      <table class="table"><thead><tr><th>${esc(t("name"))}</th><th>PID</th><th>${esc(t("memory"))}</th><th></th></tr></thead>
      <tbody>${procRows}</tbody></table>
    </div>`;

  $("#refreshTune").onclick = () => go("tune");
  $$("[data-tweak-restore]").forEach((b)=> b.onclick = async ()=>{ try { await call("tweak_restore", { id: b.dataset.tweakRestore }, t("cleaning")); toast(t("changeRestored")); go("tune"); } catch(e){} });
  $("#boostBtn").onclick = async () => {
    const [freed, trimmed] = await call("free_memory", {}, t("cleaning"));
    toast(t("ramFreed", { size: fmtBytes(freed) }));
    void trimmed;
    go("tune");
  };
  $$("[data-tweak]").forEach((el) =>
    el.addEventListener("click", async () => {
      const id = el.dataset.tweak;
      const tw = state.tweaks.find((x) => x.id === id);
      if (!tw) return;
      // Explicit consent before any registry/setting change (project rule).
      const riskTag = tw.risk >= 2 ? "🔴 " : tw.risk === 1 ? "🟡 " : "";
      if (!(await confirmBox(`${riskTag}${tw.name} • ${tw.desc} • ${t("tweakConfirm")}`, t("nav.tune")))) return;
      try {
        if (state.settings.restoreBeforeTweaks) {
          try { await call("restore_create", { description: "VEXORA before tweak: " + id }, t("creatingRestore")); }
          catch (e) { toast(t("restoreFailed") + ": " + friendlyError(e), "warn"); }
        }
        await call("tweak_set", { id, on: !tw.applied });
        toast(t("saved"));
        go("tune");
      } catch {
        toast(t("applyFailed"), "error");
      }
    })
  );
  $$("[data-kill]").forEach((b) =>
    b.addEventListener("click", async () => {
      const pid = Number(b.dataset.kill);
      const p = (procs || []).find((x) => x.pid === pid);
      if (p && p.system) { toast(t("error"), "warn"); return; }
      const msg = p
        ? `${t("endTask")}?
${p.name} (PID ${pid})
${p.exe || ""}
RAM: ${fmtBytes(p.mem || 0)}`
        : t("confirmDelete");
      if (!(await confirmBox(msg, t("endTask")))) return;
      const ok = await call("kill_process", { pid });
      toast(ok ? t("done") : t("error"), ok ? "ok" : "error");
      go("tune");
    })
  );
}

/* ------------------------------------------------------------------ REGISTRY */
async function renderRegistry() {
  const issues = state.issues;
  const rows = issues
    .map(
      (i) => `<tr>
      <td style="width:34px"><div class="check ${i._sel ? "on" : ""}" data-issue="${i.id}" style="background:${i._sel ? "var(--accent)" : "var(--card-2)"};border-color:${i._sel ? "var(--accent)" : "var(--line)"};color:${i._sel ? "#05130a" : "transparent"}">${svg("check", 13)}</div></td>
      <td><div style="font-weight:600">${esc(state.lang === "ar" ? i.titleAr : i.titleEn)}</div>
          <div class="tiny muted ellipsis" style="max-width:520px">${esc(i.detail)}</div></td>
      <td style="text-align:end">${i.advanced ? `<span class="chip danger">${esc(t("danger"))}</span>` : `<span class="chip">${esc(t("ok"))}</span>`}</td>
    </tr>`
    )
    .join("");

  $("#view").innerHTML =
    pageHead(
      t("nav.registry"),
      t("subtitle"),
      `<button class="btn" id="regScan">${svg("search")}${esc(t("registryScan"))}</button>
       ${issues.length ? `<button class="btn ghost" id="regBackup">${svg("file")}${esc(t("backup"))}</button>
       <button class="btn primary" id="regFix">${svg("shield")}${esc(t("registryFix"))}</button>` : ""}`
    ) +
    (issues.length
      ? `<div class="card flat">
          <div class="card-title">${svg("database")}${issues.length} ${esc(t("issues"))}
            <span style="margin-inline-start:auto"></span>
            <button class="btn link" id="regSelAll">${esc(t("selectAll"))}</button>
          </div>
          <table class="table"><tbody>${rows}</tbody></table>
        </div>`
      : `<div class="empty">${svg("shield", 44)}<div>${esc(t("registryClean"))}</div></div>`);

  if ($("#regScan"))
    $("#regScan").onclick = async () => {
      const list = await call("registry_scan", {}, t("scanning"));
      state.issues = list.map((i) => ({ ...i, _sel: !i.advanced }));
      toast(`${list.length} ${t("issues")}`);
      go("registry");
    };

  $$("[data-issue]").forEach((el) =>
    el.addEventListener("click", () => {
      const id = Number(el.dataset.issue);
      const it = state.issues.find((x) => x.id === id);
      if (it) it._sel = !it._sel;
      go("registry");
    })
  );
  if ($("#regSelAll"))
    $("#regSelAll").onclick = () => {
      const all = state.issues.every((i) => i._sel);
      state.issues.forEach((i) => (i._sel = !all));
      go("registry");
    };
  if ($("#regBackup"))
    $("#regBackup").onclick = async () => {
      const path = await call("registry_backup_only");
      modal({
        title: t("backup"),
        body: `<p class="muted">${esc(t("backupAt", { path }))}</p><div class="kv"><span>Path</span><span>${esc(path)}</span></div>`,
        buttons: [
          { label: t("openFolder"), onClick: () => invoke("reveal_path", { path }) },
          { label: t("ok"), cls: "primary" },
        ],
      });
    };
  if ($("#regFix"))
    $("#regFix").onclick = async () => {
      const sel = state.issues.filter((i) => i._sel).map((i) => i.id);
      if (!sel.length) return toast(t("nothingSelected"), "warn");
      if (!(await confirmBox(t("fixConfirm", { n: sel.length }), t("registryFix")))) return;
      const out = await call("registry_fix", { ids: sel }, t("cleaning"));
      modal({
        title: t("done"),
        body: `<p>${esc(t("fixed", { n: out.fixed }))}</p>
          <div class="tiny muted">${esc(t("backupAt", { path: out.backup }))}</div>
          ${out.errors.length ? `<div class="tiny" style="color:var(--warn);margin-top:10px">${esc(out.errors.join("\n"))}</div>` : ""}`,
        buttons: [{ label: t("ok"), cls: "primary" }],
      });
      const list = await call("registry_scan");
      state.issues = list.map((i) => ({ ...i, _sel: !i.advanced }));
      go("registry");
    };
}

/* ------------------------------------------------------------------- STARTUP */
async function renderStartup() {
  const items = await call("startup_list");
  state.startup = items;
  // Load icons + publishers for the startup executables.
  try {
    const exes = [...new Set(items.map((it) => {
      const m = (it.command || "").match(/"([^"]+\.exe)"/i) || (it.command || "").match(/(\S+\.exe)/i);
      return m ? m[1] : "";
    }).filter(Boolean))];
    const extras = await call("startup_extras", { exes });
    state.startupExtras = extras || { icons: {}, publishers: {} };
  } catch (e) { state.startupExtras = { icons: {}, publishers: {} }; }
  const extras = state.startupExtras || { icons: {}, publishers: {} };
  const enabled = items.filter((i) => i.enabled).length;
  const safeItems = items.filter((i) => i.enabled && i.safeToDisable);

  const rows = items
    .map(
      (it, idx) => {
        const m = (it.command || "").match(/"([^"]+\.exe)"/i) || (it.command || "").match(/(\S+\.exe)/i);
        const exe = m ? m[1] : "";
        const icon = extras.icons[exe] || extras.icons[exe.toLowerCase()];
        const pub = it.publisher || extras.publishers[exe] || extras.publishers[exe.toLowerCase()] || "";
        return `<tr>
      <td style="width:30px"><div class="toggle ${it.enabled ? "on" : ""}" data-st="${idx}"></div></td>
      <td>
        <div style="display:flex;align-items:center;gap:8px">
          <span class="prog-icon-wrap ${icon ? "has-icon" : ""}" style="width:24px;height:24px;flex-shrink:0">
            <span class="prog-icon-ph">${svg("rocket", 14)}</span>
            <img class="prog-icon ${icon ? "loaded" : ""}" src="${icon ? "data:image/png;base64," + icon : ""}" alt="" draggable="false" />
          </span>
          <div>
            <div style="font-weight:600">${esc(it.name)} ${it.exists ? "" : `<span class="chip warn">missing</span>`}
              ${it.safeToDisable ? `<span class="chip ok" title="${esc(it.suggestion)}">${esc(t("safeToDisable"))}</span>` : `<span class="chip">${esc(t("essential"))}</span>`}
            </div>
            <div class="tiny muted ellipsis" style="max-width:520px;direction:ltr;text-align:start">${esc(it.command)}</div>
            ${pub ? `<div class="tiny muted">${esc(pub)}</div>` : ""}
          </div>
        </div>
      </td>
      <td><span class="pill">${esc(state.lang === "ar" ? it.sourceAr : it.sourceEn)}</span></td>
      <td style="text-align:end">
        <button class="btn small ghost" data-sreopen="${idx}">${svg("folder", 14)}</button>
        <button class="btn small danger" data-sdel="${idx}">${esc(t("delete"))}</button>
      </td>
    </tr>`;
      }
    )
    .join("");

  $("#view").innerHTML =
    pageHead(
      t("startupTitle"),
      t("subtitle"),
      `<span class="pill ok">${enabled} ${esc(t("enabled"))}</span>
       <span class="pill">${items.length - enabled} ${esc(t("disabled"))}</span>
       <button class="btn small" id="stSelSafe">${esc(t("selectSafe"))} (${safeItems.length})</button>
       <button class="btn small danger" id="stDisableSel" disabled>${esc(t("disableSelected"))}</button>
       <button class="btn" id="stRefresh">${svg("refresh")}</button>`
    ) +
    (items.length
      ? `<div class="card flat"><table class="table">
          <thead><tr><th></th><th>${esc(t("name"))}</th><th>${esc(t("startupTitle"))}</th><th></th></tr></thead>
          <tbody>${rows}</tbody></table></div>`
      : `<div class="empty">${svg("rocket", 44)}<div>${esc(t("nothingFound"))}</div></div>`);

  $("#stRefresh").onclick = () => go("startup");

  $("#stSelSafe").onclick = async () => {
    const safeIdx = items.map((it, idx) => (it.enabled && it.safeToDisable ? idx : -1)).filter((x) => x >= 0);
    if (!safeIdx.length) return toast(t("nothingFound"), "warn");
    const names = safeIdx.map((i) => items[i].name).join("\n");
    if (!(await confirmBox(`${t("confirmDisableBatch")}\n${names}`, t("disableSelected")))) return;
    for (const idx of safeIdx) {
      const it = state.startup[idx];
      if (!it || !it.enabled) continue;
      try { await call("startup_set", { source: it.source, name: it.name, enabled: false }); }
      catch (e) { toast(String(e), "error"); }
    }
    toast(t("done"));
    go("startup");
  };
  if ($("#stDisableSel")) $("#stDisableSel").style.display = "none";

  $$("[data-st]").forEach((el) =>
    el.addEventListener("click", async () => {
      const it = state.startup[Number(el.dataset.st)];
      if (!it) return;
      if (it.enabled && !(await confirmBox(t("confirmDisable"), t("disable")))) return;
      try {
        await call("startup_set", { source: it.source, name: it.name, enabled: !it.enabled });
        toast(t("saved"));
      } catch (e) {
        toast(String(e), "error");
      }
      go("startup");
    })
  );
  $$("[data-sdel]").forEach((b) =>
    b.addEventListener("click", async () => {
      const it = state.startup[Number(b.dataset.sdel)];
      if (!it) return;
      if (!(await confirmBox(t("confirmDelete"), t("delete")))) return;
      try {
        await call("startup_remove", { source: it.source, name: it.name });
        toast(t("done"));
      } catch (e) {
        toast(String(e), "error");
      }
      go("startup");
    })
  );
  $$("[data-sreopen]").forEach((b) =>
    b.addEventListener("click", () => {
      const it = state.startup[Number(b.dataset.sreopen)];
      if (it) invoke("reveal_path", { path: it.command });
    })
  );
}

async function renderBgPrograms() {
  const procs = await call("processes", { limit: 250 }, t("scanning"));
  state.bgProcs = procs;
  // Load icons for the (unique) executables.
  try {
    const exes = [...new Set(procs.map((p) => p.exe).filter(Boolean))];
    const icons = await call("process_icons", { exes });
    const map = icons || {};
    // Normalize keys (lowercase) for robust lookup.
    const norm = {};
    for (const [k, v] of Object.entries(map)) norm[k.toLowerCase()] = v;
    state.bgIcons = { ...state.bgIcons, ...norm };
    // Also keep original keys.
    state.bgIcons = { ...state.bgIcons, ...map };
  } catch (e) { console.error("process_icons failed", e); }
  const showSystem = state.bgShowSystem || false;
  const list = showSystem ? procs : procs.filter((p) => !p.system);
  const sel = state.bgSel || new Set();
  const rows = list
    .map(
      (p) => `<tr>
      <td style="width:30px"><input type="checkbox" class="bg-check" data-bgp="${p.pid}" ${sel.has(p.pid) ? "checked" : ""} ${p.system ? "disabled" : ""} /></td>
      <td>
        <div style="display:flex;align-items:center;gap:8px">
          <span class="prog-icon-wrap ${state.bgIcons[p.exe] || state.bgIcons[p.exe.toLowerCase()] ? "has-icon" : ""}" style="width:24px;height:24px;flex-shrink:0">
            <span class="prog-icon-ph">${svg("apps", 16)}</span>
            <img class="prog-icon ${(state.bgIcons[p.exe] || state.bgIcons[p.exe.toLowerCase()]) ? "loaded" : ""}" src="${(state.bgIcons[p.exe] || state.bgIcons[p.exe.toLowerCase()]) ? "data:image/png;base64," + (state.bgIcons[p.exe] || state.bgIcons[p.exe.toLowerCase()]) : ""}" alt="" draggable="false" />
          </span>
          <div>
            <div style="font-weight:600">${esc(p.name)} ${p.system ? `<span class="chip">${esc(t("system"))}</span>` : ""} ${p.hasTrayIcon ? `<span class="chip ok">tray</span>` : ""}</div>
            <div class="tiny muted ellipsis" style="max-width:420px;direction:ltr;text-align:start">${esc(p.exe)}</div>
            ${p.publisher ? `<div class="tiny muted">${esc(p.publisher)}</div>` : ""}
          </div>
        </div>
      </td>
      <td class="num">${fmtBytes(p.mem)}</td>
      <td class="num">${(p.cpu || 0).toFixed(1)}%</td>
      <td style="text-align:end">
        ${p.system ? `<span class="tiny muted">—</span>` : `<button class="btn small ghost" data-bgp-close="${p.pid}">${esc(t("closeGraceful"))}</button>
        <button class="btn small danger" data-bgp-kill="${p.pid}">${esc(t("forceKill"))}</button>`}
      </td>
    </tr>`
    )
    .join("");

  $("#view").innerHTML =
    pageHead(t("nav.bgprograms"), t("subtitle"), `<button class="btn" id="bgRefresh">${svg("refresh")}</button>`) +
    `<div class="card flat" style="margin-bottom:12px">
      <div class="field-row" style="align-items:center">
        <label class="chip" style="cursor:pointer"><input type="checkbox" id="bgShowSystem" ${showSystem ? "checked" : ""} style="accent-color:var(--accent);margin-inline-end:6px" /> ${esc(t("showSystem"))}</label>
        <span class="spacer"></span>
        <button class="btn small primary" id="bgFreeMem">${svg("zap",14)}${esc(t("freeMemory"))}</button>
        <button class="btn small danger" id="bgCloseSel" disabled>${esc(t("closeSelected"))}</button>
      </div>
      <div class="tiny muted" style="margin-top:8px">${esc(t("bgProgramsHint"))}</div>
    </div>
    <div class="card flat">
      <table class="table">
        <thead><tr><th style="width:30px"></th><th>${esc(t("name"))}</th><th>${esc(t("memory"))}</th><th>CPU</th><th></th></tr></thead>
        <tbody>${rows || `<tr><td colspan="5" class="muted">${esc(t("nothingFound"))}</td></tr>`}</tbody>
      </table>
    </div>`;

  const updateSel = () => {
    const n = $$("[data-bgp]:checked").length;
    const btn = $("#bgCloseSel");
    if (btn) btn.disabled = !n;
    state.bgSel = new Set($$("[data-bgp]:checked").map((c) => Number(c.dataset.bgp)));
  };
  $$("[data-bgp]").forEach((c) => (c.onchange = updateSel));
  updateSel();

  $("#bgShowSystem").onchange = (e) => { state.bgShowSystem = e.target.checked; go("bgprograms"); };
  $("#bgRefresh").onclick = () => go("bgprograms");
  $("#bgFreeMem").onclick = async () => {
    const r = await call("free_memory", {}, t("cleaning"));
    toast(`${t("done")} — ${fmtBytes(r[0] || 0)} freed, ${r[1] || 0} processes`);
    go("bgprograms");
  };
  $("#bgCloseSel").onclick = async () => {
    const pids = [...(state.bgSel || [])];
    if (!pids.length) return;
    if (!(await confirmBox(t("confirmCloseBg"), t("closeSelected")))) return;
    let n = 0;
    for (const pid of pids) {
      try {
        const r = await call("safe_close_process", { pid });
        if (r === "closed" || r === "stillRunning") n++;
        else toast(`${pid}: ${r}`, "warn");
      } catch (e) { /* reported */ }
    }
    toast(`${t("done")} (${n})`);
    go("bgprograms");
  };
  $$("[data-bgp-close]").forEach((b) =>
    b.addEventListener("click", async () => {
      const pid = Number(b.dataset.bgpClose);
      try {
        const r = await call("safe_close_process", { pid }, t("cleaning"));
        toast(r === "closed" ? t("done") : String(r));
      } catch (e) { /* reported */ }
      go("bgprograms");
    })
  );
  $$("[data-bgp-kill]").forEach((b) =>
    b.addEventListener("click", async () => {
      const pid = Number(b.dataset.bgpKill);
      if (!(await confirmBox(t("confirmForceKill"), t("forceKill")))) return;
      try { await call("force_kill_process", { pid }); toast(t("done")); }
      catch (e) { /* reported */ }
      go("bgprograms");
    })
  );
}

/* ---------------------------------------------------------------- UNINSTALL */
function progIconUrl(key) {
  const b64 = state.progIcons[key];
  return b64 ? `data:image/png;base64,${b64}` : "";
}
function progSizeKb(p) {
  return p && typeof p.actualSizeKb === "number" ? p.actualSizeKb : (p && p.sizeKb) || 0;
}
function progTypeLabel(p) {
  const map = { win32: "typeWin32", msi: "typeMsi", store: "typeStore", portable: "typePortable" };
  return t(map[(p && p.installType) || "unknown"] || "typeUnknown");
}
function progSourceLabel(p) {
  const map = {
    machineUninstallKey: "HKLM Uninstall",
    userUninstallKey: "HKCU Uninstall",
    msiProvider: "MSI Provider",
    storeEntry: "Microsoft Store",
    manualAdd: "Manual",
  };
  return map[p && p.installSource] || t("typeUnknown");
}
function progSafetyLabel(p) {
  const map = { safe: "safe", review: "review", sensitive: "sensitive" };
  return t(map[(p && p.protectionLevel) || "safe"] || "safe");
}

function progIconWrap(p, size, cls) {
  const src = progIconUrl(p.key);
  return `<span class="prog-icon-wrap ${cls}" style="width:${size}px;height:${size}px">
    <span class="prog-icon-ph">${svg("apps", Math.round(size * 0.7))}</span>
    <img class="prog-icon${src ? " loaded" : ""}" data-icon-key="${esc(p.key)}" src="${esc(src)}" alt="" draggable="false" />
  </span>`;
}
function applyProgIcons() {
  $$("img[data-icon-key]").forEach((img) => {
    const src = progIconUrl(img.dataset.iconKey);
    if (!src) return;
    if (img.getAttribute("src") !== src) img.setAttribute("src", src);
    img.classList.add("loaded");
    if (img.parentElement) img.parentElement.classList.add("has-icon");
  });
}
function saveProgIcons() {
  try {
    const o = Object.assign({}, state.progIcons, { __t: Date.now() });
    localStorage.setItem("vexora_prog_icons", JSON.stringify(o));
  } catch (e) { /* storage full — memory cache still works */ }
}
async function loadProgIcons(list) {
  const need = list.map((p) => p.key).filter((k) => !state.progIcons[k]);
  if (!need.length) return;
  try {
    const map = await call("program_icons", { keys: need });
    if (map && typeof map === "object") {
      Object.assign(state.progIcons, map);
      saveProgIcons();
    }
  } catch (e) { /* icons are optional */ }
  applyProgIcons();
}
function progFolder(p) {
  if (p.location && p.location.trim()) return p.location.trim();
  const di = String(p.displayIcon || "").split(",")[0].trim();
  if (!di) return "";
  const i = Math.max(di.lastIndexOf("\\"), di.lastIndexOf("/"));
  return i > 0 ? di.slice(0, i) : "";
}
function copyText(text) {
  const fallback = () => {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    try { document.execCommand("copy"); } catch (e) {}
    ta.remove();
  };
  if (navigator.clipboard && navigator.clipboard.writeText)
    navigator.clipboard.writeText(text).catch(fallback);
  else fallback();
}
function closeProgMenu() {
  const m = $("#progCtxMenu");
  if (m) m.remove();
}
function openProgMenu(key, x, y) {
  closeProgMenu();
  const p = state.progs.find((z) => z.key === key);
  if (!p) return;
  const menu = document.createElement("div");
  menu.id = "progCtxMenu";
  menu.className = "ctx-menu";
  menu.innerHTML = `
    <button data-act="uninstall">${svg("trash", 14)}<span>${esc(t("uninstall"))}</span></button>
    <button data-act="details">${svg("search", 14)}<span>${esc(t("leftovers"))}</span></button>
    <button data-act="open">${svg("folder", 14)}<span>${esc(t("openFolder"))}</span></button>
    <button data-act="copy">${svg("apps", 14)}<span>${esc(t("copyName"))}</span></button>
    <div class="ctx-sep"></div>
    <button data-act="remove" class="danger">${svg("x", 14)}<span>${esc(t("removeEntry"))}</span></button>`;
  document.body.appendChild(menu);
  const w = menu.offsetWidth, h = menu.offsetHeight;
  menu.style.left = Math.max(4, Math.min(x, window.innerWidth - w - 6)) + "px";
  menu.style.top = Math.max(4, Math.min(y, window.innerHeight - h - 6)) + "px";
  menu.querySelectorAll("button").forEach((b) => {
    b.onclick = async (e) => {
      e.stopPropagation();
      closeProgMenu();
      const act = b.dataset.act;
      if (act === "uninstall") {
        if (!(await confirmBox(p.name, t("uninstall")))) return;
        try {
          const out = await runUninstall(key, p.name);
          state.progSel.delete(key);
          if (out.ok) state.progs = state.progs.filter((z) => z.key !== key);
        } catch (err) { /* call() already reported the failure */ }
        go("uninstall");
      } else if (act === "details") {
        showProgDetails(key);
      } else if (act === "open") {
        const f = progFolder(p);
        if (!f) return toast(t("nothingFound"), "warn");
        try { await invoke("reveal_path", { path: f }); } catch (err) { toast(String(err), "error"); }
      } else if (act === "copy") {
        copyText(p.name);
        toast(t("done"));
      } else if (act === "remove") {
        if (!(await confirmBox(t("confirmDelete"), t("removeEntry")))) return;
        try {
          await call("program_remove_entry", { key });
          state.progs = state.progs.filter((z) => z.key !== key);
          state.progSel.delete(key);
          toast(t("done"));
          go("uninstall");
        } catch (err) { toast(String(err), "error"); }
      }
    };
  });
  setTimeout(() => document.addEventListener("click", closeProgMenu, { once: true }), 0);
}
function cleanupReason(reason) {
  if (reason.startsWith("deadShortcut:")) {
    const label = state.lang === "ar" ? "اختصار لا يعمل (الهدف مفقود)" : "Dead shortcut (target missing)";
    return `${label} → ${reason.slice("deadShortcut:".length)}`;
  }
  const ar = {
    match: "مطابق للبرنامج",
    protected: "مجلد محمي — لا يمكن حذفه",
    reparse: "رابط نظام (Junction) — تم تجاهله",
    otherProgram: "يستخدمه برنامج آخر مثبّت",
    containsOther: "يحتوي على برنامج آخر",
    folder: "مجلد متبقٍ",
    registry: "مفتاح إلغاء تثبيت متبقٍ",
    installDir: "مجلد التثبيت",
    brokenUninstall: "مفتاح تالف — أداة إلغاء التثبيت مفقودة",
    orphanFolder: "مجلد يتيم يحتوي على أداة إلغاء تثبيت",
    shortcut: "اختصار مكسور",
    deadShortcut: "اختصار لا يعمل (الهدف مفقود)",
  };
  const en = {
    match: "Matched program",
    protected: "Protected folder — cannot be deleted",
    reparse: "Junction or symlink — skipped",
    otherProgram: "Used by another installed program",
    containsOther: "Contains another program",
    folder: "Leftover folder",
    registry: "Leftover registry key",
    installDir: "Install folder",
    brokenUninstall: "Broken entry — uninstaller missing",
    orphanFolder: "Orphan folder containing an uninstaller",
    shortcut: "Dead shortcut",
    deadShortcut: "Dead shortcut (target missing)",
  };
  return (state.lang === "ar" ? ar : en)[reason] || reason;
}
function showCleanupPlan(title, plan, okLabel) {
  return new Promise((resolve) => {
    if (!plan.items || !plan.items.length) {
      toast(t("nothingFound"), "warn");
      return resolve(null);
    }
    let settled = false;
    const done = (v) => { if (settled) return; settled = true; resolve(v); };
    const rows = plan.items
      .map((it, i) => `<label class="cleanup-row ${it.safe ? "" : "blocked"}">
        <input type="checkbox" data-cl="${i}" ${it.safe ? "checked" : "disabled"} />
        <span class="grow min0">
          <span class="tiny ellipsis" style="direction:ltr;text-align:start;display:block">${esc(it.path)}</span>
          <span class="tiny muted">${esc(cleanupReason(it.reason))}${it.size ? " • " + fmtBytes(it.size) : ""}</span>
        </span>
      </label>`)
      .join("");
    modal({
      title: `${title} (${plan.safeCount}/${plan.items.length})`,
      body: `<div class="tiny muted" style="margin-bottom:8px">${esc(t("cleanupPlanHint"))}</div>
        <div class="cleanup-list">${rows}</div>`,
      onClose: (reason) => {
        if (reason !== "ok") return done(null);
        done($("[data-cl]:checked").map((c) => plan.items[Number(c.dataset.cl)].path));
      },
      buttons: [
        { label: t("cancel"), cls: "ghost", reason: "cancel" },
        { label: okLabel, cls: "danger", reason: "ok" },
      ],
    });
  });
}

/** Convert leftover results into a CleanupPlan-shaped object for review. */
function leftoversToPlan(lv) {
  const items = lv.map((l) => ({
    path: l.path,
    kind: l.kind,
    size: l.size,
    safe: true,
    reason: l.kind,
  }));
  return { items, safeCount: items.length, blockedCount: 0 };
}

/** Mandatory review dialog before any leftover is deleted. */
async function reviewLeftovers(lv, title) {
  if (!lv || !lv.length) return null;
  const paths = await showCleanupPlan(
    title || t("leftoversReview"),
    leftoversToPlan(lv),
    t("deleteSelected")
  );
  if (!paths || !paths.length) return null;
  const out = await call("delete_leftovers", { paths }, t("cleaning"));
  toast(`${t("done")} (${out.removed})${out.failed ? ` • ${t("failedItems")}: ${out.failed}` : ""}`);
  return out;
}

/**
 * Run a program's uninstaller, wait for the backend to verify the registry
 * entry is gone, report failures loudly, then review leftover files.
 */
async function runUninstall(key, name) {
  const out = await call("program_uninstall", { key, name }, t("uninstalling"));
  if (out.ok) {
    if (out.leftovers && out.leftovers.length) toast(t("uninstallDone"));
    else toast(`${t("uninstallDone")} — ${t("noLeftoversHint")}`);
  } else
    toast(
      `${t("uninstall_" + out.message)}${out.detail ? " — " + out.detail : ""}`,
      "error"
    );
  if (out.ok) await reviewLeftovers(out.leftovers);
  return out;
}

function showProgDetails(key) {
  const p = state.progs.find((x) => x.key === key);
  if (!p) return;
  const m = modal({
    title: p.name,
    body: `<div class="prog-details-head">${progIconWrap(p, 44, "grid")}
        <div class="grow min0"><div style="font-weight:600">${esc(p.name)}</div><div class="tiny muted">${esc(p.publisher || "")}</div></div>
      </div>
      <div class="kv"><span>${esc(t("publisher"))}</span><span>${esc(p.publisher || "—")}</span></div>
      <div class="kv"><span>${esc(t("version"))}</span><span>${esc(p.version || "—")}</span></div>
      <div class="kv"><span>${esc(t("size"))}</span><span>${progSizeKb(p) ? fmtBytes(progSizeKb(p) * 1024) : "—"}${
        typeof p.actualSizeKb === "number" ? ` <span class="tiny muted">(${esc(t("progSizeMeasured"))})</span>` : ""
      }</span></div>
      <div class="kv"><span>${esc(t("progInstallType"))}</span><span>${esc(progTypeLabel(p))}</span></div>
      <div class="kv"><span>${esc(t("progSource"))}</span><span>${esc(progSourceLabel(p))}</span></div>
      <div class="kv"><span>${esc(t("progSafety"))}</span><span>${esc(progSafetyLabel(p))}</span></div>
      <div class="kv"><span>${esc(t("installedOn"))}</span><span>${esc(p.installDate || "—")}</span></div>
      <div class="kv"><span>Path</span><span class="ellipsis" style="max-width:360px;direction:ltr;text-align:start">${esc(p.location || "—")}</span></div>
      <div style="margin-top:8px;display:flex;gap:8px;align-items:center">
        <span class="tiny muted">${esc(t("deepScanLevel"))}</span>
        <select id="leftoverLevel" class="btn" style="padding:6px 10px">
          <option value="safe">${esc(t("safe"))}</option>
          <option value="moderate">${esc(t("moderate"))}</option>
          <option value="advanced" selected>${esc(t("advanced"))}</option>
        </select>
      </div>
      <div id="leftovers" style="margin-top:12px"></div>`,
    buttons: [
      { label: t("close"), cls: "ghost" },
      {
        label: t("leftovers"),
        onClick: async () => {
          const lvl = ($("#leftoverLevel") && $("#leftoverLevel").value) || "advanced";
          const lv = await call("leftovers_level", { key, level: lvl }, t("scanning"));
          const box = $("#leftovers");
          if (!box) return;
          if (!lv.length) {
            box.innerHTML = `<div class="tiny muted">${esc(t("noLeftovers"))}</div>`;
            return;
          }
          box.innerHTML = `<div class="tiny muted" style="margin-bottom:6px">${lv.length} ${esc(t("leftovers"))}</div>
            <div class="filelist">${lv
              .map((l) => `<div class="f"><span>${esc(l.path)}</span><span>${fmtBytes(l.size)}</span></div>`)
              .join("")}</div>
            <button class="btn danger small" id="delLeft" style="margin-top:10px">${esc(t("deleteSelected"))} (${fmtBytes(
            lv.reduce((a, x) => a + x.size, 0)
          )})</button>`;
          const btn = $("#delLeft");
          if (btn)
            btn.onclick = async () => {
              const out = await call("delete_leftovers", { paths: lv.map((x) => x.path) }, t("cleaning"));
              deleteResultToast(out);
              box.innerHTML = "";
            };
        },
      },
      {
        label: t("removeEntry"),
        cls: "ghost",
        onClick: async () => {
          if (!(await confirmBox(t("confirmDelete"), t("removeEntry")))) return;
          try {
            await call("program_remove_entry", { key });
            state.progs = state.progs.filter((x) => x.key !== key);
            state.progSel.delete(key);
            toast(t("done"));
            go("uninstall");
          } catch (e) { toast(String(e), "error"); }
        },
      },
      {
        label: t("uninstall"),
        cls: "primary",
        onClick: async () => {
          try {
            const out = await runUninstall(key, p.name);
            if (out.ok) {
              state.progs = state.progs.filter((x) => x.key !== key);
              state.progSel.delete(key);
              go("uninstall");
            }
          } catch (e) { /* reported by call() */ }
        },
      },
    ],
  });
  void m;
}
async function renderUninstall() {
  state.progs = await call("programs_list");
  const q = state.progQuery.toLowerCase().trim();
  let list = state.progs.filter(
    (p) => !q || [p.name, p.publisher, p.location].some((v) => String(v || "").toLowerCase().includes(q))
  );
  const by = state.progSort;
  list = list.slice().sort((a, b) => {
    if (by === "size") return progSizeKb(b) - progSizeKb(a);
    if (by === "date") return String(b.installDate || "").localeCompare(String(a.installDate || ""));
    if (by === "publisher") return String(a.publisher || "").localeCompare(String(b.publisher || ""));
    return String(a.name || "").localeCompare(String(b.name || ""));
  });
  const totalSize = state.progs.reduce((a, p) => a + progSizeKb(p) * 1024, 0);
  const selCount = list.filter((p) => state.progSel.has(p.key)).length;
  const view = state.progView;

  const checkbox = (p) => `<input type="checkbox" class="prog-check" data-sel="${esc(p.key)}" ${state.progSel.has(p.key) ? "checked" : ""} />`;
  const ctxAttrs = (p) => `data-progopen="${esc(p.key)}" data-progctx="${esc(p.key)}"`;

  let bodyHtml = "";
  if (view === "grid") {
    bodyHtml = list.length
      ? `<div class="prog-grid">${list.map((p) => `<div class="prog-card ${state.progSel.has(p.key) ? "selected" : ""}" ${ctxAttrs(p)}>
          <label class="prog-card-check" onclick="event.stopPropagation()">${checkbox(p)}</label>
          ${progIconWrap(p, 44, "grid")}
          <div class="prog-card-name ellipsis" title="${esc(p.name)}">${esc(p.name)}</div>
          <div class="tiny muted ellipsis" style="max-width:100%">${esc(p.publisher || "")}</div>
          <div class="tiny muted">${progSizeKb(p) ? fmtBytes(progSizeKb(p) * 1024) : "—"}</div>
          <button class="btn small primary prog-card-btn" data-progopen="${esc(p.key)}">${esc(t("uninstall"))}</button>
        </div>`).join("")}</div>`
      : `<div class="empty-state">${esc(t("nothingFound"))}</div>`;
  } else {
    const detail = view === "details";
    const head = `<tr><th style="width:34px"></th><th>${esc(t("name"))}</th>${detail ? `<th>${esc(t("publisher"))}</th><th style="direction:ltr">Path</th>` : ""}<th>${esc(t("version"))}</th><th>${esc(t("size"))}</th><th>${esc(t("installedOn"))}</th><th></th></tr>`;
    const rows = list
      .map((p) => `<tr ${ctxAttrs(p)} class="${state.progSel.has(p.key) ? "selected" : ""}">
        <td>${checkbox(p)}</td>
        <td><div class="prog-name">${progIconWrap(p, 26, "list")}<div class="grow min0"><div class="prog-title ellipsis">${esc(p.name)}</div><div class="tiny muted ellipsis">${esc(p.publisher || "")}</div></div></div></td>
        ${detail ? `<td class="tiny muted ellipsis" style="max-width:180px">${esc(p.publisher || "—")}</td><td class="tiny muted ellipsis" style="max-width:240px;direction:ltr;text-align:start">${esc(p.location || "—")}</td>` : ""}
        <td class="num">${esc(p.version || "—")}</td>
        <td class="num">${progSizeKb(p) ? fmtBytes(progSizeKb(p) * 1024) : "—"}</td>
        <td class="num">${esc(p.installDate || "—")}</td>
        <td style="text-align:end"><button class="btn small" data-progopen="${esc(p.key)}">${esc(t("uninstall"))}</button></td>
      </tr>`)
      .join("");
    bodyHtml = `<div class="card flat"><table class="table prog-table"><thead>${head}</thead><tbody>${rows ||
      `<tr><td colspan="${detail ? 8 : 7}" class="muted">${esc(t("nothingFound"))}</td></tr>`}</tbody></table></div>`;
  }

  const viewBtns = [["list", "listview", "viewList"], ["grid", "gridview", "viewGrid"], ["details", "detailview", "viewDetails"]]
    .map(([v, ic, label]) => `<button class="btn small ${view === v ? "active" : ""}" data-view="${v}" title="${esc(t(label))}">${svg(ic, 14)}</button>`)
    .join("");
  const sortOpts = [["name", "sortName"], ["size", "sortSize"], ["date", "sortDate"], ["publisher", "sortPublisher"]]
    .map(([v, label]) => `<option value="${v}" ${state.progSort === v ? "selected" : ""}>${esc(t(label))}</option>`)
    .join("");

  $("#view").innerHTML =
    pageHead(t("programs"), `${state.progs.length} ${esc(state.lang === "ar" ? "برنامج" : "programs")} • ${fmtBytes(totalSize)}`, "") +
    `<div class="toolbar uninst-toolbar">
      <div class="search">${svg("search")}<input id="progSearch" placeholder="${esc(t("search"))}" value="${esc(state.progQuery)}"/></div>
      <div class="view-switch">${viewBtns}</div>
      <select id="progSort" class="btn small" title="${esc(t("sortBy"))}">${sortOpts}</select>
      <button class="btn small ghost" id="progSelAll">${esc(t("selectAll"))}</button>
      <button class="btn small ghost" id="progSelNone">${esc(t("clearAll"))}</button>
      <button class="btn small danger" id="progBatch" ${selCount ? "" : "disabled"}>${esc(t("batchUninstall"))}${selCount ? ` (${selCount})` : ""}</button>
      <button class="btn small" id="progLeftoverScan" title="${esc(t("leftoverScanHint"))}">${svg("search", 14)}${esc(t("leftoverScan"))}</button>
      <button class="btn small" id="progRefresh" title="${esc(t("scan"))}">${svg("refresh", 14)}</button>
    </div>
    <div class="tiny muted" style="margin:-8px 0 10px">${esc(t("progIconHint"))}</div>
    ${bodyHtml}
    <div class="card flat" style="margin-top:14px">
      <div class="card-title">${svg("trash")}${esc(t("forcedUninstall"))}</div>
      <div class="tiny muted" style="margin-bottom:10px">${esc(t("forcedUninstallHint"))}</div>
      <div class="field-row">
        <div class="field" style="flex:2"><input type="text" id="forcedName" placeholder="${esc(state.lang === "ar" ? "اسم البرنامج" : "Program name")}" /></div>
        <button class="btn danger" id="forcedBtn">${esc(t("forcedUninstallBtn"))}</button>
      </div>
      <div class="tiny muted" style="margin-top:8px">${esc(t("quarantineNote"))}</div>
      <div style="margin-top:8px;display:flex;gap:8px">
        <button class="btn small ghost" id="quarantineRestore">${esc(t("restoreQuarantine"))}</button>
        <button class="btn small ghost" id="quarantineEmpty">${esc(t("emptyQuarantine"))}</button>
      </div>
      <div style="margin-top:14px;border-top:1px solid var(--line);padding-top:12px">
        <div class="card-title" style="margin-bottom:6px">${svg("search")}${esc(t("hunterMode"))}</div>
        <div class="tiny muted" style="margin-bottom:8px">${esc(t("hunterModeHint"))}</div>
        <button class="btn blue" id="hunterBtn">${svg("search", 14)}${esc(t("captureTarget"))}</button>
        <div id="hunterResult" class="tiny" style="margin-top:8px"></div>
      </div>
    </div>`;

  const search = $("#progSearch");
  search.oninput = () => {
    state.progQuery = search.value;
    renderUninstall();
  };
  search.focus();
  search.setSelectionRange(search.value.length, search.value.length);

  $$("[data-view]").forEach((b) => (b.onclick = () => {
    state.progView = b.dataset.view;
    localStorage.setItem("vexora_prog_view", state.progView);
    renderUninstall();
  }));
  if ($("#progSort"))
    $("#progSort").onchange = (e) => {
      state.progSort = e.target.value;
      localStorage.setItem("vexora_prog_sort", state.progSort);
      renderUninstall();
    };
  const updateBatch = () => {
    const n = list.filter((p) => state.progSel.has(p.key)).length;
    const btn = $("#progBatch");
    if (btn) {
      btn.disabled = !n;
      btn.textContent = `${t("batchUninstall")}${n ? ` (${n})` : ""}`;
    }
  };
  if ($("#progSelAll"))
    $("#progSelAll").onclick = () => {
      list.forEach((p) => state.progSel.add(p.key));
      $$("[data-sel]").forEach((c) => { c.checked = true; c.closest("tr, .prog-card")?.classList.add("selected"); });
      updateBatch();
    };
  if ($("#progSelNone"))
    $("#progSelNone").onclick = () => {
      list.forEach((p) => state.progSel.delete(p.key));
      $$("[data-sel]").forEach((c) => { c.checked = false; c.closest("tr, .prog-card")?.classList.remove("selected"); });
      updateBatch();
    };
  if ($("#progBatch"))
    $("#progBatch").onclick = async () => {
      const keys = [...state.progSel];
      if (!keys.length) return toast(t("nothingSelected"), "warn");
      if (!(await confirmBox(`${t("batchHint")}\n${keys.length} ${t("programs")}`, t("batchUninstall")))) return;
      const allLeft = [];
      for (const key of keys) {
        const p = state.progs.find((x) => x.key === key);
        if (!p) continue;
        try {
          const out = await call("program_uninstall", { key, name: p.name }, `${t("uninstalling")} — ${p.name}`);
          if (out.ok) allLeft.push(...(out.leftovers || []));
          else toast(`${p.name}: ${t("uninstall_" + out.message)}`, "error");
        } catch (e) { /* call() already reported it */ }
      }
      // One review dialog for everything that was left behind.
      if (allLeft.length) await reviewLeftovers(allLeft);
      state.progSel.clear();
      state.progs = [];
      go("uninstall");
    };
  $$("[data-sel]").forEach((c) => (c.onchange = () => {
    const key = c.dataset.sel;
    if (c.checked) state.progSel.add(key); else state.progSel.delete(key);
    c.closest("tr, .prog-card")?.classList.toggle("selected", c.checked);
    updateBatch();
  }));
  $$("[data-progctx]").forEach((el) => (el.oncontextmenu = (e) => {
    e.preventDefault();
    openProgMenu(el.dataset.progctx, e.clientX, e.clientY);
  }));
  $$("[data-progopen]").forEach((b) =>
    b.addEventListener("click", (e) => {
      e.stopPropagation();
      showProgDetails(b.dataset.progopen);
    })
  );
  $("#progRefresh").onclick = () => { state.progs = []; go("uninstall"); };
  if ($("#progLeftoverScan"))
    $("#progLeftoverScan").onclick = async () => {
      const plan = await call("leftover_scan", {}, t("scanning"));
      if (!plan.items || !plan.items.length) return toast(t("nothingFound"), "warn");
      // Mandatory review: nothing is deleted before the user confirms.
      const paths = await showCleanupPlan(t("leftoverScan"), plan, t("deleteSelected"));
      if (!paths || !paths.length) return;
      const out = await call("delete_leftovers", { paths }, t("cleaning"));
      toast(`${t("done")} (${out.removed})${out.failed ? ` • ${t("failedItems")}: ${out.failed}` : ""}`);
    };
  if ($("#forcedBtn"))
    $("#forcedBtn").onclick = async () => {
      const name = ($("#forcedName").value || "").trim();
      if (!name) return toast(t("nothingSelected"), "warn");
      const plan = await call("forced_uninstall_plan", { name }, t("scanning"));
      const paths = await showCleanupPlan(t("forcedUninstall"), plan, t("forcedUninstallBtn"));
      if (!paths || !paths.length) return;
      const n = await call("forced_uninstall_selected", { name, paths }, t("cleaning"));
      toast(`${t("done")} (${n}) — ${t("quarantineNote")}`);
      state.progs = [];
      go("uninstall");
    };
  if ($("#quarantineRestore"))
    $("#quarantineRestore").onclick = async () => {
      const n = await call("quarantine_restore", {}, t("cleaning"));
      toast(`${t("done")} (${n})`);
    };
  if ($("#quarantineEmpty"))
    $("#quarantineEmpty").onclick = async () => {
      if (!(await confirmBox(t("emptyQuarantine"), t("delete")))) return;
      const n = await call("quarantine_empty", {}, t("cleaning"));
      toast(`${t("done")} (${n})`);
    };
  if ($("#hunterBtn"))
    $("#hunterBtn").onclick = async () => {
      $("#hunterResult").innerHTML = `<div class="tiny muted">${esc(t("scanning"))}</div>`;
      await new Promise((r) => setTimeout(r, 3000));
      const info = await call("hunter_capture");
      const res = $("#hunterResult");
      if (!info || !info.process) {
        res.innerHTML = `<div class="tiny muted">${esc(t("noMatch"))}</div>`;
        return;
      }
      const match = state.progs.find((p) => (p.name || "").toLowerCase().includes((info.process || "").toLowerCase().replace(".exe", "")));
      res.innerHTML = `<div>${esc(info.process)} • ${esc(info.title)}</div>
        <div class="tiny muted" style="direction:ltr;text-align:start">${esc(info.path)}</div>
        ${match ? `<button class="btn danger small" id="hunterUninstall" style="margin-top:8px">${esc(t("uninstall"))} ${esc(match.name)}</button>` : `<div class="tiny muted" style="margin-top:6px">${esc(t("noMatch"))}</div>`}`;
      if (match && $("#hunterUninstall"))
        $("#hunterUninstall").onclick = async () => {
          if (!(await confirmBox(match.name, t("uninstall")))) return;
          try {
            const out = await runUninstall(match.key, match.name);
            state.progSel.delete(match.key);
            if (out.ok) state.progs = state.progs.filter((z) => z.key !== match.key);
          } catch (e) { /* reported by call() */ }
          go("uninstall");
        };
    };

  applyProgIcons();
  loadProgIcons(list);
}

/* --------------------------------------------------------------------- TOOLS */
async function renderTools() {
  if (!state.tools) state.tools = { tab: "dupes", root: "C:\\", results: {}, browserCats: ["cache", "cookies", "history", "sessions"], junk: null, services: null, tasks: null, monitorName: "", monitorActive: false, monitorLogs: null, similar: null, wipe: null };
  const tab = state.tools.tab;
  const tabs = [
    ["dupes", t("dupes"), "apps"],
    ["big", t("bigFiles"), "file"],
    ["empty", t("emptyDirs"), "folder"],
    ["map", t("diskMap"), "disk"],
    ["drives", t("drives"), "disk"],
    ["sysrepair", t("sysRepair"), "shield"],
    ["spooler", t("spooler"), "tools"],
    ["contextmenu", t("contextMenu"), "settings"],
    ["ramopt", t("ramOptimizer"), "zap"],
    ["hwmon", t("hwmon"), "shield"],
    ["battery", t("battery"), "shield"],
    ["shredder", t("shredder"), "shield"],
    ["fonts", t("fonts"), "file"],
    ["adobecache", t("adobeCache"), "folder"],
    ["runtimes", t("runtimes"), "shield"],
    ["wipe", t("wipe"), "shield"],
    ["browsers", t("browsers"), "browser"],
    ["junk", t("junkFiles"), "tools"],
    ["services", t("services"), "settings"],
    ["monitor", t("monitor"), "search"],
  ];

  let body = "";
  const r = state.tools.results || {};

  if (tab === "dupes") {
    const res = r.dupes;
    const roots = state.tools.roots && state.tools.roots.length ? state.tools.roots : [state.tools.root || "C:\\"];
    body = `<div class="card flat">
      <div class="field"><label>${esc(t("chooseFolder"))} — ${esc(state.lang === "ar" ? "مجلد واحد في كل سطر" : "one folder per line")}</label>
        <textarea id="dupRoots" rows="3" style="width:100%;background:var(--card);border:1px solid var(--line);border-radius:8px;color:var(--text);padding:8px;direction:ltr;text-align:start">${esc(roots.join("\n"))}</textarea></div>
      <div class="field-row" style="margin-top:8px">
        <button class="btn ghost" data-pick-roots="dupes">${svg("folder")}${esc(t("chooseFolder"))}</button>
      </div>
      <div class="field-row" style="margin-top:12px">
        <div class="field"><label>${esc(t("minSize"))} (KB)</label><input type="number" id="dupMin" value="${res ? res._min : 64}" /></div>
        <button class="btn primary" id="dupScan">${svg("search")}${esc(t("scan"))}</button>
        ${state.tools.scanning ? `<button class="btn danger" id="dupStop">${svg("x")}${esc(t("stop"))}</button>` : ""}
      </div>
      ${state.tools.scanning ? `<div id="dupProg" class="tiny muted" style="margin-top:10px">${state.tools.dupScanned || 0} ${esc(state.lang === "ar" ? "ملف تم فحصه…" : "files scanned…")}</div>` : ""}
      ${
        res
          ? `<div style="margin-top:14px" class="tiny muted">${res.groups.length} ${esc(t("groups"))} · ${esc(t("reclaim"))} ${fmtBytes(res.reclaimable)}</div>
             <div id="dupList">${res.groups
               .slice(0, 200)
               .map(
                 (g, gi) => `<div class="card flat" style="margin-top:8px;padding:12px">
                  <div class="tiny muted" style="margin-bottom:6px">${fmtBytes(g.size)} × ${g.files.length}</div>
                  ${g.files
                    .map(
                      (f, fi) => `<label class="row" style="padding:6px 0;border:0;gap:8px;cursor:pointer">
                        <input type="checkbox" data-dup="${gi}:${fi}" ${fi > 0 ? "checked" : ""} style="accent-color:var(--accent)" />
                        <span class="grow ellipsis tiny" style="direction:ltr;text-align:start">${esc(f)}</span>
                      </label>`
                    )
                    .join("")}
                </div>`
               )
               .join("")}</div>
             <button class="btn danger" id="dupDelete" style="margin-top:12px">${esc(t("deleteSelected"))}</button>`
          : ""
      }
    </div>`;
  } else if (tab === "big") {
    const res = r.big;
    body = `<div class="card flat">
      ${pathPicker("big")}
      <div class="field-row" style="margin-top:12px">
        <div class="field"><label>${esc(t("minSize"))} (MB)</label><input type="number" id="bigMin" value="${res ? res._min : 100}" /></div>
        <button class="btn primary" id="bigScan">${svg("search")}${esc(t("scan"))}</button>
      </div>
      ${
        res
          ? `<table class="table" style="margin-top:12px"><thead><tr><th>${esc(t("size"))}</th><th>${esc(t("path"))}</th><th></th></tr></thead><tbody>
             ${res.files
               .map(
                 (f) => `<tr><td class="num">${fmtBytes(f.size)}</td>
                 <td class="tiny ellipsis" style="direction:ltr;text-align:start;max-width:460px">${esc(f.path)}</td>
                 <td style="text-align:end"><button class="btn small ghost" data-reveal="${esc(f.path)}">${esc(t("reveal"))}</button></td></tr>`
               )
               .join("")}
             </tbody></table>`
          : ""
      }
    </div>`;
  } else if (tab === "empty") {
    const res = r.empty;
    body = `<div class="card flat">
      ${pathPicker("empty")}
      <div class="field-row" style="margin-top:12px">
        <button class="btn primary" id="emptyScan">${svg("search")}${esc(t("scan"))}</button>
      </div>
      ${
        res && res.dirs.length
          ? `<div class="row" style="margin-top:12px;gap:8px;align-items:center">
               <div class="tiny muted">${res.dirs.length} ${esc(t("emptyDirs"))}</div>
               <span class="spacer"></span>
               <button class="btn small ghost" id="emptySelAll">${esc(t("selectAll"))}</button>
               <button class="btn small ghost" id="emptySelNone">${esc(t("selectNone"))}</button>
             </div>
             <div class="filelist" style="margin-top:8px">${res.dirs
               .map((d, i) => `<label class="f" style="cursor:pointer"><input type="checkbox" data-empty="${i}" checked style="accent-color:var(--accent);margin-inline-end:8px" /><span style="direction:ltr;text-align:start">${esc(d)}</span></label>`)
               .join("")}</div>
             <button class="btn danger" id="emptyDelete" style="margin-top:10px">${esc(t("deleteSelected"))}</button>`
          : res
          ? `<div class="tiny muted" style="margin-top:12px">${esc(t("nothingFound"))}</div>`
          : ""
      }
    </div>`;
  } else if (tab === "map") {
    const res = r.map;
    body = `<div class="card flat">
      ${pathPicker("map")}
      <div class="field-row" style="margin-top:12px">
        <button class="btn primary" id="mapScan">${svg("search")}${esc(t("scan"))}</button>
      </div>
      ${
        res
          ? `<div style="margin-top:14px" class="tiny muted">${esc(res.path)} · ${fmtBytes(res.size)}</div>
             <div class="bars" style="margin-top:10px">${res.children
               .slice(0, 30)
               .map(
                 (c) => `<div class="bar-row">
                  <span class="nm" title="${esc(c.name)}">${esc(c.name)}</span>
                  <span class="bar-track"><i style="width:${((c.size / Math.max(1, res.size)) * 100).toFixed(1)}%"></i></span>
                  <span class="vl">${fmtBytes(c.size)}</span>
                </div>`
               )
               .join("")}</div>`
          : ""
      }
    </div>`;
  } else if (tab === "browsers") {
    const bl = await call("browsers_list", {}, t("scanning"));
    const cats = state.tools.browserCats || ["cache", "cookies", "history", "sessions"];
    const catLabels = { cache: t("browserCache"), cookies: t("browserCookies"), history: t("browserHistory"), sessions: t("browserSessions"), siteData: t("browserSiteData") };
    body = `<div class="card flat">
      <div class="tiny muted" style="margin-bottom:10px">${esc(t("browserClean"))}</div>
      <div style="display:flex;gap:8px;flex-wrap:wrap;margin-bottom:12px">
        ${Object.keys(catLabels).map((c) => `<label class="chip ${cats.includes(c) ? "ok" : ""}" style="cursor:pointer"><input type="checkbox" data-bcat="${c}" ${cats.includes(c) ? "checked" : ""} style="accent-color:var(--accent)" /> ${esc(catLabels[c])}</label>`).join("")}
      </div>
      <div id="browserList">
        ${bl.map((b) => `<div class="row" style="padding:8px 0">
          <span class="grow">${esc(b.name)}</span>
          <span class="tiny muted">${b.installed ? (b.paths.length + " profile(s)") : "—"}</span>
          <button class="btn small ${b.installed ? "primary" : "ghost"}" data-bclean="${b.id}" ${b.installed ? "" : "disabled"}>${svg("broom", 14)}${esc(t("clean"))}</button>
        </div>`).join("")}
      </div>
      <div style="margin-top:12px">
        <button class="btn danger" id="browserCleanAll">${svg("broom")}${esc(t("browserCleanAll"))}</button>
      </div>
    </div>`;
  } else if (tab === "monitor") {
    const logs = state.tools.monitorLogs;
    let logsHtml = "";
    if (logs && logs.length) {
      logsHtml = logs.map((l) => `<div class="row" style="padding:8px 0">
        <span class="grow">${esc(l.name)}</span>
        <span class="tiny muted">${new Date(l.ts * 1000).toLocaleString()} · ${l.changes.length} changes</span>
        <button class="btn small danger" data-monuninst="${esc(l.name)}">${esc(t("monitorUninstall"))}</button>
      </div>`).join("");
    } else {
      logsHtml = `<div class="tiny muted">${esc(t("monitorNoLogs"))}</div>`;
    }
    body = `<div class="card flat">
      <div class="card-title">${svg("search")}${esc(t("monitor"))}</div>
      <div class="tiny muted" style="margin-bottom:12px">${esc(t("monitorHint"))}</div>
      <div class="field-row" style="margin-bottom:12px">
        <div class="field" style="flex:2"><input type="text" id="monName" placeholder="${esc(state.lang === "ar" ? "اسم البرنامج" : "Program name")}" value="${esc(state.tools.monitorName || "")}" /></div>
        <button class="btn primary" id="monStart">${esc(t("monitorStart"))}</button>
        <button class="btn danger" id="monFinish">${esc(t("monitorFinish"))}</button>
      </div>
      <div id="monStatus" class="tiny muted" style="margin-bottom:12px">${state.tools.monitorActive ? "Monitoring: " + esc(state.tools.monitorName) : ""}</div>
      <div style="border-top:1px solid var(--line);padding-top:12px">
        <div class="card-title" style="margin-bottom:8px">${esc(t("monitor"))} — logs</div>
        ${logsHtml}
      </div>
    </div>`;
  } else if (tab === "services") {
    const svcs = state.tools.services || [];
    const tasks = state.tools.tasks || [];
    if (!state.tools.services) {
      // load
    }
    body = `<div class="card flat">
      <div class="card-title">${svg("settings")}${esc(t("services"))}</div>
      <div class="field-row" style="margin-bottom:12px">
        <button class="btn primary" id="svcRefresh">${svg("refresh")}${esc(t("scan"))}</button>
      </div>
      <h3 style="margin:10px 0 6px;font-size:13px">${esc(t("serviceStatus")) === t("serviceStatus") ? "Services" : t("serviceStatus")}</h3>
      <div style="max-height:260px;overflow:auto"><table class="table">
        <thead><tr><th>${esc(t("serviceName"))}</th><th>${esc(t("serviceDisplay"))}</th><th>${esc(t("serviceStatus"))}</th><th>${esc(t("serviceStartup"))}</th><th></th></tr></thead>
        <tbody>${svcs.map((s) => `<tr>
          <td class="tiny" style="direction:ltr;text-align:start">${esc(s.name)}</td>
          <td class="tiny">${esc(s.display)}</td>
          <td class="tiny">${esc(s.status)}</td>
          <td class="tiny">${esc(s.startType)}</td>
          <td style="text-align:end">
            ${s.startType === "Disabled"
              ? `<button class="btn small" data-svcon="${esc(s.name)}">${esc(t("serviceEnable"))}</button>`
              : `<button class="btn small ghost" data-svcoff="${esc(s.name)}">${esc(t("serviceDisable"))}</button>`}
          </td>
        </tr>`).join("") || `<tr><td colspan="5" class="muted">${esc(t("nothingFound"))}</td></tr>`}
        </tbody></table></div>
      <h3 style="margin:14px 0 6px;font-size:13px">Scheduled Tasks</h3>
      <div style="max-height:260px;overflow:auto"><table class="table">
        <thead><tr><th>${esc(t("taskName"))}</th><th>${esc(t("taskStatus"))}</th><th>${esc(t("taskNextRun"))}</th><th></th></tr></thead>
        <tbody>${tasks.map((tk) => `<tr>
          <td class="tiny" style="direction:ltr;text-align:start">${esc(tk.name)}</td>
          <td class="tiny">${esc(tk.status)}</td>
          <td class="tiny">${esc(tk.nextRun)}</td>
          <td style="text-align:end">
            ${tk.status.toLowerCase().includes("disabled")
              ? `<button class="btn small" data-taskon="${esc(tk.name)}">${esc(t("serviceEnable"))}</button>`
              : `<button class="btn small ghost" data-taskoff="${esc(tk.name)}">${esc(t("serviceDisable"))}</button>`}
          </td>
        </tr>`).join("") || `<tr><td colspan="4" class="muted">${esc(t("nothingFound"))}</td></tr>`}
        </tbody></table></div>
    </div>`;
  } else if (tab === "junk") {
    const res = state.tools.junk;
    body = `<div class="card flat">
      <div class="field-row" style="margin-bottom:12px">
        <button class="btn primary" id="junkScan">${svg("search")}${esc(t("junkScan"))}</button>
      </div>
      ${
        res
          ? `<div class="tiny muted" style="margin-bottom:8px">${res.items.length} ${esc(t("junkFiles"))} · ${esc(t("junkTotal"))} ${fmtBytes(res.totalSize)}</div>
             <div class="filelist" style="max-height:360px;overflow:auto">${res.items.map((j, i) => `<label class="f" style="gap:8px;cursor:pointer"><input type="checkbox" data-junk="${i}" checked style="accent-color:var(--accent)" /><span class="grow ellipsis tiny" style="direction:ltr;text-align:start">${esc(j.path)}</span><span>${fmtBytes(j.size)}</span></label>`).join("")}</div>
             <button class="btn danger" id="junkDelete" style="margin-top:10px">${esc(t("junkDelete"))}</button>`
          : ""
      }
    </div>`;
  } else if (tab === "drives") {
    if (state.tools.drives === undefined) {
      state.tools.drives = [];
      call("drives_list", {})
        .then((l) => {
          state.tools.drives = l;
          if (state.page === "tools" && state.tools.tab === "drives") go("tools");
        })
        .catch(() => {});
    }
    const drives = state.tools.drives;
    const chk = state.tools.driveCheck;
    const typeLabel = (d) =>
      d.driveType === "Removable" ? t("removable") : d.driveType === "Fixed" ? t("fixedDisk") : d.driveType === "Cloud" ? t("cloudDisk") : d.driveType || "—";
    const healthLabel = (d) => d.diskHealth || d.health || "—";
    const healthPill = (d) => {
      const h = (d.diskHealth || d.health || "").toLowerCase();
      return h.includes("healthy") ? "pill ok" : h.includes("warn") || h.includes("unhealthy") ? "pill bad" : "pill";
    };
    const statusLabel = (s) => {
      const map = {
        clean: t("driveStatusClean"),
        errors: t("driveStatusErrors"),
        cleaned: t("driveStatusCleaned"),
        unavailable: t("driveStatusUnavailable"),
        unknown: t("driveStatusUnknown"),
      };
      return map[s] || t("driveStatusUnknown");
    };
    body = `<div class="card flat">
      ${!(state.info && state.info.admin) ? `<div class="card flat" style="margin-bottom:12px;padding:10px;display:flex;align-items:center;justify-content:space-between;gap:10px">
        <div class="tiny muted" style="flex:1">${esc(t("driveAdminHint"))}</div>
        <button class="btn small blue" id="driveElevate">${svg("shield", 14)}${esc(t("relaunchAdmin"))}</button>
      </div>` : ""}
      <div class="field-row" style="margin-bottom:12px">
        <button class="btn primary" id="drivesScan">${svg("search")}${esc(t("scan"))}</button>
      </div>
      ${
        drives && drives.length
          ? `<table class="table"><thead><tr><th>${esc(t("disk"))}</th><th>${esc(t("driveType"))}</th><th>${esc(t("driveTotal"))}</th><th>${esc(t("driveFree"))}</th><th>${esc(t("driveHealth"))}</th><th></th></tr></thead><tbody>
             ${drives
               .map(
                 (d) => `<tr>
               <td><div style="font-weight:600">${esc(d.letter)} ${d.label ? esc(d.label) : ""}</div>
                 <div class="tiny muted" style="direction:ltr;text-align:start">${esc(d.model || "")}${d.bus ? " · " + esc(d.bus) : ""}</div>
                 <div class="tiny muted" style="direction:ltr;text-align:start">${d.temp >= 0 ? esc(t("driveTemp")) + " " + d.temp + "°C" : ""}${d.wear >= 0 ? " · " + esc(t("driveWear")) + " " + d.wear + "%" : ""}${d.powerOn >= 0 ? " · " + esc(t("drivePowerOn")) + " " + d.powerOn : ""}</div></td>
               <td><span class="pill">${esc(typeLabel(d))}</span></td>
               <td class="num">${fmtBytes(d.total)}</td>
               <td class="num">${fmtBytes(d.free)}</td>
               <td><span class="${healthPill(d)}">${esc(healthLabel(d))}</span></td>
               <td style="text-align:end">${d.driveType === "Cloud" ? `<span class="tiny muted">—</span>` : `<button class="btn small ghost" data-drivecheck="${esc(d.letter)}">${esc(t("driveScan"))}</button>`}</td>
             </tr>`
               )
               .join("")}
             </tbody></table>`
          : drives && drives.length === 0
          ? `<div class="tiny muted">${esc(t("nothingFound"))}</div>`
          : `<div class="tiny muted">${esc(t("scanning"))}</div>`
      }
      ${
        chk
          ? `<div class="card flat" style="margin-top:12px;padding:10px">
             <div class="tiny" style="font-weight:600;margin-bottom:6px">${esc(chk.letter)} — ${esc(statusLabel(chk.status))}</div>
             ${chk.raw ? `<details class="tiny" style="margin-top:6px"><summary style="cursor:pointer">${esc(t("driveRaw"))}</summary><pre style="white-space:pre-wrap;direction:ltr;text-align:start;max-height:220px;overflow:auto;margin:8px 0 0">${esc(chk.raw)}</pre></details>` : ""}
           </div>`
          : ""
      }
    </div>`;
  } else if (tab === "sysrepair") {
    if (!state.tools.repair) state.tools.repair = { running: false, stage: "", percent: -1, log: "", result: null };
    const rep = state.tools.repair;
    const stageLabel = (s) => ({ dismCheck: t("repairStageCheck"), dismRestore: t("repairStageRestore"), sfc: t("repairStageSfc") }[s] || s);
    const resLabel = (r) => (r === "clean" ? t("repairClean") : r === "errors" ? t("repairErrors") : r === "cancelled" ? t("repairCancelled") : r || "");
    body = `<div class="card flat">
      ${!(state.info && state.info.admin) ? `<div class="card flat" style="margin-bottom:12px;padding:10px;display:flex;align-items:center;justify-content:space-between;gap:10px">
        <div class="tiny muted" style="flex:1">${esc(t("repairNeedsAdmin"))}</div>
        <button class="btn small blue" id="repairElevate">${svg("shield", 14)}${esc(t("relaunchAdmin"))}</button>
      </div>` : ""}
      <div class="field-row" style="margin-bottom:12px">
        <button class="btn primary" id="repairStart" ${rep.running ? "disabled" : ""}>${svg("shield")}${esc(t("repairStart"))}</button>
        ${rep.running ? `<button class="btn danger" id="repairCancel">${svg("x")}${esc(t("repairCancel"))}</button>` : ""}
      </div>
      ${rep.running || rep.result ? `<div class="tiny muted" style="margin-bottom:6px">${esc(stageLabel(rep.stage))} ${rep.percent >= 0 ? rep.percent + "%" : ""}</div>` : ""}
      ${rep.running ? `<div class="progress" style="margin-bottom:12px"><i style="width:${Math.max(0, rep.percent)}%"></i></div>` : ""}
      ${rep.result ? `<div class="tiny" style="font-weight:600;margin-bottom:8px">${esc(resLabel(rep.result))}</div>` : ""}
      ${rep.log ? `<details ${rep.running ? "" : "open"} class="tiny" style="margin-top:8px"><summary style="cursor:pointer">${esc(t("repairLog"))}</summary><pre style="white-space:pre-wrap;direction:ltr;text-align:start;max-height:260px;overflow:auto;margin:8px 0 0">${esc((rep.log || "").slice(-8000))}</pre></details>` : ""}
    </div>`;
  } else if (tab === "spooler") {
    if (state.tools.spoolerPending === undefined) {
      state.tools.spoolerPending = null;
      call("spooler_status", {}).then((n) => { state.tools.spoolerPending = n; if (state.page === "tools" && state.tools.tab === "spooler") go("tools"); }).catch(() => {});
    }
    const pending = state.tools.spoolerPending;
    const res = state.tools.spoolerResult;
    body = `<div class="card flat">
      ${!(state.info && state.info.admin) ? `<div class="card flat" style="margin-bottom:12px;padding:10px;display:flex;align-items:center;justify-content:space-between;gap:10px">
        <div class="tiny muted" style="flex:1">${esc(t("repairNeedsAdmin"))}</div>
        <button class="btn small blue" id="spoolerElevate">${svg("shield", 14)}${esc(t("relaunchAdmin"))}</button>
      </div>` : ""}
      <div class="field-row" style="margin-bottom:12px;align-items:center">
        <div class="tiny muted">${esc(t("spoolerPending"))}: <b>${pending == null ? "—" : pending}</b></div>
        <button class="btn ghost small" id="spoolerRefresh">${svg("search", 14)}${esc(t("scan"))}</button>
      </div>
      <button class="btn danger" id="spoolerFix">${svg("tools")}${esc(t("spoolerFix"))}</button>
      ${res ? `<div class="tiny" style="font-weight:600;margin-top:10px">${esc(res.ok ? t("spoolerFixed") : t("spoolerFixFailed"))}${res.msg ? " · " + esc(res.msg) : ""}</div>` : ""}
    </div>`;
  } else if (tab === "contextmenu") {
    if (state.tools.ctxItems === undefined) {
      state.tools.ctxItems = null;
      state.tools.ctxClassic = null;
      call("ctxmenu_list", {}).then((l) => { state.tools.ctxItems = l; if (state.page === "tools" && state.tools.tab === "contextmenu") go("tools"); }).catch(() => {});
      call("ctxmenu_win11_status", {}).then((v) => { state.tools.ctxClassic = v; if (state.page === "tools" && state.tools.tab === "contextmenu") go("tools"); }).catch(() => {});
    }
    const items = state.tools.ctxItems;
    const classic = state.tools.ctxClassic;
    const scopeLabel = (s) => (s.startsWith("*") ? t("ctxScopeFile") : s.startsWith("Directory\\Background") ? t("ctxScopeBackground") : t("ctxScopeFolder"));
    body = `<div class="card flat">
      <div class="setting" style="margin-bottom:12px">
        <div class="txt"><div class="t">${esc(t("ctxWin11Classic"))}</div><div class="s">${esc(t("ctxWin11ClassicHint"))}</div></div>
        <div class="toggle ${classic ? "on" : ""}" id="ctxClassicToggle"></div>
      </div>
      ${items
        ? `<table class="table"><thead><tr><th>${esc(t("ctxScope"))}</th><th>${esc(t("ctxName"))}</th><th>${esc(t("ctxFriendly"))}</th><th></th></tr></thead><tbody>
           ${items.map((it, i) => `<tr>
             <td><span class="pill">${esc(scopeLabel(it.scope))}</span></td>
             <td class="tiny" style="direction:ltr;text-align:start">${esc(it.name)}</td>
             <td class="tiny">${esc(it.friendly || it.clsid)}</td>
             <td style="text-align:end"><button class="btn small ${it.enabled ? "ghost" : "primary"}" data-ctx="${i}">${esc(it.enabled ? t("ctxDisable") : t("ctxEnable"))}</button></td>
           </tr>`).join("")}
           </tbody></table>`
        : `<div class="tiny muted">${esc(t("scanning"))}</div>`}
    </div>`;
  } else if (tab === "adobecache") {
    if (state.tools.adobeItems === undefined) {
      state.tools.adobeItems = null;
      state.tools.adobeRunning = null;
      call("adobe_running", {}).then((v) => { state.tools.adobeRunning = v; if (state.page === "tools" && state.tools.tab === "adobecache") go("tools"); }).catch(() => {});
      call("adobe_cache_items", {}).then((l) => { state.tools.adobeItems = l; if (state.page === "tools" && state.tools.tab === "adobecache") go("tools"); }).catch(() => {});
    }
    const items = state.tools.adobeItems;
    const running = state.tools.adobeRunning;
    body = `<div class="card flat">
      ${running ? `<div class="card flat" style="margin-bottom:12px;padding:10px;background:rgba(245,158,11,.12);border-color:rgba(245,158,11,.4)"><div class="tiny" style="color:var(--warn)">${esc(t("adobeRunning"))}</div></div>` : ""}
      <div class="field-row" style="margin-bottom:12px">
        <button class="btn ghost small" id="adobeScan">${svg("search", 14)}${esc(t("adobeScan"))}</button>
      </div>
      ${items && items.length
        ? `<div class="filelist" style="max-height:360px;overflow:auto">${items.map((it, i) => `<label class="f" style="gap:8px;cursor:pointer"><input type="checkbox" data-adobe="${i}" checked style="accent-color:var(--accent)" /><span class="grow ellipsis tiny" style="direction:ltr;text-align:start">${esc(it.path)}</span><span class="tiny muted">${esc(it.kind)}</span><span>${fmtBytes(it.size)}</span></label>`).join("")}</div>
           <button class="btn danger" id="adobeClean" style="margin-top:10px">${esc(t("adobeClean"))}</button>`
        : items ? `<div class="tiny muted">${esc(t("nothingFound"))}</div>` : `<div class="tiny muted">${esc(t("scanning"))}</div>`}
    </div>`;
  } else if (tab === "hwmon") {
    if (state.tools.hwmon === undefined) {
      state.tools.hwmon = null;
      call("hwmon_status", {})
        .then((h) => { state.tools.hwmon = h; if (state.page === "tools" && state.tools.tab === "hwmon") go("tools"); })
        .catch(() => {});
    }
    const h = state.tools.hwmon;
    const rows = h && h.sensors && h.sensors.length
      ? h.sensors
          .map(
            (s) =>
              `<tr><td>${esc(s.label)}</td><td>${esc(s.metric)}</td>` +
              `<td class="num" style="color:${s.available ? "var(--text)" : "var(--muted)"}">${esc(s.value)}</td></tr>`
          )
          .join("")
      : "";
    body = `<div class="card flat">
      <div class="field-row" style="margin-bottom:12px">
        <button class="btn primary" id="hwRefresh">${svg("refresh")} ${esc(t("hwmonRefresh"))}</button>
      </div>
      ${h && h.gpuName ? `<div class="tiny muted" style="margin-bottom:10px">${esc(t("hwmonGpu"))}: ${esc(h.gpuName)}</div>` : ""}
      ${
        rows
          ? `<table class="table"><thead><tr><th>${esc(t("name"))}</th><th>${esc(t("hwmonMetric"))}</th><th>${esc(t("hwmonValue"))}</th></tr></thead><tbody>${rows}</tbody></table>`
          : `<div class="tiny muted">${esc(t("hwmonNoSensors"))}</div>`
      }
      ${h && h.sources && h.sources.length ? `<div class="tiny muted" style="margin-top:10px">${esc(t("hwmonSources"))}: ${esc(h.sources.join(", "))}</div>` : ""}
      ${h && h.note ? `<div class="tiny muted" style="margin-top:6px">${esc(h.note)}</div>` : ""}
    </div>`;
  } else if (tab === "runtimes") {
    if (state.tools.runtimes === undefined) {
      state.tools.runtimes = null;
      call("runtimes_status", {}).then((l) => { state.tools.runtimes = l; if (state.page === "tools" && state.tools.tab === "runtimes") go("tools"); }).catch(() => {});
    }
    const rts = state.tools.runtimes;
    const backupFolder = state.tools.driverBackupFolder || "";
    const backupRes = state.tools.driverBackupResult;
    body = `<div class="card flat">
      <div class="card-title" style="margin-bottom:8px">${esc(t("driverBackup"))}</div>
      <div class="field-row" style="margin-bottom:12px">
        <div class="field" style="flex:2"><input type="text" id="driverBackupFolder" value="${esc(backupFolder)}" placeholder="${esc(t("driverBackupFolder"))}" /></div>
        <button class="btn ghost" id="driverBackupPick">${svg("folder")}${esc(t("chooseFolder"))}</button>
        <button class="btn primary" id="driverBackupRun">${svg("shield")}${esc(t("driverBackupRun"))}</button>
      </div>
      ${backupRes ? `<div class="tiny" style="font-weight:600;margin-bottom:10px">${esc(backupRes.ok ? t("spoolerFixed") : t("spoolerFixFailed"))}${backupRes.msg ? " · " + esc((backupRes.msg || "").slice(0, 120)) : ""}</div>` : ""}
      <div class="card-title" style="margin-bottom:8px">${esc(t("runtimes"))}</div>
      ${rts
        ? `<table class="table"><thead><tr><th>${esc(t("ctxName"))}</th><th>${esc(t("runtimeVersion"))}</th><th>${esc(t("driveHealth"))}</th><th></th></tr></thead><tbody>
           ${rts.map((r, i) => `<tr>
             <td class="tiny">${esc(r.name)}</td>
             <td class="tiny">${esc(r.version || "—")}</td>
             <td><span class="pill ${r.installed ? "ok" : "bad"}">${esc(r.installed ? t("runtimeInstalled") : t("runtimeMissing"))}</span></td>
             <td style="text-align:end">${!r.installed && r.wingetId ? `<button class="btn small primary" data-rt="${i}">${esc(t("runtimeInstall"))}</button>` : ""}</td>
           </tr>`).join("")}
           </tbody></table>`
        : `<div class="tiny muted">${esc(t("scanning"))}</div>`}
    </div>`;
  } else if (tab === "ramopt") {
    if (state.tools.ramStatus === undefined) {
      state.tools.ramStatus = null;
      state.tools.ramResult = null;
      call("ram_status", {}).then((s) => { state.tools.ramStatus = s; if (state.page === "tools" && state.tools.tab === "ramopt") go("tools"); }).catch(() => {});
    }
    const st = state.tools.ramStatus;
    const res = state.tools.ramResult;
    body = `<div class="card flat">
      <div class="field-row" style="margin-bottom:12px;flex-wrap:wrap">
        <div class="kv"><span>${esc(t("ramTotal"))}</span><span class="num">${st ? fmtBytes(st.total) : "—"}</span></div>
        <div class="kv"><span>${esc(t("ramUsed"))}</span><span class="num">${st ? fmtBytes(st.used) : "—"}</span></div>
        <div class="kv"><span>${esc(t("ramAvailable"))}</span><span class="num">${st ? fmtBytes(st.available) : "—"}</span></div>
        <div class="kv"><span>Load</span><span class="num">${st ? st.load + "%" : "—"}</span></div>
      </div>
      <button class="btn primary" id="ramClean">${svg("zap")}${esc(t("ramClean"))}</button>
      ${res ? `<div class="tiny" style="font-weight:600;margin-top:10px">${esc(t("ramFreed"))}: ${fmtBytes(res.freed)} · ${res.trimmed} processes</div>` : ""}
      <div class="tiny muted" style="margin-top:10px">${esc(t("ramAutoClean"))} — ${esc(state.settings && state.settings.ramAutoCleanThreshold ? state.settings.ramAutoCleanThreshold + "%" : "off")}</div>
    </div>`;
  } else if (tab === "battery") {
    if (state.tools.batteryInfo === undefined) {
      state.tools.batteryInfo = null;
      call("battery_status", {}).then((b) => { state.tools.batteryInfo = b; if (state.page === "tools" && state.tools.tab === "battery") go("tools"); }).catch(() => {});
    }
    const b = state.tools.batteryInfo;
    const mon = state.settings || {};
    const upper = mon.batteryUpperThreshold || 95;
    const lower = mon.batteryLowerThreshold || 20;
    const enabled = !!mon.batteryMonitorEnabled;
    body = `<div class="card flat">
      ${!b || !b.present ? `<div class="tiny muted" style="padding:20px;text-align:center">${esc(t("batteryNoBattery"))}</div>` : `
      <div class="field-row" style="margin-bottom:12px;flex-wrap:wrap">
        <div class="kv"><span>${esc(t("batteryCharge"))}</span><span class="num">${b.chargePct}%</span></div>
        <div class="kv"><span>${esc(t("batteryHealth"))}</span><span class="num">${b.healthPct}%</span></div>
        <div class="kv"><span>${esc(t("batteryDesign"))}</span><span class="num">${b.designCapacity} mWh</span></div>
        <div class="kv"><span>${esc(t("batteryFull"))}</span><span class="num">${b.fullCapacity} mWh</span></div>
        <div class="kv"><span>${esc(t("batteryCycles"))}</span><span class="num">${b.cycleCount >= 0 ? b.cycleCount : "—"}</span></div>
        <div class="kv"><span>${esc(t("batteryRate"))}</span><span class="num">${b.discharging ? b.dischargeRateW + " W ↓" : b.charging ? b.chargeRateW + " W ↑" : "—"}</span></div>
        <div class="kv"><span>${esc(t("batteryTime"))}</span><span class="num">${b.timeRemainingH > 0 ? b.timeRemainingH + " h" : "—"}</span></div>
        <div class="kv"><span>${esc(t("batteryPresent"))}</span><span class="pill ${b.powerOnline ? "ok" : "warn"}">${esc(b.powerOnline ? t("batteryStatusAC") : b.charging ? t("batteryStatusCharging") : t("batteryStatusDischarging"))}</span></div>
      </div>`}
      <div class="card-title" style="margin-bottom:8px">${esc(t("batteryMonitor"))}</div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("batteryMonitor"))}</div></div><div class="toggle ${enabled ? "on" : ""}" id="batMonToggle"></div></div>
      <div class="field-row" style="margin-top:10px">
        <div class="field"><label>${esc(t("batteryUpper"))} (%)</label><input type="number" id="batUpper" min="50" max="100" value="${upper}" /></div>
        <div class="field"><label>${esc(t("batteryLower"))} (%)</label><input type="number" id="batLower" min="5" max="50" value="${lower}" /></div>
        <button class="btn primary" id="batApply">${svg("shield")}${esc(t("save"))}</button>
      </div>
      <div class="tiny muted" style="margin-top:10px">${esc(t("bgMonitorHint"))}</div>
    </div>`;
  } else if (tab === "shredder") {
    const path = state.tools.shredPath || "";
    const locks = state.tools.shredLocks;
    const res = state.tools.shredResult;
    body = `<div class="card flat">
      <div class="field-row" style="margin-bottom:12px">
        <div class="field" style="flex:2"><input type="text" id="shredPath" value="${esc(path)}" placeholder="${esc(state.lang === "ar" ? "مسار الملف أو المجلد" : "File or folder path")}" /></div>
        <button class="btn ghost" id="shredPick">${svg("folder")}${esc(t("chooseFolder"))}</button>
      </div>
      <div class="field-row" style="margin-bottom:12px">
        <div class="field"><label>${esc(t("shredPasses"))}</label><select id="shredPasses"><option value="1">1</option><option value="3">3 (DoD)</option></select></div>
        <button class="btn ghost small" id="shredScan">${svg("search", 14)}${esc(t("unlockerScan"))}</button>
        <button class="btn danger" id="shredRun">${svg("trash")}${esc(t("shredFile"))}</button>
      </div>
      ${locks && locks.length ? `<div class="tiny muted" style="margin-bottom:8px">${esc(t("lockingProcesses"))}: ${locks.map((l) => esc(l.name) + " (PID " + l.pid + ")").join(", ")}</div>
        <button class="btn small danger" id="shredKill">${esc(t("killAndDelete"))}</button>` : (locks ? `<div class="tiny muted" style="margin-bottom:8px">${esc(state.lang === "ar" ? "لا توجد عمليات مانعة" : "No locking processes")}</div>` : "")}
      ${res ? `<div class="tiny" style="font-weight:600;margin-top:10px">${esc(res.ok ? (state.lang === "ar" ? "تم" : "Done") : (state.lang === "ar" ? "فشل" : "Failed"))}${res.message ? " · " + esc(res.message) : ""}</div>` : ""}
      <div class="tiny muted" style="margin-top:10px;color:var(--danger)">${esc(t("shredderWarn"))}</div>
    </div>`;
  } else if (tab === "fonts") {
    if (state.tools.fonts === undefined) {
      state.tools.fonts = null;
      call("fonts_list", {}).then((l) => { state.tools.fonts = l; if (state.page === "tools" && state.tools.tab === "fonts") go("tools"); }).catch(() => {});
    }
    const fonts = state.tools.fonts;
    const preview = state.tools.fontPreview || "VEXORA — خط تجريبي — ABC abc 123";
    body = `<div class="card flat">
      <div class="field-row" style="margin-bottom:12px">
        <div class="field" style="flex:2"><input type="text" id="fontPreviewText" value="${esc(preview)}" /></div>
      </div>
      ${fonts
        ? `<table class="table"><thead><tr><th>${esc(t("ctxName"))}</th><th>${esc(t("driveType"))}</th><th>${esc(t("ctxFriendly"))}</th><th></th></tr></thead><tbody>
           ${fonts.map((f, i) => `<tr>
             <td class="tiny" style="direction:ltr;text-align:start">${esc(f.name)}</td>
             <td><span class="pill ${f.source === "system" ? "ok" : ""}">${esc(f.source === "system" ? t("fontSystem") : t("fontUser"))}</span>${f.exists ? "" : ` <span class="pill bad">${esc(t("fontBroken"))}</span>`}${f.duplicate ? ` <span class="pill warn">${esc(t("fontDuplicate"))}</span>` : ""}</td>
             <td style="font-family:'${esc(f.name)}',sans-serif;font-size:16px">${esc(preview)}</td>
             <td style="text-align:end">
               <button class="btn small ghost" data-fontbackup="${i}">${esc(t("fontBackup"))}</button>
               ${f.source === "user" ? `<button class="btn small danger" data-fontdel="${i}">${esc(t("fontDelete"))}</button>` : ""}
             </td>
           </tr>`).join("")}
           </tbody></table>`
        : `<div class="tiny muted">${esc(t("scanning"))}</div>`}
    </div>`;
  } else if (tab === "similar") {
    const res = state.tools.similar;
    body = `<div class="card flat">
      ${pathPicker("similar")}
      <div class="field-row" style="margin-top:12px">
        <div class="field"><label>${esc(t("maxDiff"))}</label><input type="number" id="simDiff" min="0" max="40" value="${res ? res._diff : 10}" /></div>
        <button class="btn primary" id="simScan">${svg("search")}${esc(t("scan"))}</button>
      </div>
      ${res && res.groups && res.groups.length
        ? `<div style="margin-top:14px" class="tiny muted">${res.groups.length} ${esc(t("groups"))}</div>
           <div style="margin-top:10px">${res.groups
             .slice(0, 80)
             .map(
               (g, gi) => `<div class="card flat" style="margin-top:8px;padding:10px">
             <div class="tiny muted" style="margin-bottom:6px">${g.images.length} ${esc(state.lang === "ar" ? "صور متشابهة" : "similar images")} · ${esc(t("maxDiff"))} ${g.difference}</div>
             <div style="display:flex;flex-wrap:wrap;gap:10px">${g.images
               .map(
                 (im, ii) => `<label style="display:flex;flex-direction:column;align-items:center;gap:4px;cursor:pointer;width:112px">
               <img src="${im.thumb ? "data:image/png;base64," + im.thumb : ""}" style="width:96px;height:96px;object-fit:cover;border-radius:8px;border:1px solid var(--line);background:var(--card-3)" draggable="false" />
               <input type="checkbox" data-sim="${gi}:${ii}" ${ii > 0 ? "checked" : ""} style="accent-color:var(--accent)" />
               <span class="tiny muted ellipsis" style="max-width:104px;direction:ltr" title="${esc(im.path)}">${esc(im.path.split(/[\\\\/]/).pop())}</span>
             </label>`
               )
               .join("")}</div>
           </div>`
             )
             .join("")}</div>
           <button class="btn danger" id="simDelete" style="margin-top:12px">${esc(t("deleteSelected"))}</button>`
        : res ? `<div class="tiny muted" style="margin-top:12px">${esc(t("nothingFound"))}</div>` : ""}
    </div>`;
  } else {
    const w = state.tools.wipe || { drive: (state.health && state.health.disks[0] ? state.health.disks[0].path : "C:\\"), passes: 1, running: false, written: 0, total: 0 };
    const pct = w.total ? Math.min(100, (w.written / w.total) * 100) : 0;
    body = `<div class="card flat">
      <div class="field-row">
        <div class="field"><label>${esc(t("disk"))}</label><select id="wipeDrive">
          ${((state.health && state.health.disks) || [{ path: "C:\\" }])
            .map((d) => `<option ${d.path === w.drive ? "selected" : ""}>${esc(d.path)}</option>`)
            .join("")}
        </select></div>
        <div class="field"><label>${esc(t("wipePasses"))}</label><select id="wipePasses">
          <option value="1" ${w.passes === 1 ? "selected" : ""}>1</option>
          <option value="3" ${w.passes === 3 ? "selected" : ""}>3</option>
        </select></div>
        <button class="btn ${w.running ? "danger" : "primary"}" id="wipeBtn">${svg(w.running ? "x" : "shield")}${esc(w.running ? t("stop") : t("start"))}</button>
      </div>
      ${w.running || w.written ? `<div class="progress" style="margin-top:16px"><i style="width:${pct.toFixed(1)}%"></i></div>
        <div class="tiny muted" style="margin-top:6px">${esc(t("wipeRunning"))} ${fmtBytes(w.written)} / ${fmtBytes(w.total)}</div>` : ""}
      <div class="tiny muted" style="margin-top:14px">${esc(state.lang === "ar" ? "يكتب بيانات عشوائية فوق المساحة الحرة لتمنع استعادة الملفات المحذوفة. لا يمس ملفاتك الحالية." : "Overwrites free space so deleted files cannot be recovered. Your existing files are untouched.")}</div>
    </div>`;
  }

  const toolCards = [
    ["dupes", "apps", t("dupes"), t("toolDupesDesc")],
    ["big", "file", t("bigFiles"), t("toolBigDesc")],
    ["empty", "folder", t("emptyDirs"), t("toolEmptyDesc")],
    ["map", "disk", t("diskMap"), t("toolMapDesc")],
    ["drives", "disk", t("drives"), t("toolDrivesDesc")],
    ["sysrepair", "shield", t("sysRepair"), t("toolSysRepairDesc")],
    ["spooler", "tools", t("spooler"), t("toolSpoolerDesc")],
    ["contextmenu", "settings", t("contextMenu"), t("toolContextMenuDesc")],
    ["ramopt", "zap", t("ramOptimizer"), t("toolRamOptimizerDesc")],
    ["hwmon", "shield", t("hwmon"), t("toolHwmonDesc")],
    ["battery", "shield", t("battery"), t("toolBatteryDesc")],
    ["shredder", "shield", t("shredder"), t("toolShredderDesc")],
    ["fonts", "file", t("fonts"), t("toolFontsDesc")],
    ["adobecache", "folder", t("adobeCache"), t("toolAdobeCacheDesc")],
    ["runtimes", "shield", t("runtimes"), t("toolRuntimesDesc")],
    ["wipe", "shield", t("wipe"), t("toolWipeDesc")],
    ["browsers", "browser", t("browsers"), t("toolBrowsersDesc")],
    ["junk", "tools", t("junkFiles"), t("toolJunkDesc")],
    ["services", "settings", t("services"), t("toolServicesDesc")],
    ["monitor", "search", t("monitor"), t("toolMonitorDesc")],
    ["similar", "apps", t("similarImages"), t("toolSimilarDesc")],
  ];

  $("#view").innerHTML =
    pageHead(t("nav.tools"), t("subtitle")) +
    `<div class="tool-grid" style="display:grid;grid-template-columns:repeat(auto-fill,minmax(160px,1fr));gap:10px;margin-bottom:14px">
      ${toolCards
        .map(
          ([id, ic, name, desc]) =>
            `<button class="tool-card ${tab === id ? "active" : ""}" data-tool="${id}" style="text-align:start;padding:12px;border:1px solid var(--line);border-radius:12px;background:${tab === id ? "var(--accent-soft,rgba(99,102,241,.14))" : "var(--card,transparent)"};cursor:pointer;display:flex;flex-direction:column;gap:6px">
              <span style="display:flex;align-items:center;gap:8px;font-weight:600">${svg(ic, 18)} ${esc(name)}</span>
              <span class="tiny muted" style="line-height:1.4">${esc(desc)}</span>
            </button>`
        )
        .join("")}
    </div>` + body;

  $$("[data-tool]").forEach((b) => (b.onclick = () => { state.tools.tab = b.dataset.tool; go("tools"); }));
  $$("[data-pick]").forEach((b) =>
    b.addEventListener("click", async () => {
      const target = b.dataset.pick;
      const path = await call("pick_folder");
      if (path) {
        state.tools.root = path;
        go("tools");
      }
      void target;
    })
  );
  $$("[data-pick-roots]").forEach((b) =>
    b.addEventListener("click", async () => {
      const path = await call("pick_folder");
      if (!path) return;
      const cur = state.tools.roots && state.tools.roots.length ? state.tools.roots : [state.tools.root || "C:\\"];
      if (!cur.includes(path)) cur.push(path);
      state.tools.roots = cur;
      go("tools");
    })
  );
  $$("[data-reveal]").forEach((b) =>
    b.addEventListener("click", () => invoke("reveal_path", { path: b.dataset.reveal }))
  );

  if (tab === "similar" && $("#simScan"))
    $("#simScan").onclick = async () => {
      const root = state.tools.root;
      const diff = Number($("#simDiff").value) || 10;
      const res = await call("similar_images_scan", { root, maxDiff: diff }, t("scanning"));
      res._diff = diff;
      state.tools.results.similar = res;
      go("tools");
    };
  if (tab === "dupes" && $("#dupScan"))
    $("#dupScan").onclick = async () => {
      const roots = ($("#dupRoots") ? $("#dupRoots").value : state.tools.root || "")
        .split(/\r?\n/)
        .map((s) => s.trim())
        .filter(Boolean);
      if (!roots.length) return toast(t("nothingSelected"), "warn");
      state.tools.roots = roots;
      const min = (Number($("#dupMin").value) || 64) * 1024;
      state.tools.scanning = true;
      state.tools.dupScanned = 0;
      await go("tools");
      try {
        const res = await call("dup_scan", { roots, minSize: min, engine: "classic" });
        res._min = min / 1024;
        state.tools.results.dupes = res;
      } finally {
        state.tools.scanning = false;
      }
      go("tools");
    };
  if ($("#dupStop"))
    $("#dupStop").onclick = async () => {
      try { await call("dup_cancel", {}); } catch (e) {}
      toast(state.lang === "ar" ? "جارٍ الإيقاف…" : "Stopping…", "warn");
    };
  if ($("#dupDelete"))
    $("#dupDelete").onclick = async () => {
      const boxes = $$("[data-dup]").filter((b) => b.checked);
      const res = state.tools.results.dupes;
      const paths = boxes.map((b) => {
        const [gi, fi] = b.dataset.dup.split(":").map(Number);
        return res.groups[gi].files[fi];
      });
      if (!paths.length) return toast(t("nothingSelected"), "warn");
      if (!(await confirmBox(t("confirmDelete"), t("delete")))) return;
      const out = await call("delete_paths", { paths, shred: state.settings.shred }, t("cleaning"));
      deleteResultToast(out);
      state.tools.results.dupes = null;
      go("tools");
    };
  if ($("#simDelete"))
    $("#simDelete").onclick = async () => {
      const boxes = $$("[data-sim]").filter((b) => b.checked);
      const res = state.tools.results.similar;
      if (!res || !res.groups) return;
      const paths = boxes.map((b) => {
        const [gi, ii] = b.dataset.sim.split(":").map(Number);
        return res.groups[gi].images[ii].path;
      });
      if (!paths.length) return toast(t("nothingSelected"), "warn");
      if (!(await confirmBox(t("confirmDelete"), t("delete")))) return;
      const out = await call("delete_paths", { paths, shred: state.settings.shred }, t("cleaning"));
      deleteResultToast(out);
      state.tools.results.similar = null;
      go("tools");
    };

  if (tab === "drives" && $("#drivesScan"))
    $("#drivesScan").onclick = async () => {
      const list = await call("drives_list", {}, t("scanning"));
      state.tools.drives = list;
      go("tools");
    };
  if ($("#driveElevate"))
    $("#driveElevate").onclick = async () => {
      try { await call("relaunch_admin", {}); } catch (e) { toast(friendlyError(e), "error"); }
    };
  $$("[data-drivecheck]").forEach((b) =>
    b.addEventListener("click", async () => {
      const letter = b.dataset.drivecheck;
      const res = await call("disk_check", { letter }, t("scanning"));
      state.tools.driveCheck = { letter, ...res };
      go("tools");
    })
  );

  if (tab === "sysrepair") {
    if (!state.tools.repair) state.tools.repair = { running: false, stage: "", percent: -1, log: "", result: null };
    if ($("#repairElevate"))
      $("#repairElevate").onclick = async () => { try { await call("relaunch_admin", {}); } catch (e) { toast(friendlyError(e), "error"); } };
    if ($("#repairStart") && !state.tools.repair.running)
      $("#repairStart").onclick = async () => {
        state.tools.repair = { running: true, stage: "", percent: 0, log: "", result: null };
        await go("tools");
        try {
          const res = await call("system_repair_run", {});
          state.tools.repair.running = false;
          state.tools.repair.result = res.summary;
          if (res.log) state.tools.repair.log = res.log;
        } catch (e) {
          state.tools.repair.running = false;
          state.tools.repair.result = "errors";
          state.tools.repair.log += "\n" + (e && e.message ? e.message : e);
        }
        go("tools");
      };
    if ($("#repairCancel"))
      $("#repairCancel").onclick = async () => { try { await call("system_repair_cancel", {}); } catch (e) {} };
  }
  if (tab === "spooler") {
    if ($("#spoolerRefresh"))
      $("#spoolerRefresh").onclick = async () => {
        const n = await call("spooler_status", {});
        state.tools.spoolerPending = n;
        go("tools");
      };
    if ($("#spoolerFix"))
      $("#spoolerFix").onclick = async () => {
        const r = await call("spooler_fix", {}, t("cleaning"));
        state.tools.spoolerResult = { ok: r[0], msg: r[1] };
        const n = await call("spooler_status", {});
        state.tools.spoolerPending = n;
        go("tools");
      };
    if ($("#spoolerElevate"))
      $("#spoolerElevate").onclick = async () => { try { await call("relaunch_admin", {}); } catch (e) { toast(friendlyError(e), "error"); } };
  }
  if (tab === "contextmenu") {
    if ($("#ctxClassicToggle"))
      $("#ctxClassicToggle").onclick = async () => {
        const next = !$("#ctxClassicToggle").classList.contains("on");
        $("#ctxClassicToggle").classList.toggle("on", next);
        try {
          const ok = await call("ctxmenu_win11_classic", { enabled: next });
          if (ok === false) {
            $("#ctxClassicToggle").classList.toggle("on", !next);
            toast(t("ctxAdminDenied"), "warn");
            return;
          }
        } catch (e) { toast(friendlyError(e), "error"); }
        state.tools.ctxClassic = next;
      };
    $$("[data-ctx]").forEach((b) =>
      b.addEventListener("click", async () => {
        const idx = Number(b.dataset.ctx);
        const it = (state.tools.ctxItems || [])[idx];
        if (!it) return;
        const ok = await call("ctxmenu_set", { scope: it.scope, name: it.name, clsid: it.clsid, enable: !it.enabled });
        if (ok) { it.enabled = !it.enabled; go("tools"); }
        else toast(t("ctxAdminDenied"), "warn");
      })
    );
  }

  if (tab === "adobecache") {
    if ($("#adobeScan"))
      $("#adobeScan").onclick = async () => {
        state.tools.adobeItems = undefined;
        state.tools.adobeRunning = undefined;
        await go("tools");
      };
    if ($("#adobeClean"))
      $("#adobeClean").onclick = async () => {
        const items = state.tools.adobeItems || [];
        const boxes = $$("[data-adobe]").filter((b) => b.checked);
        const paths = boxes.map((b) => items[Number(b.dataset.adobe)].path);
        if (!paths.length) return toast(t("nothingSelected"), "warn");
        if (!(await confirmBox(t("confirmDelete"), t("delete")))) return;
        const out = await call("delete_paths", { paths, shred: false }, t("cleaning"));
        deleteResultToast(out);
        state.tools.adobeItems = undefined;
        go("tools");
      };
  }
  if (tab === "hwmon") {
    const btn = $("#hwRefresh");
    if (btn)
      btn.onclick = async () => {
        try {
          const h = await call("hwmon_status", {}, t("hwmonRefresh"));
          state.tools.hwmon = h;
          go("tools");
        } catch (e) {
          toast(String(e), "error");
        }
      };
  }
  if (tab === "runtimes") {
    if ($("#driverBackupPick"))
      $("#driverBackupPick").addEventListener("click", async () => {
        const p = await call("pick_folder");
        if (p) { state.tools.driverBackupFolder = p; go("tools"); }
      });
    if ($("#driverBackupRun"))
      $("#driverBackupRun").onclick = async () => {
        const folder = ($("#driverBackupFolder") ? $("#driverBackupFolder").value : state.tools.driverBackupFolder || "").trim();
        if (!folder) return toast(t("nothingSelected"), "warn");
        state.tools.driverBackupFolder = folder;
        const r = await call("drivers_backup", { folder }, t("cleaning"));
        state.tools.driverBackupResult = { ok: r[0], msg: r[1] };
        go("tools");
      };
    $$("[data-rt]").forEach((b) =>
      b.addEventListener("click", async () => {
        const idx = Number(b.dataset.rt);
        const r = state.tools.runtimes[idx];
        if (!r || !r.wingetId) return;
        b.disabled = true;
        const out = await call("winget_install", { id: r.wingetId }, t("cleaning"));
        toast(out[0] ? t("spoolerFixed") : t("spoolerFixFailed"), out[0] ? "ok" : "error");
        const fresh = await call("runtimes_status", {}, t("scanning"));
        state.tools.runtimes = fresh;
        go("tools");
      })
    );
  }

  if (tab === "ramopt") {
    if ($("#ramClean"))
      $("#ramClean").onclick = async () => {
        const r = await call("ram_clean", {}, t("cleaning"));
        state.tools.ramResult = { freed: r[0], trimmed: r[1] };
        const s = await call("ram_status", {});
        state.tools.ramStatus = s;
        go("tools");
      };
  }
  if (tab === "battery") {
    if ($("#batMonToggle"))
      $("#batMonToggle").onclick = async () => {
        const next = !$("#batMonToggle").classList.contains("on");
        $("#batMonToggle").classList.toggle("on", next);
        state.settings = { ...state.settings, batteryMonitorEnabled: next };
        await persist({ batteryMonitorEnabled: next });
      };
    if ($("#batApply"))
      $("#batApply").onclick = async () => {
        const upper = Number($("#batUpper").value) || 95;
        const lower = Number($("#batLower").value) || 20;
        await persist({ batteryUpperThreshold: upper, batteryLowerThreshold: lower });
        await call("battery_monitor_apply", { enabled: !!state.settings.batteryMonitorEnabled, upper, lower });
        toast(t("saved"));
      };
  }

  if (tab === "shredder") {
    if ($("#shredPick"))
      $("#shredPick").addEventListener("click", async () => {
        const p = await call("pick_file");
        if (p) { state.tools.shredPath = p; go("tools"); }
      });
    if ($("#shredScan"))
      $("#shredScan").onclick = async () => {
        const path = ($("#shredPath") ? $("#shredPath").value : state.tools.shredPath || "").trim();
        if (!path) return;
        state.tools.shredPath = path;
        const locks = await call("unlocker_list", { path });
        state.tools.shredLocks = locks;
        go("tools");
      };
    if ($("#shredRun"))
      $("#shredRun").onclick = async () => {
        const path = ($("#shredPath") ? $("#shredPath").value : state.tools.shredPath || "").trim();
        if (!path) return toast(t("nothingSelected"), "warn");
        if (!(await typedConfirmBox(t("shredderWarn"), t("shredFile"), "DELETE"))) return;
        const passes = Number($("#shredPasses").value) || 1;
        const res = await call("shred_path", { path, passes }, t("cleaning"));
        state.tools.shredResult = res;
        state.tools.shredLocks = null;
        go("tools");
      };
    if ($("#shredKill"))
      $("#shredKill").onclick = async () => {
        const path = state.tools.shredPath;
        if (!path) return toast(t("nothingSelected"), "warn");
        if (!(await typedConfirmBox(t("shredderWarn"), t("shredFile"), "DELETE"))) return;
        const locks = state.tools.shredLocks || [];
        for (const l of locks) {
          try { await call("kill_process", { pid: l.pid }); } catch (e) {}
        }
        const passes = Number($("#shredPasses").value) || 1;
        const res = await call("shred_path", { path, passes }, t("cleaning"));
        state.tools.shredResult = res;
        state.tools.shredLocks = null;
        go("tools");
      };
  }
  if (tab === "fonts") {
    if ($("#fontPreviewText"))
      $("#fontPreviewText").onchange = (e) => { state.tools.fontPreview = e.target.value; go("tools"); };
    $$("[data-fontbackup]").forEach((b) =>
      b.addEventListener("click", async () => {
        const idx = Number(b.dataset.fontbackup);
        const f = (state.tools.fonts || [])[idx];
        if (!f) return;
        const dest = await call("pick_folder");
        if (!dest) return;
        const ok = await call("font_backup", { name: f.name, path: f.path, source: f.source, dest });
        toast(ok ? t("spoolerFixed") : t("spoolerFixFailed"), ok ? "ok" : "error");
      })
    );
    $$("[data-fontdel]").forEach((b) =>
      b.addEventListener("click", async () => {
        const idx = Number(b.dataset.fontdel);
        const f = (state.tools.fonts || [])[idx];
        if (!f) return;
        if (!(await confirmBox(t("confirmDelete"), t("delete")))) return;
        const ok = await call("font_delete", { name: f.name, path: f.path, source: f.source });
        if (ok) { state.tools.fonts = undefined; go("tools"); }
      })
    );
  }

  if (tab === "big" && $("#bigScan"))
    $("#bigScan").onclick = async () => {
      const min = (Number($("#bigMin").value) || 100) * 1024 * 1024;
      const files = await call("big_scan", { root: state.tools.root, minSize: min, limit: 120 }, t("scanning"));
      state.tools.results.big = { files, _min: min / 1024 / 1024 };
      go("tools");
    };

  if (tab === "empty" && $("#emptyScan"))
    $("#emptyScan").onclick = async () => {
      const dirs = await call("empty_scan", { root: state.tools.root, limit: 500 }, t("scanning"));
      state.tools.results.empty = { dirs };
      go("tools");
    };
  if ($("#emptySelAll"))
    $("#emptySelAll").onclick = () => { $$("[data-empty]").forEach((c) => (c.checked = true)); };
  if ($("#emptySelNone"))
    $("#emptySelNone").onclick = () => { $$("[data-empty]").forEach((c) => (c.checked = false)); };
  if ($("#emptyDelete"))
    $("#emptyDelete").onclick = async () => {
      const res = state.tools.results.empty;
      if (!res || !res.dirs.length) return;
      const paths = $$("[data-empty]:checked").map((c) => res.dirs[Number(c.dataset.empty)]);
      if (!paths.length) return toast(t("nothingSelected"), "warn");
      if (!(await confirmBox(t("confirmDelete"), t("delete")))) return;
      const out = await call("delete_paths", { paths, shred: false }, t("cleaning"));
      state.tools.results.empty = null;
      deleteResultToast(out);
      go("tools");
    };

  if (tab === "map" && $("#mapScan"))
    $("#mapScan").onclick = async () => {
      const node = await call("disk_map", { root: state.tools.root, depth: 1 }, t("scanning"));
      state.tools.results.map = node;
      go("tools");
    };

  if (tab === "browsers") {
    state.tools.browserCats = state.tools.browserCats || ["cache", "cookies", "history", "sessions"];
    $$("[data-bcat]").forEach((cb) => {
      cb.onchange = () => {
        const c = cb.dataset.bcat;
        if (cb.checked) {
          if (!state.tools.browserCats.includes(c)) state.tools.browserCats.push(c);
        } else {
          state.tools.browserCats = state.tools.browserCats.filter((x) => x !== c);
        }
      };
    });
    const cleanOne = async (id) => {
      const cats = state.tools.browserCats || [];
      const out = await call("browser_clean", { id, categories: cats }, t("cleaning"));
      toast(t("cleanResult", { n: out.removed, size: fmtBytes(out.freed) }));
      go("tools");
    };
    $$("[data-bclean]").forEach((b) => {
      b.onclick = () => cleanOne(b.dataset.bclean);
    });
    if ($("#browserCleanAll"))
      $("#browserCleanAll").onclick = async () => {
        const bl = await call("browsers_list");
        const cats = state.tools.browserCats || [];
        let total = 0, freed = 0;
        for (const b of bl) {
          if (!b.installed) continue;
          const out = await call("browser_clean", { id: b.id, categories: cats });
          total += out.removed; freed += out.freed;
        }
        toast(t("cleanResult", { n: total, size: fmtBytes(freed) }));
        go("tools");
      };
  }

  if (tab === "monitor") {
    if (!state.tools.monitorLogs) {
      (async () => {
        state.tools.monitorLogs = await call("monitor_list", {}, t("scanning"));
        go("tools");
      })();
    }
    if ($("#monStart"))
      $("#monStart").onclick = async () => {
        const name = ($("#monName").value || "").trim();
        if (!name) return toast(t("nothingSelected"), "warn");
        await call("monitor_start", { name }, t("scanning"));
        state.tools.monitorName = name;
        state.tools.monitorActive = true;
        toast(t("done"));
        go("tools");
      };
    if ($("#monFinish"))
      $("#monFinish").onclick = async () => {
        const name = state.tools.monitorName || ($("#monName").value || "").trim();
        if (!name) return toast(t("nothingSelected"), "warn");
        const log = await call("monitor_finish", { name }, t("scanning"));
        state.tools.monitorName = "";
        state.tools.monitorActive = false;
        state.tools.monitorLogs = null;
        toast(t("done") + " (" + log.changes.length + " changes)");
        go("tools");
      };
    $$("[data-monuninst]").forEach((b) => {
      b.onclick = async () => {
        const name = b.dataset.monuninst;
        const plan = await call("monitor_uninstall_plan", { name }, t("scanning"));
        const paths = await showCleanupPlan(t("monitorUninstall"), plan, t("monitorUninstall"));
        if (!paths || !paths.length) return;
        const n = await call("monitor_uninstall_selected", { name, paths }, t("cleaning"));
        state.tools.monitorLogs = null;
        toast(`${t("done")} (${n}) — ${t("quarantineNote")}`);
        go("tools");
      };
    });
  }

  if (tab === "services") {
    if (!state.tools.services) {
      (async () => {
        state.tools.services = await call("services_list", {}, t("scanning"));
        state.tools.tasks = await call("tasks_list", {}, t("scanning"));
        go("tools");
      })();
    }
    if ($("#svcRefresh"))
      $("#svcRefresh").onclick = async () => {
        state.tools.services = await call("services_list", {}, t("scanning"));
        state.tools.tasks = await call("tasks_list", {}, t("scanning"));
        go("tools");
      };
    $$("[data-svcon]").forEach((b) => {
      b.onclick = async () => {
        await call("service_set", { name: b.dataset.svcon, enabled: true }, t("scanning"));
        state.tools.services = null;
        toast(t("done"));
        go("tools");
      };
    });
    $$("[data-svcoff]").forEach((b) => {
      b.onclick = async () => {
        if (!(await confirmBox(t("confirmDisable"), t("disable")))) return;
        await call("service_set", { name: b.dataset.svcoff, enabled: false }, t("scanning"));
        state.tools.services = null;
        toast(t("done"));
        go("tools");
      };
    });
    $$("[data-taskon]").forEach((b) => {
      b.onclick = async () => {
        await call("task_set", { name: b.dataset.taskon, enabled: true }, t("scanning"));
        state.tools.tasks = null;
        toast(t("done"));
        go("tools");
      };
    });
    $$("[data-taskoff]").forEach((b) => {
      b.onclick = async () => {
        if (!(await confirmBox(t("confirmDisable"), t("disable")))) return;
        await call("task_set", { name: b.dataset.taskoff, enabled: false }, t("scanning"));
        state.tools.tasks = null;
        toast(t("done"));
        go("tools");
      };
    });
  }

  if (tab === "junk") {
    if ($("#junkScan"))
      $("#junkScan").onclick = async () => {
        const res = await call("junk_scan", { limit: 500 }, t("scanning"));
        state.tools.junk = res;
        go("tools");
      };
    if ($("#junkDelete"))
      $("#junkDelete").onclick = async () => {
        const checks = $$("[data-junk]:checked");
        const paths = checks.map((c) => state.tools.junk.items[Number(c.dataset.junk)].path);
        if (!paths.length) return toast(t("nothingSelected"), "warn");
        if (!(await confirmBox(t("confirmDelete"), t("delete")))) return;
        const out = await call("delete_paths", { paths, shred: false }, t("cleaning"));
        state.tools.junk = null;
        deleteResultToast(out);
        go("tools");
      };
  }

  if (tab === "wipe") {
    const driveSel = $("#wipeDrive");
    const passSel = $("#wipePasses");
    $("#wipeBtn").onclick = async () => {
      const w = state.tools.wipe || {};
      if (w.running) {
        await call("cancel_work");
        return;
      }
      const drive = driveSel.value;
      const passes = Number(passSel.value) || 1;
      const ok = await confirmBox(
        state.lang === "ar"
          ? `سيتم الكتابة فوق المساحة الحرة على ${drive} ${passes} مرة. لا يمكن الإيقاف فوراً. متابعة؟`
          : `Free space on ${drive} will be overwritten ${passes}×. Continue?`,
        t("start")
      );
      if (!ok) return;
      state.tools.wipe = { drive, passes, running: true, written: 0, total: 0 };
      go("tools");
      try {
        const bytes = await call("wipe_free_space", { drive, passes });
        state.tools.wipe = { drive, passes, running: false, written: bytes, total: bytes };
        toast(t("wipeDone", { size: fmtBytes(bytes) }));
      } catch (e) {
        state.tools.wipe.running = false;
        toast(String(e), "error");
      }
      go("tools");
    };
  }
}

function pathPicker(kind) {
  return `<div class="field-row">
    <div class="field" style="flex:2"><label>${esc(t("chooseFolder"))}</label>
      <input type="text" id="toolRoot" value="${esc(state.tools.root)}" /></div>
    <button class="btn" data-pick="${kind}">${svg("folder")}${esc(t("chooseFolder"))}</button>
  </div>`;
}

/* ------------------------------------------------------------------- HISTORY */
async function renderHistory() {
  const [entries, totals] = await Promise.all([
    call("history_list", { limit: 400 }),
    call("history_totals"),
  ]);
  const rows = entries
    .map(
      (e) => `<tr>
      <td><span class="pill">${esc(e.kind)}</span></td>
      <td>${esc(e.title)}<div class="tiny muted">${esc(e.detail)}</div></td>
      <td class="num">${e.size ? fmtBytes(e.size) : "—"}</td>
      <td class="num">${e.items || "—"}</td>
      <td class="num">${fmtDate(e.ts)}</td>
    </tr>`
    )
    .join("");

  $("#view").innerHTML =
    pageHead(
      t("nav.history"),
      `${fmtBytes(totals[0])} · ${totals[1]} ${esc(state.lang === "ar" ? "عنصر" : "items")}`,
      `${entries.length ? `<button class="btn ghost" id="histHtml">${svg("file")}${esc(t("exportHtml"))}</button>
        <button class="btn ghost" id="histCsv">${svg("file")}${esc(t("exportCsv"))}</button>
        <button class="btn danger" id="histClear">${svg("trash")}${esc(t("clearHistory"))}</button>` : ""}`
    ) +
    (entries.length
      ? `<div class="card flat"><table class="table">
          <thead><tr><th>${esc(t("name"))}</th><th></th><th>${esc(t("size"))}</th><th>${esc(t("items"))}</th><th>${esc(t("time"))}</th></tr></thead>
          <tbody>${rows}</tbody></table></div>`
      : `<div class="empty">${svg("clock", 44)}<div>${esc(t("historyEmpty"))}</div></div>`);

  const exportHistory = async (format) => {
    const dir = await call("pick_folder");
    if (!dir) return;
    const path = await call("export_report", { format, destDir: dir }, t("scanning"));
    toast(t("reportSaved", { path }));
  };
  if ($("#histHtml")) $("#histHtml").onclick = () => exportHistory("html");
  if ($("#histCsv")) $("#histCsv").onclick = () => exportHistory("csv");

  if ($("#histClear"))
    $("#histClear").onclick = async () => {
      if (!(await confirmBox(t("confirmDelete"), t("clearHistory")))) return;
      try {
        await call("history_clear");
      } catch (_) {
        return; // call() already showed the error toast
      }
      toast(t("done"));
      go("history");
    };
}

/* ------------------------------------------------------------------ SETTINGS */
async function renderSettings() {
  const s = state.settings;
  const info = state.info;
  const ex = s.exclusions || [];
  const wt = (ar, en) => (state.lang === "ar" ? ar : en);
  if (!vexAppearanceDraft) vexAppearanceDraft = vexDraftFromSettings();

  $("#view").innerHTML =
    pageHead(t("nav.settings"), t("subtitle")) +
    `<div class="grid cols-2">
      <div class="card flat">
        <div class="card-title">${svg("settings")}${esc(t("general"))}</div>
        <div class="setting"><div class="txt"><div class="t">${esc(t("language"))}</div></div>
          <select id="setLang" class="btn" style="padding:8px 10px">
            <option value="ar" ${s.language === "ar" ? "selected" : ""}>العربية</option>
            <option value="en" ${s.language === "en" ? "selected" : ""}>English</option>
          </select></div>
        <div class="setting"><div class="txt"><div class="t">${esc(t("theme"))}</div></div>
          <select id="setTheme" class="btn" style="padding:8px 10px">
            <option value="dark" ${s.theme === "dark" ? "selected" : ""}>${esc(t("dark"))}</option>
            <option value="light" ${s.theme === "light" ? "selected" : ""}>${esc(t("light"))}</option>
          </select></div>
        <div class="setting"><div class="txt"><div class="t">${esc(t("previewFirst"))}</div></div>
          <div class="toggle ${s.previewFirst ? "on" : ""}" data-set="previewFirst"></div></div>
        <div class="setting"><div class="txt"><div class="t">${esc(t("confirmBeforeClean"))}</div></div>
          <div class="toggle ${s.confirmClean ? "on" : ""}" data-set="confirmClean"></div></div>
        <div class="setting"><div class="txt"><div class="t">${esc(t("useRecycle"))}</div><div class="s">${esc(t("recycleHint"))}</div></div>
          <div class="toggle ${s.recycleBin ? "on" : ""}" data-set="recycleBin"></div></div>
        <div class="setting"><div class="txt"><div class="t">${esc(t("shred"))}</div></div>
          <div class="toggle ${s.shred ? "on" : ""}" data-set="shred"></div></div>
        <div class="setting"><div class="txt"><div class="t">${esc(t("shredPasses"))}</div></div>
          <input type="number" id="setPasses" min="1" max="7" value="${s.shredPasses}" style="width:80px;padding:8px;border-radius:9px;border:1px solid var(--line);background:var(--card-2);color:var(--text)" /></div>
      </div>

      <div class="card flat">
        <div class="card-title">${svg("clock")}${esc(t("schedule"))}</div>
        <div class="setting"><div class="txt"><div class="t">${esc(t("scheduleEnable"))}</div><div class="s">${esc(t("scheduleNote"))}</div></div>
          <div class="toggle ${s.schedule.enabled ? "on" : ""}" id="setSched"></div></div>
        <div class="setting"><div class="txt"><div class="t">${esc(t("daily"))} / ${esc(t("weekly"))} / ${esc(t("monthly"))}</div></div>
          <select id="setFreq" class="btn" style="padding:8px 10px">
            ${["daily", "weekly", "monthly"].map((f) => `<option value="${f}" ${s.schedule.frequency === f ? "selected" : ""}>${esc(t(f))}</option>`).join("")}
          </select></div>
        <div class="setting"><div class="txt"><div class="t">${esc(t("time"))}</div></div>
          <input type="time" id="setTime" value="${esc(s.schedule.time || "03:00")}" style="padding:8px;border-radius:9px;border:1px solid var(--line);background:var(--card-2);color:var(--text)" /></div>
      </div>
    </div>

    <div class="card flat vex-appearance" id="vexAppearance" style="margin-top:14px">
      <div class="card-title">${svg("settings")}${esc(t("appearance"))}</div>
      <div class="tiny muted" style="margin-bottom:12px">${esc(t("appearanceDesc"))}</div>
      <div class="vex-appearance-body">
        <div class="vex-appearance-controls">
          <div class="vex-sec-label">${esc(t("appearanceLayout"))}</div>
          <div class="vex-layout-picks">
            ${VEX_LAYOUTS.map(
              (l) =>
                `<button class="vex-layout-pick" data-vex-layout="${l}">` +
                `<span class="vex-layout-mini vex-mini-${l}"><i></i><i></i><i></i></span>` +
                `<span>${esc(t("layout" + l.charAt(0).toUpperCase() + l.slice(1)))}</span></button>`
            ).join("")}
          </div>
          <div class="vex-sec-label">${esc(t("appearanceAccent"))}</div>
          <div class="vex-swatches">
            ${Object.entries(VEX_ACCENTS)
              .map(
                ([k, v]) =>
                  `<button class="vex-swatch" data-vex-accent="${k}" title="${esc(t(v.label))}" ` +
                  `style="--sw:${v.hex}"><span style="background:${v.hex}"></span></button>`
              )
              .join("")}
          </div>
          <div class="vex-sec-label">${esc(t("appearanceMode"))}</div>
          <div class="vex-mode-picks">
            <button class="btn small" data-vex-mode="dark">${esc(t("dark"))}</button>
            <button class="btn small" data-vex-mode="light">${esc(t("light"))}</button>
          </div>
          <div class="vex-toggles">
            <div class="setting"><div class="txt"><div class="t">${esc(t("appearanceGlow"))}</div></div>
              <div class="toggle" data-vex-toggle="glow"></div></div>
            <div class="setting"><div class="txt"><div class="t">${esc(t("appearanceAnimation"))}</div></div>
              <div class="toggle" data-vex-toggle="animation"></div></div>
          </div>
          <div class="vex-appearance-actions">
            <button class="btn small primary" id="vexApply">${svg("check", 14)}${esc(t("appearanceApply"))}</button>
            <button class="btn small ghost" id="vexReset">${svg("refresh", 14)}${esc(t("appearanceReset"))}</button>
          </div>
        </div>
        <div class="vex-preview-wrap">
          <div class="tiny muted" style="margin-bottom:8px">${esc(t("appearancePreview"))}</div>
          <div id="vexPreview" class="vex-preview"></div>
          <div class="tiny muted" style="margin-top:8px">${esc(t("appearancePreviewHint"))}</div>
        </div>
      </div>
    </div>

    <div class="card flat" style="margin-top:14px">
      <div class="card-title">${svg("shield")}${esc(t("exclusions"))}</div>
      <div class="tiny muted" style="margin-bottom:10px">${esc(t("exclusionsHint"))}</div>
      <div id="exList">${ex
        .map(
          (p) => `<div class="row" style="padding:7px 0"><span class="grow tiny ellipsis" style="direction:ltr;text-align:start">${esc(p)}</span>
          <button class="btn small ghost" data-exdel="${esc(p)}">${esc(t("remove"))}</button></div>`
        )
        .join("") || `<div class="tiny muted">—</div>`}</div>
      <button class="btn small" id="exAdd" style="margin-top:10px">${svg("plus", 14)}${esc(t("add"))}</button>
    </div>

    <div class="card flat" style="margin-top:14px">
      <div class="card-title">${svg("shield")}${esc(t("safety"))}</div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("autoElevate"))}</div><div class="s">${esc(t("autoElevateHint"))}</div></div>
        <div class="toggle ${s.autoElevate ? "on" : ""}" data-set="autoElevate"></div></div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("restoreBeforeClean"))}</div><div class="s">${esc(t("restoreBeforeCleanHint"))}</div></div>
        <div class="toggle ${s.restoreBeforeClean ? "on" : ""}" data-set="restoreBeforeClean"></div></div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("restorePointBeforeTweaks"))}</div><div class="s">${esc(t("restoreBeforeTweaksHint"))}</div></div>
        <div class="toggle ${s.restoreBeforeTweaks ? "on" : ""}" data-set="restoreBeforeTweaks"></div></div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("minimizeToTray"))}</div><div class="s">${esc(t("minimizeToTrayHint"))}</div></div>
        <div class="toggle ${s.minimizeToTray ? "on" : ""}" data-set="minimizeToTray"></div></div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("showBubble"))}</div><div class="s">${esc(t("showBubbleHint"))}</div></div>
        <div class="toggle ${s.showBubble ? "on" : ""}" data-set="showBubble"></div></div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("restore"))}</div><div class="s">${esc(t("restoreHint"))}</div>
        <div class="tiny muted" style="margin-top:4px">${esc(t("lastRestore"))}: ${s.lastRestore ? esc(fmtDate(s.lastRestore)) : esc(t("never"))}</div></div>
        <div style="display:flex;gap:8px;flex-wrap:wrap">
          <button class="btn small" id="restoreCreate">${svg("check", 14)}${esc(t("createRestore"))}</button>
          <button class="btn small ghost" id="restoreOpen">${svg("clock", 14)}${esc(t("openRestore"))}</button>
        </div></div>
      <div id="restoreList" class="tiny muted" style="margin-top:8px"></div>
      <div style="margin-top:14px;border-top:1px solid var(--line);padding-top:12px">
        <div style="font-weight:700;margin-bottom:4px">${esc(t("restoreCenter"))}</div>
        <div class="tiny muted" style="margin-bottom:8px">${esc(t("restoreCenterHint"))}</div>
        <div id="changeLogBox" class="tiny muted">${esc(t("scanning"))}</div>
      </div>
    </div>

    <div class="card flat" style="margin-top:14px">
      <div class="card-title">${svg("zap")}${esc(t("bgMonitorTitle"))}</div>
      <div class="tiny muted" style="margin-bottom:10px">${esc(t("bgMonitorHint"))}</div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("bgMonitorEnabled"))}</div><div class="s">${esc(t("bgMonitorEnabledHint"))}</div></div>
        <div class="toggle ${s.bgMonitorEnabled ? "on" : ""}" data-set="bgMonitorEnabled"></div></div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("bgMonitorInterval"))}</div></div>
        <select id="bgMonInterval" class="btn" style="padding:8px 10px">
          ${[10,30,120,300].map((v) => `<option value="${v}" ${Number(s.bgMonitorIntervalMin) === v ? "selected" : ""}>${v} ${esc(t("minutes"))}</option>`).join("")}
          <option value="custom" ${![10,30,120,300].includes(Number(s.bgMonitorIntervalMin)) ? "selected" : ""}>${esc(t("custom"))}</option>
        </select>
        <input type="number" id="bgMonIntervalCustom" min="1" max="1440" value="${Number(s.bgMonitorIntervalMin) || 30}" style="width:70px;padding:8px;border-radius:9px;border:1px solid var(--line);background:var(--card-2);color:var(--text);margin-inline-start:6px" />
      </div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("bgMonitorMemThreshold"))}</div></div>
        <input type="number" id="bgMonMemThr" min="50" max="99" value="${Number(s.bgMonitorMemThreshold) || 80}" style="width:70px;padding:8px;border-radius:9px;border:1px solid var(--line);background:var(--card-2);color:var(--text)" /> %</div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("bgMonitorCpuThreshold"))}</div></div>
        <input type="number" id="bgMonCpuThr" min="50" max="99" value="${Number(s.bgMonitorCpuThreshold) || 80}" style="width:70px;padding:8px;border-radius:9px;border:1px solid var(--line);background:var(--card-2);color:var(--text)" /> %</div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("bgMonitorAutoClose"))}</div><div class="s">${esc(t("bgMonitorAutoCloseHint"))}</div></div>
        <div class="toggle ${s.bgMonitorAutoClose ? "on" : ""}" data-set="bgMonitorAutoClose"></div></div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("bgMonitorAutoClean"))}</div><div class="s">${esc(t("bgMonitorAutoCleanHint"))}</div></div>
        <div class="toggle ${s.bgMonitorAutoCleanTemp ? "on" : ""}" data-set="bgMonitorAutoCleanTemp"></div></div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("bgMonitorAllowed"))}</div><div class="s">${esc(t("bgMonitorAllowedHint"))}</div></div>
        <textarea id="bgMonAllowed" rows="3" style="width:100%;padding:8px;border-radius:9px;border:1px solid var(--line);background:var(--card-2);color:var(--text);font-family:monospace;font-size:12px">${esc((s.bgMonitorAllowedClose || []).join("\n"))}</textarea></div>
      <div class="setting"><div class="txt"><div class="t">${esc(t("bgMonitorRunWin"))}</div><div class="s">${esc(t("bgMonitorRunWinHint"))}</div></div>
        <div class="toggle ${s.bgMonitorRunWithWindows ? "on" : ""}" data-set="bgMonitorRunWithWindows"></div></div>
      <div style="display:flex;gap:8px;margin-top:10px">
        <button class="btn small" id="bgMonStatus">${svg("search",14)}${esc(t("bgMonitorCheck"))}</button>
        <span id="bgMonStatusOut" class="tiny muted" style="align-self:center"></span>
      </div>
    </div>

    <div class="card flat" style="margin-top:14px">
      <div class="card-title">${svg("settings")}${esc(wt("الفقاعة العائمة الذكية", "Smart Floating Bubble"))}</div>
      <div class="tiny muted" style="margin-bottom:10px">${esc(wt("شريط عائم مستقل يعرض RAM و CPU وسرعة الشبكة، مع سحب والتصاق بالحواف وإخفاء تلقائي وبيانات نظام حقيقية.", "An independent floating bar showing RAM, CPU and network speed with drag, edge snap, auto-hide and real system data."))}</div>
      ${[
        ["widgetEnabled", wt("تفعيل الفقاعة", "Enable widget")],
        ["widgetRunWithApp", wt("تشغيلها مع VEXORA", "Run with VEXORA")],
        ["widgetRunWithWindows", wt("تشغيلها مع Windows", "Run with Windows")],
        ["widgetAlwaysOnTop", wt("دائمًا في المقدمة", "Always on top")],
        ["widgetAutoHide", wt("إخفاء تلقائي على الحافة", "Auto-hide on edge")],
        ["widgetSnap", wt("التصاق بالحواف", "Snap to edges")],
      ].map(([k, label]) => `<div class="setting"><div class="txt"><div class="t">${esc(label)}</div></div>
        <div class="toggle ${s[k] ? "on" : ""}" data-wset="${k}"></div></div>`).join("")}
      <div class="setting"><div class="txt"><div class="t">${esc(wt("معدل التحديث (مللي ثانية)", "Refresh rate (ms)"))}</div></div>
        <select id="wRefresh" class="btn" style="padding:8px 10px">
          ${[1000,1500,2000,3000].map((v) => `<option value="${v}" ${Number(s.widgetRefreshMs) === v ? "selected" : ""}>${v}</option>`).join("")}
        </select></div>
      <div class="setting"><div class="txt"><div class="t">${esc(wt("مسافة الالتصاق", "Snap distance"))}</div></div>
        <input type="number" id="wSnapDist" min="8" max="80" value="${Number(s.widgetSnapDistance) || 28}" style="width:80px;padding:8px;border-radius:9px;border:1px solid var(--line);background:var(--card-2);color:var(--text)" /></div>
      <div class="setting"><div class="txt"><div class="t">${esc(wt("شفافية الفقاعة", "Widget opacity"))}</div></div>
        <input type="range" id="wOpacity" min="40" max="100" value="${Math.round((Number(s.widgetOpacity) || .96) * 100)}" /></div>
      <div class="setting"><div class="txt"><div class="t">${esc(wt("حجم الفقاعة", "Widget size"))}</div></div>
        <input type="range" id="wScale" min="80" max="150" value="${Math.round((Number(s.widgetScale) || 1) * 100)}" /></div>
      <div style="display:flex;gap:8px;flex-wrap:wrap;margin-top:10px">
        <button class="btn small" id="wResetPos">${svg("refresh", 14)}${esc(wt("استعادة الموضع الافتراضي", "Reset position"))}</button>
        <button class="btn small ghost" id="wHide">${esc(wt("إخفاء الفقاعة", "Hide widget"))}</button>
        <button class="btn small ghost" id="wShow">${esc(wt("إظهار الفقاعة", "Show widget"))}</button>
      </div>
    </div>

    <div class="card flat" style="margin-top:14px">
      <div class="card-title">${svg("refresh")}${esc(t("update"))}</div>
      <div class="kv"><span>${esc(t("currentVersion"))}</span><span>${esc(info.version)}</span></div>
      <div class="kv"><span>${esc(t("latestVersion"))}</span><span id="latestVer">—</span></div>
      <div class="field" style="margin-top:10px"><label>${esc(t("updateSource"))}</label>
        <input type="text" id="updateUrl" value="${esc(s.updateUrl || "")}" placeholder="https://…/latest.json" style="direction:ltr" /></div>
      <div class="tiny muted">${esc(t("updateSourceHint"))}</div>
      <div class="field" style="margin-top:10px"><label>${esc(t("githubRepo"))}</label>
        <input type="text" id="githubRepo" value="${esc(s.githubRepo || "ramadanfadel/VEXORA-1.4.0-Clean")}" placeholder="ramadanfadel/VEXORA-1.4.0-Clean" style="direction:ltr" /></div>
      <div class="tiny muted">${esc(t("githubRepoHint"))}</div>
      <div class="field" style="margin-top:10px"><label>${esc(t("githubToken"))}</label>
        <input type="password" id="githubToken" value="${esc(s.githubToken || "")}" placeholder="ghp_…" style="direction:ltr" /></div>
      <div class="tiny muted">${esc(t("githubTokenHint"))}</div>
      <div class="tiny muted" style="margin-top:4px">${svg("shield", 13)} ${esc(t("shaVerified"))}</div>
      <div style="display:flex;gap:8px;margin-top:12px;flex-wrap:wrap">
        <button class="btn small blue" id="checkUpd">${svg("refresh", 14)}${esc(t("checkUpdate"))}</button>
        <button class="btn small ghost" id="updSave">${svg("check", 14)}${esc(t("saved"))}</button>
        <button class="btn small ghost" id="githubPage">${svg("globe", 14)}${esc(t("githubPage"))}</button>
      </div>
    </div>

    <div class="card flat" style="margin-top:14px">
      <div class="card-title">${svg("folder")}${esc(t("backupRestore"))}</div>
      <div style="display:flex;gap:8px;flex-wrap:wrap">
        <button class="btn small" id="settingsExport">${svg("file", 14)}${esc(t("exportSettings"))}</button>
        <button class="btn small ghost" id="settingsImport">${svg("folder", 14)}${esc(t("importSettings"))}</button>
        <button class="btn small ghost" id="exportHtml">${svg("file", 14)}${esc(t("exportHtml"))}</button>
        <button class="btn small ghost" id="exportCsv">${svg("file", 14)}${esc(t("exportCsv"))}</button>
        <button class="btn small ghost" id="openReports">${svg("folder", 14)}${esc(t("reportsFolder"))}</button>
      </div>
    </div>

        <div class="card flat" style="margin-top:14px">
      <div class="card-title">${svg("cpu")}${esc(t("specs"))}</div>
      <div id="specsBox" class="tiny muted">${esc(t("scanning"))}</div>
    </div>

<div class="card flat" style="margin-top:14px">
      <div class="card-title">${svg("info")}${esc(t("about"))}</div>
      <div class="about-brand">
        <img src="assets/logo.png" alt="VEXORA" />
        <div><strong>VEXORA</strong> <span>${esc(t("app.tagline"))}</span></div>
      </div>
      <div class="kv"><span>${esc(t("version"))}</span><span>${esc(info.version)}</span></div>
      <div class="kv"><span>${esc(t("designer"))}</span><span>${esc(t("creditShort"))}</span></div>
      <div class="kv"><span>Status</span><span>${esc(info.admin ? t("adminOn") : t("adminOff"))}</span></div>
      <div class="kv"><span>${esc(t("dataFolder"))}</span><span class="ellipsis" style="max-width:420px">${esc(info.dataDir)}</span></div>
      <div class="kv"><span>${esc(t("backupsFolder"))}</span><span class="ellipsis" style="max-width:420px">${esc(info.backupsDir)}</span></div>
      <div style="display:flex;gap:8px;margin-top:12px;flex-wrap:wrap">
        <button class="btn small" id="openData">${svg("folder", 14)}${esc(t("openFolder"))}</button>
        <button class="btn small" id="openBackups">${svg("folder", 14)}${esc(t("openFolder"))}</button>
        ${info.admin ? "" : `<button class="btn small blue" id="relaunch">${svg("shield", 14)}${esc(t("relaunchAdmin"))}</button>`}
      </div>
    </div>`;

  const persist = async (patch) => {
    const next = { ...state.settings, ...patch };
    state.settings = await call("save_settings", { settings: next });
    toast(t("saved"));
  };

  $("#setLang").onchange = async (e) => {
    await persist({ language: e.target.value });
    state.lang = e.target.value;
    applyShell();
    await loadCatalog();
    go("settings");
  };
  $("#setTheme").onchange = async (e) => {
    await persist({ theme: e.target.value });
    state.theme = e.target.value;
    applyShell();
  };
  $$("[data-set]").forEach((el) =>
    el.addEventListener("click", async () => {
      const key = el.dataset.set;
      const next = !state.settings[key];
      el.classList.toggle("on", next);
      await persist({ [key]: next });
      if (key.startsWith("bgMonitor")) {
        try { await call("bg_monitor_apply", { settings: state.settings }); } catch (e) { /* reported */ }
      }
      if (key === "bgMonitorRunWithWindows") {
        try { await call("widget_set_run_at_startup", { enable: next }); } catch (e) { toast(friendlyError(e), "error"); }
      }
    })
  );
  // Background monitor extra controls
  const applyBgMon = async (patch) => {
    const next = { ...state.settings, ...patch };
    state.settings = await call("save_settings", { settings: next });
    try { await call("bg_monitor_apply", { settings: state.settings }); } catch (e) { /* reported */ }
    toast(t("saved"));
  };
  if ($("#bgMonInterval"))
    $("#bgMonInterval").onchange = async (e) => {
      const v = e.target.value === "custom" ? (Number($("#bgMonIntervalCustom").value) || 30) : Number(e.target.value);
      await applyBgMon({ bgMonitorIntervalMin: v });
    };
  if ($("#bgMonIntervalCustom"))
    $("#bgMonIntervalCustom").onchange = async (e) => {
      await applyBgMon({ bgMonitorIntervalMin: Number(e.target.value) || 30 });
    };
  if ($("#bgMonMemThr"))
    $("#bgMonMemThr").onchange = async (e) => {
      await applyBgMon({ bgMonitorMemThreshold: Number(e.target.value) || 80 });
    };
  if ($("#bgMonCpuThr"))
    $("#bgMonCpuThr").onchange = async (e) => {
      await applyBgMon({ bgMonitorCpuThreshold: Number(e.target.value) || 80 });
    };
  if ($("#bgMonAllowed"))
    $("#bgMonAllowed").onchange = async (e) => {
      const list = e.target.value.split(/\n+/).map((s) => s.trim()).filter(Boolean);
      await applyBgMon({ bgMonitorAllowedClose: list });
    };
  if ($("#bgMonStatus"))
    $("#bgMonStatus").onclick = async () => {
      const r = await call("bg_monitor_status", {}, t("scanning"));
      const out = $("#bgMonStatusOut");
      if (out) out.textContent = `mem ${r[1]}% · cpu ${r[2].toFixed(1)}% · ${r[0] ? "ON" : "OFF"}`;
    };

  // Bubble visibility toggle -> apply immediately
  const bubbleToggle = document.querySelector('[data-set="showBubble"]');
  if (bubbleToggle) {
    bubbleToggle.addEventListener("click", () => {
      const next = bubbleToggle.classList.contains("on");
      invoke("set_bubble_visible", { visible: next }).catch(() => {});
    });
  }
  // Smart Floating Bubble controls
  $$("[data-wset]").forEach((el) =>
    el.addEventListener("click", async () => {
      const key = el.dataset.wset;
      const next = !state.settings[key];
      el.classList.toggle("on", next);
      await persist({ [key]: next });
      if (key === "widgetEnabled") invoke("widget_set_visible", { visible: next }).catch(() => {});
      if (key === "widgetAlwaysOnTop") invoke("widget_set_always_on_top", { on: next }).catch(() => {});
      if (key === "widgetRunWithWindows") invoke("widget_set_run_at_startup", { enable: next }).catch(() => {});
    })
  );
  $("#wRefresh").onchange = (e) => persist({ widgetRefreshMs: Number(e.target.value) });
  $("#wSnapDist").onchange = (e) => persist({ widgetSnapDistance: Math.max(8, Math.min(80, Number(e.target.value) || 28)) });
  $("#wOpacity").onchange = (e) => persist({ widgetOpacity: Math.max(.4, Math.min(1, Number(e.target.value) / 100)) });
  $("#wScale").onchange = (e) => persist({ widgetScale: Math.max(.8, Math.min(1.5, Number(e.target.value) / 100)) });
  $("#wResetPos").onclick = async () => { await invoke("widget_reset_position").catch(() => {}); toast(t("saved")); };
  $("#wHide").onclick = () => invoke("widget_hide").catch(() => {});
  $("#wShow").onclick = () => invoke("widget_set_visible", { visible: true }).catch(() => {});
  $("#setPasses").onchange = (e) => persist({ shredPasses: Math.max(1, Number(e.target.value) || 1) });
  $("#setSched").onclick = async () => {
    const enabled = !state.settings.schedule.enabled;
    try {
      await call("schedule_set", {
        frequency: state.settings.schedule.frequency,
        time: state.settings.schedule.time,
        enabled,
      });
      await persist({ schedule: { ...state.settings.schedule, enabled } });
      go("settings");
    } catch (e) {
      toast(String(e), "error");
    }
  };
  $("#setFreq").onchange = async (e) => {
    await persist({ schedule: { ...state.settings.schedule, frequency: e.target.value } });
    if (state.settings.schedule.enabled)
      await call("schedule_set", { frequency: e.target.value, time: state.settings.schedule.time, enabled: true });
  };
  $("#setTime").onchange = async (e) => {
    await persist({ schedule: { ...state.settings.schedule, time: e.target.value } });
    if (state.settings.schedule.enabled)
      await call("schedule_set", { frequency: state.settings.schedule.frequency, time: e.target.value, enabled: true });
  };
  $("#exAdd").onclick = async () => {
    const path = await call("pick_folder");
    if (!path) return;
    const exclusions = [...(state.settings.exclusions || []), path];
    await persist({ exclusions });
    go("settings");
  };
  $$("[data-exdel]").forEach((b) =>
    b.addEventListener("click", async () => {
      const exclusions = (state.settings.exclusions || []).filter((p) => p !== b.dataset.exdel);
      await persist({ exclusions });
      go("settings");
    })
  );
  $("#openData").onclick = () => invoke("open_path", { path: info.dataDir });
  $("#openBackups").onclick = () => invoke("open_path", { path: info.backupsDir });
  if ($("#relaunch"))
    $("#relaunch").onclick = async () => {
      const ok = await call("relaunch_admin");
      if (ok) window.close();
      else toast(t("error"), "error");
    };

  call("system_specs").then((sp) => {
    const box = $("#specsBox");
    if (!box) return;
    const gpu = (sp.gpus || []).map((g) => g.name + (g.vram ? " (" + fmtBytes(g.vram) + ")" : "")).join(", ") || "—";
    const disks = (sp.disks || []).map((d) => d.path + " " + fmtBytes(d.total) + " " + (d.kind || "")).join(" | ");
    box.innerHTML =
      '<div class="kv"><span>' + esc(t("specCpu")) + '</span><span>' + esc(sp.cpuName) + ' · ' + sp.cpuCores + ' ' + esc(t("specCores")) + ' / ' + sp.cpuThreads + ' ' + esc(t("specThreads")) + ' · ' + sp.cpuMhz + ' MHz</span></div>' +
      '<div class="kv"><span>' + esc(t("specRam")) + '</span><span>' + fmtBytes(sp.ramTotal) + ' · ' + esc(sp.ramSlots) + ' ' + esc(t("specSlots")) + '</span></div>' +
      '<div class="kv"><span>' + esc(t("specGpu")) + '</span><span>' + esc(gpu) + '</span></div>' +
      '<div class="kv"><span>' + esc(t("specStorage")) + '</span><span>' + esc(disks) + '</span></div>' +
      '<div class="kv"><span>' + esc(t("specOs")) + '</span><span>' + esc(sp.osName) + ' ' + esc(sp.osVersion) + ' (' + esc(sp.osArch) + ')</span></div>' +
      '<div class="kv"><span>' + esc(t("specMotherboard")) + '</span><span>' + esc(sp.motherboard || "—") + '</span></div>' +
      '<div class="kv"><span>' + esc(t("specBios")) + '</span><span>' + esc(sp.bios || "—") + '</span></div>' +
      '<div class="kv"><span>' + esc(t("specHostname")) + '</span><span>' + esc(sp.hostname) + '</span></div>' +
      '<div class="kv"><span>' + esc(t("specUsername")) + '</span><span>' + esc(sp.username) + '</span></div>';
  }).catch(() => {});


  $("#restoreCreate").onclick = async () => {
    if (!(await confirmBox(t("restoreHint"), t("createRestore")))) return;
    try {
      await call("restore_create", { description: "VEXORA — نقطة استعادة يدوية" }, t("createRestore"));
      toast(t("restoreCreated"));
      state.settings = await call("get_settings");
      go("settings");
    } catch (e) {
      /* call() already showed the error */
    }
  };
  $("#restoreOpen").onclick = () => call("restore_open").catch(() => {});
  call("restore_points")
    .then((pts) => {
      const box = $("#restoreList");
      if (!box || !pts || !pts.length) return;
      box.innerHTML =
        `<div>${esc(t("restorePoints"))}:</div>` +
        pts.slice(0, 4).map((p) => `<div style="margin-top:3px">• ${esc(p.description || "-")} <span class="muted">${esc(p.created || "")}</span></div>`).join("");
    })
    .catch(() => {});
  call("tweak_change_log").then((changes)=>{
    const box=$("#changeLogBox"); if(!box) return;
    if(!changes || !changes.length){ box.textContent=t("noReversibleChanges"); return; }
    box.innerHTML = `<div class="filelist">${changes.slice(0,10).map(c=>`<div class="f" style="align-items:flex-start;gap:10px">
      <span style="min-width:0;flex:1"><div class="tiny" style="font-weight:600">${esc(c.id)}</div>
      <div class="tiny muted">${esc(new Date((c.at||0)*1000).toLocaleString())} • ${esc(c.action)} • ${c.on?"on":"off"} ${c.note?"• "+esc(c.note):""}</div></span>
      <button class="btn small ghost" data-restore-tweak="${esc(c.id)}">${esc(t("restoreOriginal"))}</button>
    </div>`).join("")}</div>`;
    $$("[data-restore-tweak]").forEach((b)=> b.onclick = async ()=>{ try{ await call("tweak_restore",{id:b.dataset.restoreTweak},t("cleaning")); toast(t("changeRestored")); go("settings"); }catch(e){} });
  }).catch(()=>{ const b=$("#changeLogBox"); if(b) b.textContent=t("noReversibleChanges"); });

  $("#updateUrl").onchange = (e) => persist({ updateUrl: e.target.value.trim() });
  $("#githubRepo").onchange = (e) => persist({ githubRepo: e.target.value.trim() });
  $("#githubToken").onchange = (e) => persist({ githubToken: e.target.value.trim() });
  $("#githubPage").onclick = () => invoke("open_github_page").catch(() => {});
  $("#checkUpd").onclick = () => showUpdate();
  $("#updSave").onclick = () =>
    persist({
      updateUrl: ($("#updateUrl").value || "").trim(),
      githubRepo: ($("#githubRepo").value || "").trim(),
      githubToken: ($("#githubToken").value || "").trim(),
    });

  const pickDir = async () => {
    const d = await call("pick_folder");
    return d ? d.trim() : "";
  };

  $("#settingsExport").onclick = async () => {
    const dir = await pickDir();
    if (!dir) return;
    const path = await call("export_settings", { destDir: dir });
    toast(t("settingsExported", { path }));
  };
  $("#settingsImport").onclick = async () => {
    const file = (await call("pick_file")).trim();
    if (!file) return;
    if (!(await confirmBox(t("importSettings"), t("importSettings")))) return;
    try {
      state.settings = await call("import_settings", { path: file });
      state.lang = state.settings.language || state.lang;
      state.theme = state.settings.theme || state.theme;
      applyShell();
      await loadCatalog();
      toast(t("settingsImported"));
      go("settings");
    } catch (e) {
      /* call() already showed the error */
    }
  };
  const exportReport = async (format) => {
    const dir = await pickDir();
    if (!dir) return;
    const path = await call("export_report", { format, destDir: dir });
    toast(t("reportSaved", { path }));
  };
  $("#exportHtml").onclick = () => exportReport("html");
  $("#exportCsv").onclick = () => exportReport("csv");
  $("#openReports").onclick = () => invoke("open_path", { path: info.reportsDir || info.dataDir });

  // ---- Appearance Studio (draft → Apply only) ----
  const applyAppearance = async (draft) => {
    state.settings = await call("save_settings", {
      settings: {
        ...state.settings,
        appearanceLayout: draft.layout,
        appearanceAccent: draft.accent,
        appearanceMode: draft.mode,
        appearanceGlow: draft.glow,
        appearanceAnimation: draft.animation,
        theme: draft.mode,
      },
    });
    state.theme = draft.mode;
    vexApplyAppearance();
    applyShell();
    vexAppearanceDraft = vexDraftFromSettings();
    vexRenderAppearancePreview();
    vexUpdateMyStyleLabel();
    toast(t("appearanceApplied"));
  };
  vexRenderAppearancePreview();
  $$("#vexAppearance [data-vex-layout]").forEach((b) => (b.onclick = (e) => {
    e.stopPropagation();
    vexAppearanceDraft.layout = b.dataset.vexLayout;
    vexRenderAppearancePreview();
  }));
  $$("#vexAppearance [data-vex-accent]").forEach((b) => (b.onclick = (e) => {
    e.stopPropagation();
    vexAppearanceDraft.accent = b.dataset.vexAccent;
    vexRenderAppearancePreview(false);
  }));
  $$("#vexAppearance [data-vex-mode]").forEach((b) => (b.onclick = (e) => {
    e.stopPropagation();
    vexAppearanceDraft.mode = b.dataset.vexMode;
    vexRenderAppearancePreview(false);
  }));
  $$("#vexAppearance [data-vex-toggle]").forEach((b) => (b.onclick = (e) => {
    e.stopPropagation();
    const k = b.dataset.vexToggle;
    vexAppearanceDraft[k] = !vexAppearanceDraft[k];
    vexRenderAppearancePreview(false);
  }));
  if ($("#vexApply")) $("#vexApply").onclick = () => applyAppearance(vexAppearanceDraft);
  if ($("#vexReset")) $("#vexReset").onclick = () => {
    vexAppearanceDraft = { ...VEX_APPEARANCE_DEFAULT };
    vexRenderAppearancePreview();
    applyAppearance(vexAppearanceDraft).then(() => toast(t("appearanceResetDone")));
  };
}

/* ------------------------------------------------------------------ updates */
async function showUpdate() {
  let info;
  try {
    info = await call("check_update", {}, t("checkUpdate"));
  } catch (e) {
    return;
  }
  const latest = info.error && !info.latest ? "—" : info.latest || "—";
  const badge = info.available
    ? `<span class="pill ok">${esc(t("updateAvailable", { version: info.latest }))}</span>`
    : info.error
      ? `<span class="pill warn">${esc(t("update"))}</span>`
      : `<span class="pill ok">${esc(t("upToDate"))}</span>`;
  const body = `
    <div style="display:flex;align-items:center;gap:12px;margin-bottom:16px">
      <img src="assets/logo.png" alt="VEXORA" style="width:46px;height:46px;border-radius:12px" />
      <div><strong style="letter-spacing:.14em">VEXORA</strong>
      <div class="tiny muted">${esc(t("app.tagline"))}</div></div>
    </div>
    <div style="margin-bottom:12px">${badge}</div>
    <div class="kv"><span>${esc(t("currentVersion"))}</span><span>${esc(info.current)}</span></div>
    <div class="kv"><span>${esc(t("latestVersion"))}</span><span>${esc(latest)}</span></div>
    ${
      info.notes
        ? `<div style="margin-top:14px"><div class="tiny muted">${esc(t("updateNotes"))}</div>
           <p style="line-height:1.8;margin:6px 0">${esc(info.notes)}</p></div>`
        : ""
    }
    ${
      info.error && !info.available
        ? `<div class="tiny muted" style="margin-top:14px;line-height:1.8">${esc(info.error)}<br/>${esc(t("updateNoManifest"))}</div>`
        : ""
    }
    <div class="tiny muted" style="margin-top:16px;display:flex;align-items:center;gap:6px">${svg("shield", 14)}${esc(t("shaVerified"))}</div>`;
  const buttons = [{ label: t("close"), cls: "ghost" }];
  if (info.available) {
    buttons.unshift({
      label: t("updateNow"),
      cls: "primary",
      onClick: async () => {
        try {
          await call("install_update", { url: info.url, sha256: info.sha256 }, t("downloading"));
          toast(t("updateStarted"));
          setTimeout(() => invoke("quit_app").catch(() => {}), 1600);
        } catch (e) {
          toast(`${t("updateFailed")}: ${e}`, "error");
        }
      },
    });
  }
  modal({ title: `VEXORA — ${t("update")}`, body, buttons });
}

/* -------------------------------------------------------------------- shell */
function applyShell() {
  document.documentElement.setAttribute("dir", state.lang === "ar" ? "rtl" : "ltr");
  document.documentElement.setAttribute("lang", state.lang);
  document.documentElement.setAttribute("data-theme", state.theme);
  $$("[data-i18n]").forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });
  renderNav();
}

async function loadCatalog() {
  state.catalog = await call("get_catalog");
  state.info = await call("app_info");
  if (!state.selected.size) {
    defaultCleanerIds().forEach((id) => state.selected.add(id));
  }
}

async function boot() {
  try {
    state.settings = await call("get_settings");
    state.lang = state.settings.language || "ar";
    state.theme = state.settings.theme || "dark";
    const pages = new Set(NAV.map(([id]) => id));
    state.page = pages.has(state.settings.lastPage) ? state.settings.lastPage : "dashboard";
    vexApplyAppearance();
    applyShell();
    await loadCatalog();
    const versionEl = $("#appVersion");
    if (versionEl) versionEl.textContent = `v${state.info.version}`;
    if ($("#updateBtn")) $("#updateBtn").onclick = () => showUpdate();
    if ($("#vexVersionUpdate")) {
      $("#vexVersionUpdate").title = t("checkUpdate");
      $("#vexVersionUpdate").onclick = () => showUpdate();
    }
    if ($("#vexMyStyle")) {
      $("#vexMyStyle").title = t("myStyle");
      $("#vexMyStyle").onclick = () => openAppearanceStudio();
      vexUpdateMyStyleLabel();
    }
    if ($("#appearanceBtn")) {
      $("#appearanceBtn").title = t("appearance");
      $("#appearanceBtn").onclick = () => openAppearanceStudio();
    }
    if ($("#vexUpdateDock")) {
      $("#vexUpdateDock").title = t("checkUpdate");
      $("#vexUpdateDock").onclick = () => showUpdate();
    }
    await go(state.page);
  } catch (e) {
    document.getElementById("view").innerHTML =
      `<div class="empty">${svg("alert", 44)}<div>${esc(String(e))}</div></div>`;
  }

  $("#themeBtn").onclick = async () => {
    state.theme = state.theme === "dark" ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", state.theme);
    state.settings = await call("save_settings", {
      settings: { ...state.settings, theme: state.theme, appearanceMode: state.theme },
    });
    vexApplyAppearance();
    applyShell();
  };
  $("#langBtn").onclick = async () => {
    state.lang = state.lang === "ar" ? "en" : "ar";
    state.settings = await call("save_settings", { settings: { ...state.settings, language: state.lang } });
    applyShell();
    vexUpdateMyStyleLabel();
    await loadCatalog();
    await go(state.page);
  };

  // New GitHub release found shortly after startup.
  await listen("update:available", (ev) => {
    if ($("#vexUpdateDock")) $("#vexUpdateDock").classList.add("show");
    toastUpdate(ev.payload);
  });

  // Navigation requested from the floating bubble.
  await listen("widget:navigate", (ev) => {
    const p = ev.payload;
    const allowed = ["dashboard", "clean", "tune", "registry", "startup",
      "bgprograms", "uninstall", "tools", "history", "settings"];
    if (typeof p === "string" && allowed.includes(p)) go(p).catch(() => {});
  });

  await listen("repair:progress", (ev) => {
    if (!state.tools.repair) state.tools.repair = { running: true, stage: "", percent: -1, log: "", result: null };
    state.tools.repair.running = true;
    state.tools.repair.stage = ev.payload.stage;
    if (typeof ev.payload.percent === "number" && ev.payload.percent >= 0) state.tools.repair.percent = ev.payload.percent;
    if (ev.payload.line) state.tools.repair.log = (state.tools.repair.log || "") + ev.payload.line + "\n";
    if (state.page === "tools" && state.tools.tab === "sysrepair") {
      const bar = $("#view .progress > i");
      if (bar) bar.style.width = Math.max(0, state.tools.repair.percent) + "%";
    }
  });
  await listen("repair:done", (ev) => {
    if (!state.tools.repair) state.tools.repair = {};
    state.tools.repair.running = false;
    if (ev.payload) {
      state.tools.repair.result = ev.payload.summary;
      if (ev.payload.log) state.tools.repair.log = ev.payload.log;
    }
    if (state.page === "tools" && state.tools.tab === "sysrepair") go("tools");
  });

  await listen("dup-progress", (ev) => {
    state.tools.dupScanned = ev.payload;
    if (state.page === "tools" && state.tools.tab === "dupes") {
      const el = $("#dupProg");
      if (el) el.textContent = `${ev.payload} ${state.lang === "ar" ? "ملف تم فحصه…" : "files scanned…"}`;
    }
  });

  await listen("wipe:progress", (ev) => {
    if (!state.tools.wipe) return;
    state.tools.wipe.written = ev.payload.written;
    state.tools.wipe.total = ev.payload.total;
    if (state.page === "tools" && state.tools.tab === "wipe") {
      const bar = $("#view .progress > i");
      if (bar) bar.style.width = Math.min(100, (ev.payload.written / Math.max(1, ev.payload.total)) * 100) + "%";
      const txt = $("#view .tiny.muted");
      if (txt) txt.textContent = `${t("wipeRunning")} ${fmtBytes(ev.payload.written)} / ${fmtBytes(ev.payload.total)}`;
    }
  });
}

window.addEventListener("unhandledrejection", (e) => {
  console.error("unhandledrejection", e.reason);
  try { toast(friendlyError(e.reason), "error"); } catch (_) {}
});
document.addEventListener("DOMContentLoaded", boot);
