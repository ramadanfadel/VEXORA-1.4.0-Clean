# رفع المشروع على GitHub

المجلد الحالي هو بالفعل نسخة نظيفة جاهزة (بدون `.git`، `node_modules`، `target`، شهادات، أو بيانات مستخدم).

## الطريقة الأسهل (من سطر الأوامر)
```powershell
cd "C:\Users\User\Documents\Default Project\VEXORA-1.4.0-Clean"
git init
git add .
git commit -m "VEXORA 1.4.0 — clean source release"
git branch -M main
git remote add origin https://github.com/ramadanfadel/VEXORA.git
git push -u origin main
```

> إن كان المستودع على GitHub يحتوي بالفعل على ملفات قديمة، استخدم
> `git push -u origin main --force` بعد التأكد من أنك تريد استبدالها.

## أو ارفع ملف الـ ZIP
استخدم `VEXORA-1.4.0-Clean.zip` الموجود بجانب هذا المجلد، ثم في GitHub:
`New repository` → `Upload files` → اسحب محتوى الـ ZIP بعد فكّه.

## بعد الرفع
1. انشر Release جديد بالرقم `VEXORA_1.4.1_x64` (أعلى من 1.4.0) وارفع المثبّت الموقّع.
2. تأكد أن الـ asset يحتوي على `sha256:` digest (GitHub يولّده تلقائياً).
3. افتح VEXORA 1.4.0 → سيظهر إشعار التحديث والزر الصغير بجانب رقم الإصدار.

## توقيع المثبّت من هذه النسخة
أضف تحت `bundle` في `src-tauri/tauri.conf.json`:
```json
"certificateThumbprint": "YOUR_THUMBPRINT",
"digestAlgorithm": "sha256",
"timestampUrl": "http://timestamp.digicert.com"
```
ثم شغّل `.\build.ps1 -Installer`.
