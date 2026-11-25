# VEXORA 1.5.0 — Trust & Security Update · تقرير التنفيذ

**التاريخ:** 2026-11-26
**الفرع:** `main` · **الإصدار:** 1.5.0 (بعد 1.4.9 الذي كان مستخدماً بالفعل)
**المراحل:** S1–S8، كل مرحلة بـ commit مستقل حيث أمكن.

---

## 1. المشكلات المكتشفة (من مراجعة الكود)

| # | المشكلة | الحالة |
|---|---|---|
| 1 | README يعطي أمر تعطيل حماية Windows `VerifiedAndReputablePolicyState=0` | اكتُشفت وأُزيلت |
| 2 | README الرئيسي باسم قديم و`package.json` بإصدار 1.1.0 | اكتُشفت وحُلّت |
| 3 | Token مخزّن بـ DPAPI في ملف — ليس Credential Manager، وغير ضروري | اكتُشف وأُزيل بالكامل |
| 4 | ملف التحديث المؤقت باسم متوقع → خطر TOCTOU محلي | أُصلح |
| 5 | لا يوجد allowlist لنطاق رابط التنزيل (إعدادات مستوردة) | أُصلح |
| 6 | فحص التوقيع يكتفي بـ `contains("VEXORA")` | أُصلح (مطابقة تامة للناشر) |
| 7 | سكربت إعادة الفتح `.bat` يزيل علامات التنصيص من المسارات | أُصلح |
| 8 | `cargo deny` غير موجود، ونتائج audit غير مراجعة | أُضيفت ورُوجعت |
| 9 | `clean.rs` يعتمد على تحقق المتصل فقط للـ reparse points | أُصلح (فحص داخلي) |
| 10 | `validate_path_for_read()` موجود لكن غير مستخدم | سجل كخطر متبقٍ |
| 11 | HyperScan تجريبي ما زال قابلاً للوصول عبر command | أُخفي وحُذف |
| 12 | لا تحذير SSD في Shredder | أُضيف |
| 13 | لا panic log محلي | أُضيف مع تنقيح الخصوصية |
| 14 | اختبارات تكامل للحذف/الاستعادة/النسخ الاحتياطي غير موجودة | أُضيفت |
| 15 | Junctions تُكتشف نظرياً لكن لم يُختبرا عملياً | أُضيف اختبار حقيقي بـ mklink /J |

---

## 2. ما أُصلح بالتفصيل

### S1 — التوثيق والإصدار
- حذف تعليمات تعطيل أي حماية Windows؛ استبدالها بتنبيه "لا تُعطَّل حماية Windows".
- `SECURITY.md` أُعيد كتابته بوصف حقيقي للنموذج الحالي.
- توحيد 1.5.0 في Cargo/tauri.conf/package.json/README، وإضافة LICENSE.

### S2 — تأمين التحديث
- `allowed_download_url()`: HTTPS فقط، ونطاقات GitHub المعتمدة فقط (`github.com`, `objects.githubusercontent.com`, `release-assets.githubusercontent.com`).
- اسم ملف مؤقت عشوائي + `create_new(true)` لمنع TOCTOU.
- مطابقة تامة لناشر التوقيع `CN=VEXORA, O=Ramadan Fadel Mostafa`.
- اقتباس آمن للمسارات في سكربت إعادة الفتح بعد التحديث.

### S3 — إزالة Token
- حقل `github_token`، واجهة الإعدادات، وملف DPAPI أُزيلوا بالكامل.
- تنظيف أي ملف قديم عند التحميل. الكاش 5 دقائق يبقي الطلبات أقل بكثير من حد GitHub.

### S4 — سلسلة التوريد
- `deny.toml` مع سياسة تراخيص صريحة، و`cargo deny check licenses bans sources` في CI.
- `cargo audit` مع 11 advisory مقبولة وموثّقة في `src-tauri/audit.toml` (كلها تبعيات Tauri غير المستخدمة في مساراتنا).
- Dependabot لـ Cargo/npm/GitHub Actions.
- `scripts/release-artifacts.ps1` أنتج فعلياً `SHA256SUMS.txt` و`sbom-cargo.json` و`sbom-npm.json`.
- `npm audit`: 0 ثغرات. `cargo audit` بعد الاستثناءات: 0 ثغرات + 7 تحذيرات غير حرجة.
- `cargo deny`: licenses/bans/sources = OK.

### S5 — الخصوصية
- `panic::set_hook` يكتب محلياً في `log.txt` فقط، مع تنقيح `%USERPROFILE%` و`%LOCALAPPDATA%` وأي `ghp_...`، بلا أي مخرج شبكي.
- اختبار وحدة للتنقيح.

### S6 — اختبارات تكامل حقيقية
- دمج تصدير `.reg` متعدد في ملف واحد قابل للاستعادة (اختبار فعلي).
- إنشاء junction حقيقي بـ `mklink /J` والتحقق أن `validate_path_for_delete` يرفضه.
- حماية إضافية داخل `clean.rs` تمنع أي reparse point حتى لو تجاوز المتصل التحقق.

### S7 — إخفاء غير المكتمل
- حذف وحدة HyperScan والأمر المرتبط بها؛ `dup_scan` يستخدم المحرك المكتمل فقط.
- حذف dependency `czkawka_core`.
- تحذير SSD صريح: Wear Leveling والمنطقة المحجوزة، ولا عدة عبورات ولا TRIM يضمنان المحو.
- تحديث تعليق publisher في `boost.rs` ليوضح أنه غير معروض.

### S8 — التقرير
هذا المستند + بناء المثبّت + مزامنة النسخة النظيفة.

---

## 3. الملفات المعدّلة (أهمها)
```
README.md, SECURITY.md, LICENSE, package.json, .gitignore
.github/workflows/ci.yml, .github/dependabot.yml
deny.toml, scripts/release-artifacts.ps1
src-tauri/Cargo.toml, tauri.conf.json, audit.toml
src-tauri/src/{update.rs, settings.rs, util.rs, lib.rs, main.rs, clean.rs,
              registry.rs, safety.rs, boost.rs}
src-tauri/src/hyperscan_adapter.rs  (حُذف)
src/app.js
tests/ui.smoke.test.js
VEXORA-Gaps-and-Competitive-Plan.md, VEXORA-1.5.0-Trust-Report.md
```

---

## 4. الاختبارات الفعلية ونتائجها

| الفحص | النتيجة |
|---|---|
| `cargo check` | ناجح بعد إزالة `czkawka_core` |
| `cargo test --lib` | 52 ناجحاً، 0 فشل، 1 متجاهل |
| `cargo test --bins` | 1 ناجح (تنقيح سجل الأعطال) |
| `npm run test:ui` | PASS — تنقل، استوديو، معاينة، أزرار، بلا أخطاء JS |
| `node --check src/app.js` | ناجح |
| `npm audit --audit-level=high` | 0 ثغرات |
| `cargo audit` (بعد الاستثناءات الموثقة) | 0 ثغرات + 7 تحذيرات |
| `cargo deny check licenses bans sources` | OK |
| `scripts/release-artifacts.ps1` | أنتج SHA256SUMS + SBOM فعلياً |

---

## 5. المخاطر المتبقية (بصراحة)

1. **التوقيع ليس موثوقاً** — شهادة خاصة؛ SmartScreen قد يحذّر. لم تشترَ شهادة OV/EV (قرارك).
2. **CI لم يعمل على GitHub بعد** — لا push حتى الآن. Windows Server runner ≠ Win10/11 حقيقي.
3. **11 advisory مقبولة** في تبعيات Tauri (zip/quick-xml/ruzstd) — مسارات لا نستخدمها مباشرة، لكنها تبقى في الشجرة.
4. **`validate_path_for_read()` غير مطبَّق** على عمليات القراءة؛ حماية قراءة مركزية غير مكتملة.
5. **لا مراجعة بصرية** للتصاميم الأربعة والفقاعة من طرفي.
6. **لا اختبار Windows 10 حقيقي** — اكتشاف الإصدار مُختبر بوحدة تحاكي Build 19045 فقط.
7. تحذيرات `cargo audit` (unmaintained crates) معلومة وغير حرجة حالياً.

---

## 6. ما يحتاج اختباراً يدوياً منك

- تثبيت 1.5.0 على Windows 11 وعلى Windows 10 إن توفر.
- فتح الشريط → زر المظهر → تبديل التصاميم/الألوان/الوضع → Apply → إعادة الفتح.
- الفقاعة: ضغطة تنظيف واحدة، بدون أخطاء.
- نشر Release `v1.4.10` أو أعلى ثم فحص الإشعار وإعادة الفتح التلقائي بعد التحديث.
- مراقبة سجل `log.txt` بعد أي عطل للتأكد من التنقيح.

---

## 7. جاهزية النشر

**جاهز تقنياً** من ناحية الكود والاختبارات المحلية وال policym.
**ليس جاهزاً بعد للادعاء أمام المنافسين** قبل:
1. push وتفعيل CI،
2. توقيع موثوق،
3. اختبار Windows 10/11 يدوي،
4. نشر SBOM/SHA256SUMS مع الإصدار.

التوصية: انشره كـ **Trust & Security Update** داخلياً/تجريبياً أولاً، ثم اعتبره جاهزاً للجمهور بعد الثلاثة الأولى.

---

*أُعدّ هذا التقرير قبل الـ push النهائي؛ أي فحص لم يُنجز بعد مُوضّح صراحة.*
