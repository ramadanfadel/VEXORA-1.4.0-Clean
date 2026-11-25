# Changelog

## 1.4.0 — Appearance Studio
- **Phase 1**: appearance settings (`appearanceLayout`, `appearanceAccent`, `appearanceMode`,
  `appearanceGlow`, `appearanceAnimation`, `appearanceMigrated`), live preview, Apply/Reset,
  and the `My Style` shortcut. One-time migration via an independent flag.
- **Phase 2**: Icon Dock layout (icon rail with always-visible labels, selection glow).
- **Phase 3**: Icon Grid layout (responsive card grid with short section descriptions).
- **Phase 4**: Glass Neon layout (translucent panels, ambient glow, rounded cards).
- **Phase 5**: layouts applied to all pages, RTL/LTR support, responsive window sizes,
  glow/animation kill switches.
- Validated with `cargo check`, `node --check` and 43 unit tests.

## 1.3.x — Security & Stability
- Centralized path validation (traversal, null bytes, reparse points, protected roots).
- Settings sanitization on load/save/import.
- HTTPS-only updates + SHA-256 + Authenticode verification.
- GitHub Releases updater with in-app toast and version shortcut beside the version number.
- Graceful cancellation of long operations and a global concurrency lock.
- User-only font deletion; safe Windows detection from `CurrentBuildNumber`.
- Hardware Monitor (WMI + `nvidia-smi`, `N/A` fallback).
- Fixes for the floating widget modal, background-programs navigation and shredder typed confirmation.

## Earlier
- Installer/startup icons, duplicate previews, disk check, SMART/admin status,
  SFC/DISM repair, Print Spooler fixer, Context Menu manager, Adobe cache cleaner,
  drivers/runtimes check, RAM optimizer, Battery Health guard, File Shredder/Unlocker
  and Font Manager.
