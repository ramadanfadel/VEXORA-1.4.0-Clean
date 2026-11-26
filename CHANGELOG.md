# Changelog

## 1.6.0
- Rescue Center: restore points, quarantine batches and registry backup list
  in one place, with explicit restore / empty actions.
- Registry backup viewer: read-only inspection of `.reg` exports with path-safe
  filename validation.
- Installation Tracker UI reusing the existing `monitor` logs and removal plans.
- 54 unit tests + UI smoke test.

## 1.5.0 — Trust & Security Update
- Removed all Windows-protection-disable guidance; unified version numbers and MIT license.
- Update hardening: GitHub-only HTTPS URLs, unique temp file, exact publisher check, safe relaunch.
- Removed the GitHub token feature entirely.
- cargo-audit + cargo-deny + Dependabot; documented accepted advisories; SBOM/SHA256SUMS generator.
- Local privacy-safe panic log; integration tests for registry backup, junctions and reparse guards.
- HyperScan placeholder module removed; SSD warning added to Shredder.
- 52 unit tests + UI smoke test. See VEXORA-1.5.0-Trust-Report.md.

## 1.4.9
- Trust phase: GitHub token moved to Windows DPAPI-encrypted storage (never
  plaintext in settings.json); Dependabot + `cargo audit` + `npm audit` in CI;
  SBOM and SHA256SUMS generator; version consistency check; unified docs.

## 1.4.8
- Removed the duplicate My Style button beside the version number. The single
  Appearance Studio shortcut in the toolbar (next to theme/language/update) remains.

## 1.4.7
- Manual "Check for updates" now bypasses the cache; background cache reduced
  to 5 minutes. A newly published GitHub release is detected immediately.

## 1.4.6
- Small Appearance Studio shortcut next to the theme/language/update buttons.
- Updater relaunches VEXORA automatically after a silent install.
- Periodic update checks while the app is running (every 10 minutes), with
  per-version dedupe so the notification appears immediately once.

## 1.4.5
- Fixed Appearance Studio live preview: layout buttons no longer bubble to
  `body[data-vex-layout]`, which was resetting the draft to `grid`.
  Verified in a real Chromium browser and in the UI smoke test.

## 1.4.4
- Real install-source detection: MSI provider via GUID/Installer keys or
  msiexec, Store via PackageFullName/WindowsApps, else machine/user hive.
  Shown in the program details dialog. 49 tests pass.

## 1.4.3
- UI smoke test (`npm run test:ui`) using a mocked Tauri bridge — catches
  broken handlers and runtime errors.
- GitHub Actions CI: `npm ci`, JS syntax check, UI smoke test, `cargo check`,
  `cargo test`, and a secret-file guard.
- Updater caches release data for 30 minutes and supports an optional GitHub
  token to avoid the 60 requests/hour unauthenticated API limit.
- Arabic release notes decode correctly as UTF-8.

## 1.4.2
- Faster cleaning: up to four cleaners in parallel, and no VSS restore point
  when all selected work is reversible via the Recycle Bin.
- Floating bubble: one click cleans immediately (no second confirmation).
- Appearance Studio: only CSS variables/classes update on colour/mode changes.
- UTF-8 decoding for GitHub release notes written in Arabic.
- Added `VEXORA-Gaps-Report.md` with the full gap analysis. 47 tests pass.

## 1.4.1
- Uninstaller now reports real install type (Win32 / MSI / Store / Portable),
  measured on-disk size from `InstallLocation` (no junctions followed), and a
  safety rating (safe / review / sensitive).
- Updater source switched to `ramadanfadel/VEXORA-1.4.0-Clean` with automatic
  migration from the old repository. 47 unit tests pass.

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
