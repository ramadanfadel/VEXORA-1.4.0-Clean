# VEXORA — Clean Smarter. Run Faster.

**Version:** 2.1.0 — Stability & Security
**Stack:** Rust + Tauri v2 · vanilla HTML/CSS/JS · Windows x64

All-in-one Windows cleaner and optimizer with Arabic (RTL) and English (LTR) interfaces.

## Features
- Junk / browser cache / registry cleaning with recycle-bin-first deletion and restorable backups
- Startup, uninstall, forced uninstall, monitoring and leftover removal
- Duplicate finder, big files, empty folders, disk map, similar images
- System repair (SFC/DISM), Print Spooler fix, Context Menu manager
- RAM optimizer, Battery Health / Smart Charge Guard
- File Shredder + Restart Manager unlocker, Font Manager
- Adobe/design cache cleaner, drivers & runtimes check (winget)
- Hardware Monitor (WMI + `nvidia-smi`, `N/A` fallback, no admin required)
- Secure updates from GitHub Releases (`ramadanfadel/VEXORA-1.4.0-Clean`): SHA-256 + Authenticode, toast, auto-relaunch
- Floating smart bubble with quick clean and live stats
- Appearance Studio: Classic / Icon Dock / Icon Grid / Glass Neon, 6 accents, dark/light, live preview

## Repository layout
```
src/                  Web UI (app.js, index.html, styles.css, bubble.html, widget.html)
src-tauri/src/        Rust backend (commands, cleaning, safety, settings, updater…)
src-tauri/capabilities/ Tauri permissions
src-tauri/icons/      App icons
assets/               Installer images and source logo
scripts/              Release artifact generator (SHA256SUMS + SBOM)
.github/workflows/    CI
tests/                UI smoke test (mock Tauri bridge)
```

## Build & test
```powershell
npm install
npm run test:ui        # UI smoke test
.\build.ps1             # release exe
.\build.ps1 -Installer  # NSIS installer
.\scripts\release-artifacts.ps1   # SHA256SUMS.txt + SBOM files
```

### Signing (optional)
This public source copy intentionally contains **no certificate, private key, or machine-specific
signing thumbprint**. To produce signed installers, install your certificate in the Windows
certificate store and add `certificateThumbprint`, `digestAlgorithm` and `timestampUrl` under
`bundle` in `src-tauri/tauri.conf.json`, then rebuild with `.\build.ps1 -Installer`.

The GitHub token, if used for update checks, is encrypted with Windows DPAPI and never stored in
`settings.json`.

## Safety notes
- Windows detected from `CurrentBuildNumber` (build ≥ 22000 → Windows 11).
- Protected/personal paths, reparse points and traversal are rejected before deletion.
- Registry cleanup creates `.reg` backups before changes.
- Long-running operations use a global lock and graceful cancel on app close.

## Quality & competition
See `VEXORA-Gaps-and-Competitive-Plan.md` for the honest gap analysis and measurable roadmap.

## License / credits
MIT — developed by Ramadan Fadel Mostafa (Abu Maryam).
