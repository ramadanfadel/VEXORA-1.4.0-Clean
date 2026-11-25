# VEXORA — Clean Smarter. Run Faster.

**Version:** 1.4.4  
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
- Secure updates from GitHub Releases (`ramadanfadel/VEXORA-1.4.0-Clean`): SHA-256 + Authenticode verification, in-app toast
- Floating smart bubble with quick clean and live stats
- **Appearance Studio**: Classic / Icon Dock / Icon Grid / Glass Neon layouts, 6 accent colors, dark/light modes, live preview, `My Style` shortcut

## Repository layout
```
src/                  Web UI (app.js, index.html, styles.css, bubble.html, widget.html)
src-tauri/src/        Rust backend (Tauri commands, cleaning, safety, settings, updater…)
src-tauri/icons/      App icons
src-tauri/capabilities/ Tauri permissions
assets/               Installer images and source logo
build.ps1             Build script (release exe / NSIS installer)
```

## Build
```powershell
npm install
.\build.ps1             # release exe
.\build.ps1 -Installer  # NSIS installer
```

### Signing (optional)
This public source copy intentionally **does not include** any certificate, private key,
or machine-specific signing thumbprint. To produce signed installers:

1. Install your code-signing certificate in the Windows certificate store.
2. Add `certificateThumbprint`, `digestAlgorithm` and `timestampUrl` under
   `bundle` in `src-tauri/tauri.conf.json`.
3. Rebuild with `.\build.ps1 -Installer`.

For the updater, publish a GitHub Release whose asset has a `sha256:` digest
(GitHub generates this automatically) and is signed with your certificate.

## Safety notes
- Windows detected from `CurrentBuildNumber` (build ≥ 22000 → Windows 11).
- Protected/personal paths, reparse points and traversal are rejected before deletion.
- Registry cleanup creates `.reg` backups before changes.
- Long-running operations use a global lock and graceful cancel on app close.

## License / credits
Developed by Ramadan Fadel Mostafa (Abu Maryam).
