# VEXORA — Safety & Security Model

**Version:** 1.5.0 · Last updated: 2026-11-26

## 1. Destructive operations
- `forced_uninstall` / `monitor_uninstall` follow a three-step flow:
  `*_plan` (read-only candidates) → UI review with explicit selection → `*_selected`
  (re-validation against a freshly regenerated plan).
- General deletion goes through `safety::validate_path_for_delete`:
  canonicalize, reject traversal / null bytes / reparse points, reject protected
  and personal roots, and require the path to be in the reviewed plan.
- Files are recycled by default; shredding requires an explicit typed `DELETE`
  confirmation. System fonts and protected roots are never deleted.

## 2. Update integrity
- Source: GitHub Releases API over HTTPS only.
- `sha256:` digest from the release asset is mandatory; SHA-256 is verified
  before execution.
- Authenticode signature and publisher are verified before launching.
- A new release is announced with an in-app toast; the user chooses Apply.
- Manual "Check" bypasses the 5-minute cache; background checks are deduped.

## 3. Secrets & privacy
- No telemetry, no automatic upload of diagnostics.
- The optional GitHub token is stored with Windows DPAPI (never in
  `settings.json`), and is planned for removal because the cache keeps API
  usage well below the unauthenticated limit.
- `log.txt` is local, append-only, rotated at 1 MB, with no network calls.

## 4. Least privilege
- Admin-only operations (DISM/SFC, HKCR context menu, pnputil, restore points)
  show a clear badge and fail with a toast when rights are missing.
- Background monitor, widget and battery guard run without Administrator.

## 5. Deliberately not done
- No driver replacement (can brick a machine).
- No disabling Windows Defender, SmartScreen or Smart App Control.
- No aggressive registry sweeps; only backed-up `.reg` exports are removed.

## Reporting a vulnerability
Open a private GitHub Security Advisory on
`ramadanfadel/VEXORA-1.4.0-Clean`. Do not publish exploit details publicly.
