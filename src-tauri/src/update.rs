//! Lightweight update checker.
//!
//! The app reads a small `latest.json` manifest that lives next to the
//! executable (or at a URL chosen in settings):
//!
//! ```json
//! { "version": "1.0.1", "url": "https://example.com/VEXORA_1.0.1_x64-setup.exe",
//!   "sha256": "…", "notes": "What changed" }
//! ```
//!
//! Downloads are verified with SHA-256 before the NSIS installer is launched,
//! so a corrupted or tampered file is never executed.

use serde::{Deserialize, Serialize};

use crate::{settings::Settings, util};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub available: bool,
    pub current: String,
    pub latest: String,
    pub url: String,
    pub sha256: String,
    pub notes: String,
    pub source: String,
    pub error: String,
}

fn version_tuple(v: &str) -> Vec<u64> {
    v.trim()
        .trim_start_matches('v')
        .split(|c: char| !c.is_ascii_digit())
        .filter(|p| !p.is_empty())
        .map(|p| p.parse::<u64>().unwrap_or(0))
        .collect()
}

fn is_newer(latest: &str, current: &str) -> bool {
    let a = version_tuple(latest);
    let b = version_tuple(current);
    for i in 0..a.len().max(b.len()) {
        let x = *a.get(i).unwrap_or(&0);
        let y = *b.get(i).unwrap_or(&0);
        if x != y {
            return x > y;
        }
    }
    false
}

fn read_manifest(source: &str) -> Result<String, String> {
    if source.starts_with("http://") {
        // Refuse plaintext HTTP for remote manifests.
        if !source.starts_with("http://localhost") && !source.starts_with("http://127.0.0.1") {
            return Err("insecure HTTP manifest refused — use HTTPS".into());
        }
    }
    if source.starts_with("http://") || source.starts_with("https://") {
        let safe = source.replace('\'', "''");
        let script = format!(
            "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; \
             $ProgressPreference='SilentlyContinue'; \
             (Invoke-WebRequest -UseBasicParsing -Uri '{safe}' -TimeoutSec 20).Content"
        );
        util::run_and_wait(
            "powershell",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script.as_str(),
            ],
        )
    } else {
        std::fs::read_to_string(source).map_err(|e| e.to_string())
    }
}

fn field(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

/// Check the local / configured manifest for a newer version. Never fails:
/// errors are reported in `UpdateInfo::error` so the UI can show a message.
fn manifest_check() -> UpdateInfo {
    let s = Settings::load();
    let current = env!("CARGO_PKG_VERSION").to_string();

    let source = if !s.update_url.trim().is_empty() {
        s.update_url.trim().to_string()
    } else {
        let dir = std::path::Path::new(&util::exe_path())
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        dir.join("latest.json").to_string_lossy().to_string()
    };

    let mut info = UpdateInfo {
        available: false,
        current: current.clone(),
        latest: current.clone(),
        url: String::new(),
        sha256: String::new(),
        notes: String::new(),
        source: source.clone(),
        error: String::new(),
    };

    let text = match read_manifest(&source) {
        Ok(t) => t,
        Err(e) => {
            info.error = e;
            return info;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(text.trim()) {
        Ok(v) => v,
        Err(e) => {
            info.error = format!("invalid manifest: {e}");
            return info;
        }
    };

    info.latest = field(&parsed, "version");
    info.url = field(&parsed, "url");
    info.sha256 = field(&parsed, "sha256");
    info.notes = field(&parsed, "notes");
    if info.latest.is_empty() {
        info.error = "manifest has no version".into();
        return info;
    }
    let newer = is_newer(&info.latest, &current);
    info.available = newer && !info.url.is_empty() && !info.sha256.is_empty();
    if newer && info.url.is_empty() {
        info.error = "manifest has no download url".into();
    } else if newer && info.sha256.is_empty() {
        info.error = "manifest has no sha256 — update refused".into();
    }
    info
}

/// Extract a comparable version from any release tag, e.g.
/// `VEXORA_1.3.7_x64` -> `1.3.8`, `v2.0.1-beta` -> `2.0.1`.
fn extract_version(tag: &str) -> String {
    let b = tag.as_bytes();
    let start = b.iter().position(|c| c.is_ascii_digit()).unwrap_or(b.len());
    let mut end = start;
    while end < b.len() {
        if b[end].is_ascii_digit() {
            end += 1;
        } else if b[end] == b'.'
            && end > start
            && end + 1 < b.len()
            && b[end + 1].is_ascii_digit()
        {
            end += 1;
        } else {
            break;
        }
    }
    tag[start..end].to_string()
}

/// Prefer whichever version is higher — the release tag or the installer
/// filename. This tolerates a release whose tag was not renamed after a new
/// installer was uploaded.
fn newer_of(a: &str, b: &str) -> String {
    if b.is_empty() {
        return a.to_string();
    }
    if is_newer(b, a) {
        b.to_string()
    } else {
        a.to_string()
    }
}

/// Only HTTPS URLs hosted by GitHub are accepted as update sources. An
/// imported settings file must not be able to redirect an update to an
/// arbitrary host.
fn allowed_download_url(url: &str) -> bool {
    let u = url.trim();
    if !u.starts_with("https://") {
        return false;
    }
    let rest = &u[8..];
    let host = rest.split('/').next().unwrap_or("").to_lowercase();
    matches!(
        host.as_str(),
        "github.com"
            | "www.github.com"
            | "objects.githubusercontent.com"
            | "release-assets.githubusercontent.com"
    )
}

/// Pick the installer asset from a GitHub release. Any asset name is accepted
/// as long as it ends with `.exe`; a `sha256:` digest is mandatory.
/// Returns (download url, sha256, version extracted from the filename).
fn github_asset(release: &serde_json::Value) -> (String, String, String) {
    let mut fallback = (String::new(), String::new(), String::new());
    let assets = match release.get("assets").and_then(|x| x.as_array()) {
        Some(a) => a,
        None => return fallback,
    };
    for asset in assets {
        let name = field(asset, "name");
        if !name.to_lowercase().ends_with(".exe") {
            continue;
        }
        let url = field(asset, "browser_download_url");
        if url.is_empty() || !allowed_download_url(&url) {
            continue;
        }
        let digest = field(asset, "digest");
        let sha = digest
            .strip_prefix("sha256:")
            .or_else(|| digest.strip_prefix("SHA256:"))
            .unwrap_or("")
            .trim()
            .to_string();
        let asset_version = extract_version(&name);
        if sha.len() == 64 {
            return (url, sha, asset_version);
        }
        if fallback.0.is_empty() {
            fallback = (url, String::new(), asset_version);
        }
    }
    fallback
}

/// Query `https://api.github.com/repos/{owner}/{repo}/releases/latest`.
/// The latest release is accepted regardless of its tag or asset name.
#[derive(Serialize, Deserialize, Clone)]
struct ReleaseCache {
    repo: String,
    fetched: u64,
    body: String,
}

fn release_cache_path() -> std::path::PathBuf {
    let dir = std::path::Path::new(&util::local_appdata()).join("VEXORA");
    dir.join("github-release-cache.json")
}

fn read_release_cache(repo: &str) -> Option<ReleaseCache> {
    let text = std::fs::read_to_string(release_cache_path()).ok()?;
    let c: ReleaseCache = serde_json::from_str(&text).ok()?;
    if c.repo == repo && !c.body.is_empty() {
        Some(c)
    } else {
        None
    }
}

fn write_release_cache(repo: &str, body: &str) {
    let path = release_cache_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let c = ReleaseCache {
        repo: repo.to_string(),
        fetched: util::now_unix(),
        body: body.to_string(),
    };
    if let Ok(text) = serde_json::to_string(&c) {
        let _ = std::fs::write(path, text);
    }
}

pub fn github_check(repo: &str, force: bool) -> UpdateInfo {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let repo = crate::settings::normalize_github_repo(repo);
    let mut info = UpdateInfo {
        available: false,
        current: current.clone(),
        latest: current.clone(),
        url: String::new(),
        sha256: String::new(),
        notes: String::new(),
        source: format!("GitHub: {repo}"),
        error: String::new(),
    };
    // GitHub's unauthenticated API allows only 60 requests/hour per IP. Reuse a
    // recent cached response so opening the app and pressing "check" do not
    // burn the quota; a stale copy is still better than no update at all.
    let cached = read_release_cache(&repo);
    let fresh = cached
        .as_ref()
        .filter(|c| !force && util::now_unix().saturating_sub(c.fetched) < 300)
        .cloned();
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let script = format!(
        "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; \
         $ProgressPreference='SilentlyContinue'; \
         $h=@{{'User-Agent'='VEXORA-Updater'; 'Accept'='application/vnd.github+json'}}; \
         (Invoke-WebRequest -UseBasicParsing -Headers $h -Uri '{}' -TimeoutSec 20).Content",
        ps_literal(&url)
    );
    let (text, stale) = if let Some(c) = fresh {
        (c.body, false)
    } else {
    match util::run_and_wait(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script.as_str(),
        ],
    ) {
        Ok(t) => {
            write_release_cache(&repo, t.trim());
            (t, false)
        }
        Err(e) => {
            if let Some(c) = cached {
                (c.body, true)
            } else {
                info.error = format!("GitHub request failed: {e}");
                return info;
            }
        }
    }
    };
    if stale {
        info.notes = "GitHub API unavailable — showing cached release info.".into();
    }
    let parsed: serde_json::Value = match serde_json::from_str(text.trim()) {
        Ok(v) => v,
        Err(e) => {
            info.error = format!("invalid GitHub response: {e}");
            return info;
        }
    };
    let tag = field(&parsed, "tag_name");
    let tag_version = extract_version(&tag);
    let mut notes = field(&parsed, "body");
    if stale {
        notes = format!("GitHub API unavailable — cached release info.\n{notes}");
    }
    info.notes = notes;
    let (asset_url, sha, asset_version) = github_asset(&parsed);
    info.latest = newer_of(&tag_version, &asset_version);
    if info.latest.is_empty() {
        info.error = "GitHub release has no version in tag or installer name".into();
        return info;
    }
    info.url = asset_url;
    info.sha256 = sha.clone();
    let newer = is_newer(&info.latest, &current);
    info.available = newer && !info.url.is_empty() && !info.sha256.is_empty();
    if newer && info.url.is_empty() {
        info.error = "GitHub release has no .exe installer".into();
    } else if newer && sha.is_empty() {
        info.error = "GitHub release asset has no sha256 digest — update refused".into();
    }
    info
}

/// Preferred update source: GitHub releases; falls back to the legacy
/// `latest.json` manifest when GitHub is unavailable.
pub fn check() -> UpdateInfo {
    check_with(false)
}

/// `force = true` bypasses the short-lived cache (used by the manual button).
pub fn check_with(force: bool) -> UpdateInfo {
    let s = Settings::load();
    let gh = github_check(&s.github_repo, force);
    if gh.error.is_empty() {
        return gh;
    }
    let mut fallback = manifest_check();
    if fallback.error.is_empty() && !gh.error.is_empty() {
        fallback.notes = format!("GitHub: {}\n{}", gh.error, fallback.notes).trim().to_string();
    }
    fallback
}

fn ps_literal(p: &str) -> String {
    p.replace('\'', "''")
}

/// Verify the installer's Authenticode signature. A file that is unsigned,
/// tampered with, or signed by an unexpected publisher is rejected before it
/// is ever executed — SHA-256 alone cannot prove *who* built the binary.
fn verify_authenticode(path: &std::path::Path) -> Result<(), String> {
    let path_s = path.to_string_lossy().to_string();
    let script = format!(
        "$s = Get-AuthenticodeSignature -LiteralPath '{}'; \
         $subj = if ($s.SignerCertificate) {{ $s.SignerCertificate.Subject }} else {{ '' }}; \
         '{{0}}|{{1}}|{{2}}' -f $s.Status, $s.StatusMessage, $subj",
        ps_literal(&path_s)
    );
    let out = util::run_and_wait(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script.as_str(),
        ],
    )?;
    let out = out.trim();
    let parts: Vec<&str> = out.splitn(3, '|').collect();
    if parts.len() != 3 {
        return Err("Authenticode verification failed — update refused".into());
    }
    signature_decision(parts[0], parts[1], parts[2])
}

/// Pure decision helper (unit-testable).
fn signature_decision(status: &str, message: &str, subject: &str) -> Result<(), String> {
    // VEXORA installers are signed with our own private certificate, whose
    // root is intentionally not in the public trust store. That produces
    // "UnknownError / root certificate not trusted" on a perfectly intact
    // signature. Accept that case, but never accept tampering (HashMismatch)
    // or unsigned files (NotSigned). SHA-256 pinning still guarantees integrity.
    let trusted_chain = status == "Valid";
    let private_root = status == "UnknownError"
        && (message.to_lowercase().contains("root certificate")
            || message.to_lowercase().contains("not trusted"));
    if !trusted_chain && !private_root {
        return Err(format!("Authenticode signature invalid ({status}: {message}) — update refused"));
    }
    const EXPECTED_PUBLISHER: &str = "CN=VEXORA, O=Ramadan Fadel Mostafa";
    if !subject.trim().eq_ignore_ascii_case(EXPECTED_PUBLISHER) {
        return Err(format!("unexpected installer publisher ({subject}) — update refused"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_accepts_valid_and_private_root() {
        assert!(signature_decision(
            "Valid",
            "",
            "CN=VEXORA, O=Ramadan Fadel Mostafa"
        )
        .is_ok());
        assert!(signature_decision(
            "UnknownError",
            "A certificate chain processed, but terminated in a root certificate which is not trusted by the trust provider",
            "CN=VEXORA, O=Ramadan Fadel Mostafa"
        )
        .is_ok());
    }

    #[test]
    fn signature_rejects_tampered_unsigned_and_wrong_publisher() {
        assert!(signature_decision("HashMismatch", "", "CN=VEXORA").is_err());
        assert!(signature_decision("NotSigned", "", "").is_err());
        assert!(signature_decision("Valid", "", "CN=Someone Else").is_err());
        assert!(signature_decision("UnknownError", "some other failure", "CN=VEXORA").is_err());
    }

    #[test]
    fn download_url_allowlist() {
        assert!(allowed_download_url(
            "https://github.com/ramadanfadel/VEXORA-1.4.0-Clean/releases/download/v1.5.0/a.exe"
        ));
        assert!(allowed_download_url(
            "https://objects.githubusercontent.com/a/b.exe"
        ));
        assert!(!allowed_download_url("http://github.com/a.exe"));
        assert!(!allowed_download_url("https://evil.example.com/a.exe"));
        assert!(!allowed_download_url("https://github.com.evil.com/a.exe"));
    }

    #[test]
    fn extracts_version_from_any_tag() {
        assert_eq!(extract_version("VEXORA_1.3.7_x64"), "1.3.7");
        assert_eq!(extract_version("v2.0.1-beta"), "2.0.1");
        assert_eq!(extract_version("1.3.8"), "1.3.8");
        assert_eq!(extract_version("release"), "");
    }

    #[test]
    fn picks_any_exe_asset_with_sha256_digest() {
        let release: serde_json::Value = serde_json::from_str(
            r#"{"assets":[
              {"name":"notes.txt","browser_download_url":"https://github.com/x/notes","digest":"sha256:abc"},
              {"name":"weird-name-installer.EXE","browser_download_url":"https://github.com/x/setup.exe","digest":"sha256:0123456789012345678901234567890123456789012345678901234567890123"}
            ]}"#,
        )
        .unwrap();
        let (url, sha, ver) = github_asset(&release);
        assert_eq!(url, "https://github.com/x/setup.exe");
        assert_eq!(sha.len(), 64);
        assert_eq!(ver, "");
    }

    #[test]
    fn installer_name_can_outrank_stale_tag() {
        assert_eq!(newer_of("1.3.7", "1.3.8"), "1.3.8");
        assert_eq!(newer_of("1.4.0", "1.3.8"), "1.4.0");
        assert_eq!(newer_of("1.3.7", ""), "1.3.7");
    }

    #[test]
    fn refuses_exe_without_digest() {
        let release: serde_json::Value = serde_json::from_str(
            r#"{"assets":[{"name":"setup.exe","browser_download_url":"https://github.com/x/setup.exe","digest":""}]}"#,
        )
        .unwrap();
        let (url, sha, _) = github_asset(&release);
        assert_eq!(url, "https://github.com/x/setup.exe");
        assert!(sha.is_empty());
    }
}

/// Download the installer, verify its SHA-256 and launch it silently.
/// Returns the local path of the installer.
pub fn download_and_run(url: &str, sha256: &str) -> Result<String, String> {
    if !allowed_download_url(url) {
        return Err("update URL must be an HTTPS GitHub URL — refused".into());
    }
    if url.starts_with("http://")
        && !url.starts_with("http://localhost")
        && !url.starts_with("http://127.0.0.1")
    {
        return Err("insecure HTTP download refused — use HTTPS".into());
    }
    // Unpredictable name + atomic create: a local process cannot pre-create
    // the file and redirect what we later execute (TOCTOU).
    let unique = util::now_unix()
        ^ (std::process::id() as u64).wrapping_mul(6364136223846793005)
        ^ std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
    let dest = std::env::temp_dir().join(format!("VEXORA-setup-{unique:016x}.exe"));
    use std::io::Write;
    let mut fh = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&dest)
        .map_err(|e| format!("temp file: {e}"))?;
    let _ = fh.write_all(b"");
    drop(fh);
    let dest_s = dest.to_string_lossy().to_string();

    let remote = url.starts_with("http://") || url.starts_with("https://");
    // Never execute an unverified installer: the SHA-256 is mandatory.
    if sha256.trim().is_empty() {
        let _ = std::fs::remove_file(&dest);
        return Err(if remote {
            "manifest has no sha256 — refusing to run an unverified download".into()
        } else {
            "manifest has no sha256 — refusing to run an unverified installer".into()
        });
    }

    if remote {
        let script = format!(
            "$ProgressPreference='SilentlyContinue'; \
             Invoke-WebRequest -UseBasicParsing -Uri '{}' -OutFile '{}'",
            ps_literal(url),
            ps_literal(&dest_s)
        );
        util::run_and_wait(
            "powershell",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script.as_str(),
            ],
        )?;
    } else {
        std::fs::copy(url, &dest).map_err(|e| format!("copy: {e}"))?;
    }

    if !dest.is_file() {
        return Err("download failed".into());
    }

    {
        let script = format!(
            "(Get-FileHash -Algorithm SHA256 -LiteralPath '{}').Hash",
            ps_literal(&dest_s)
        );
        let hash = util::run_and_wait(
            "powershell",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                script.as_str(),
            ],
        )?;
        if !hash.trim().eq_ignore_ascii_case(sha256.trim()) {
            let _ = std::fs::remove_file(&dest);
            return Err("SHA-256 mismatch — download rejected".into());
        }
    }

    if let Err(e) = verify_authenticode(&dest) {
        let _ = std::fs::remove_file(&dest);
        return Err(e);
    }

    // Run the installer and relaunch VEXORA afterwards. The .bat process is
    // detached from the app: it survives the app quitting during the silent
    // install, waits for the installer to finish, then starts the new exe.
    let exe = util::exe_path();
    let bat = std::env::temp_dir().join(format!("vexora-update-{}.bat", util::now_unix()));
    let script = format!(
        "@echo off\r\n\"{}\" /S\r\nping -n 6 127.0.0.1 >nul\r\nstart \"\" \"{}\"\r\n",
        dest_s,
        exe
    );
    std::fs::write(&bat, script).map_err(|e| format!("relaunch script: {e}"))?;
    util::run_command("cmd", &["/C".to_string(), bat.to_string_lossy().to_string()])
        .map_err(|e| format!("installer: {e}"))?;
    Ok(dest_s)
}
