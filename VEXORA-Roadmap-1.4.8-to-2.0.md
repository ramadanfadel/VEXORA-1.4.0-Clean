# VEXORA — خارطة طريق 1.4.8 → 2.0 (تحليل وتخطيط فقط)

**الحالة:** خطة معتمدة للمراجعة، **لم يُنفَّذ أي عنصر منها بعد**.
**القيود الحاكمة:** Rust + Tauri v2، التصاميم الأربعة، الألوان الستة، ar RTL / en LTR،
لا حذف لوظيفة مكتملة، لا ميزات وهمية، لا ادعاءات أداء غير مقاسة، الأولوية للتراجع.

---

## 1. التقييم الحالي لكل مشروع

| # | المشروع | الحالة | إعادة استخدام | مخاطرة | أولوية |
|---|---|---|---|---|---|
| 1 | Security & Stability | جزئي (1.5.0 قيد الإكمال) | `safety.rs`, `update.rs`, audit/deny, CI | `validate_path_for_read` غير مطبَّق | P0 |
| 2 | Rescue Center | جزئي | `restore.rs`, `registry::backup`, `quarantine_restore/empty`, `history.rs` | استعادة خاطئة / صلاحيات | P0 |
| 3 | HyperScan Engine | غير موجود (حُذف تجريبياً) | `tools::find_duplicates`, `scan.rs` | نتائج وهمية، RAM/CPU | P3 أو إلغاء |
| 4 | Software Updater | جزئي | `update.rs`, `winget_install`, `uninstall::list` | parsing مخرجات winget | P2 |
| 5 | Installation Tracker | موجود | `monitor.rs` snapshot→diff→plan | عرض فقط | P1 |
| 6 | Smart File Transfer | غير موجود | `safety::validate_path_*` | نسخ جزئي، reparse | P4 |
| 7 | Startup Analyzer | جزئي | `startup.rs`, `services.rs`, extras | تعطيل عنصر حرج | P2 |
| 8 | Hardware Center | جزئي | `hwmon.rs`, `sysinfo.rs`, `battery.rs` | مستشعرات N/A | P2 |
| 9 | Performance Lab | جزئي | `ramopt.rs`, `boost.rs`, `tune.rs` | ادعاءات غير مقاسة | P3 |
| 10 | Designer Care | جزئي | `adobe_cache_items`, `fonts.rs`, similar images | تكرار مع منظف الكاش | P3 |
| 11 | Network Manager | غير موجود | `flush_dns`, net stats فقط | **قطع الاتصال** | P4 (قراءة فقط) |
| 12 | Privacy Center | جزئي | `browsers.rs`, catalog cleaners | تكرار صفحة التنظيف | P2 |
| 13 | Advanced Font Manager | موجود | `fonts.rs` | إضافة كشف مكررات فقط | P3 |
| 14 | Registry Diff Viewer | جزئي | `registry.rs` + diff في `monitor.rs` | تحليل .reg عشوائي | P1 |
| 15 | Smart Automation | جزئي | `schedule.rs`, `bgmonitor` | حذف تلقائي | P3 (dry-run افتراضياً) |
| 16 | Smart Doctor | جزئي | `sysinfo::health`, `repair.rs`, `hwmon` | رقم صحة وهمي | P2 |
| 17 | Portable Edition | غير موجود | النسخة النظيفة الحالية | مسار البيانات/التحديث | P4 |

---

## 2. إصدارات قابلة للقياس

| الإصدار | المحتوى | شرط النجاح |
|---|---|---|
| 1.5.0 | Trust & Security (جارٍ) | CI أخضر، audit/deny ناجحان، 52+ اختبار، بلا أرقام أداء وهمية |
| 1.6.0 | Rescue Center + Registry Diff + Installation Tracker UI | كل عملية قابلة للاستعادة لها undo مُختبر؛ .reg round-trip |
| 1.7.0 | Hardware Center + Startup Analyzer + Privacy Center + Smart Doctor | لا رقم صحة بدون سبب؛ تصنيف startup قابل للتفسير |
| 1.8.0 | Software Updater + Designer Care + Font Manager | winget مُختبر؛ لا حذف خطوط نظام |
| 1.9.0 | HyperScan حقيقي + Performance Lab + Smart Automation | benchmark منشور؛ automation بـ dry-run |
| 2.0.0 | Network Manager (قراءة فقط) + Smart File Transfer + Portable | نسخ محقق بـ hash؛ لا تغيير شبكي تلقائي |

---

## 3. تكاليف وخدمات
- **إلزامي:** صفر خدمات مدفوعة. winget/netsh/ipconfig مدمجة في Windows.
- **اختياري:** شهادة OV/EV (قرار منفصل، لم تشترَ).
- GitHub Actions وDependabot مجانيان. لا crash reporting خارجي.

---

## 4. تعارضات يجب تجنّبها
1. Rescue Center مقابل `quarantine_restore` → توحيد لا تكرار.
2. Privacy / Designer مقابل صفحة Clean → واجهة تجميعية فقط.
3. Smart Automation مقابل مبدأ "لا حذف دون مراجعة" → dry-run افتراضياً.
4. Network Manager مقابل `GENERIC_TOKENS` → لا حذف شبكي عبر محرك التنظيف.
5. Portable مقابل المُحدِّث → تفرّع `exe_path()` و`data_dir`.

---

## 5. اختبارات إثبات (لا عدّ شكلي)
- `.reg` round-trip في بيئة معزولة + مقارنة سطرية.
- junction حقيقي بـ `mklink /J` ثم محاولة حذف → مرفوض (تم في 1.5.0).
- نسخ ملف مع hash قبل/بعد + انقطاع واستئناف (File Transfer).
- HyperScan: مجموعة مكررات معروفة + زمن/ذاكرة مُقاسان على جهاز موثق.
- Startup: تصنيف `ctfmon/dwm/lsass` كعناصر حرجة غير قابلة للتعطيل.

---

## 6. الخلاصة
ما يستحق التنفيذ: Rescue Center، Registry Diff، Installation Tracker، Hardware Center،
Startup Analyzer، Privacy Center، Software Updater، Smart Doctor.
ما يجب تأجيله/إلغاؤه: HyperScan الوهمي، Network Manager التغييري، Smart File Transfer،
Smart Doctor كرقم بلا تفسير.

*لا يُبدأ أي عنصر قبل إغلاق 1.5.0 وصدور موافقة صريحة.*
