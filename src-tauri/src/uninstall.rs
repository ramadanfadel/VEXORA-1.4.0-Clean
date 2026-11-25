//! Installed-program inventory, uninstall and leftover detection.

use serde::Serialize;

use crate::regutil;
use crate::safety;
use crate::util;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    /// Unique registry key path for this program.
    pub key: String,
    /// Display name of the program.
    pub name: String,
    /// Program version as reported by the installer.
    pub version: String,
    /// Publisher name.
    pub publisher: String,
    /// Install date if available.
    pub install_date: String,
    /// Declared size in KB from registry (may differ from actual).
    pub size_kb: u64,
    /// Uninstall string from registry.
    pub uninstall: String,
    /// Quiet uninstall command if available.
    pub quiet_uninstall: String,
    /// Install location path.
    pub location: String,
    /// Registry hive (machine/user).
    pub hive: String,
    /// Program icon (base64 or path).
    pub display_icon: String,
    /// Type of installation.
    pub install_type: InstallType,
    /// True size of program files measured on disk.
    pub actual_size_kb: Option<u64>,
    /// Level of protection (safe/review/sensitive).
    pub protection_level: ProtectionLevel,
    /// Where the program was discovered.
    pub install_source: InstallSource,
}

/// Level of program protection sensitivity.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProtectionLevel {
    Safe,
    Review,
    Sensitive,
}

/// Source where the program was discovered.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InstallSource {
    MachineUninstallKey,    // من Registry machine uninstall key
    UserUninstallKey,       // من Registry user uninstall key
    MsiProvider,            // من MSI provider
    StoreEntry,             // من Microsoft Store/MSIX
    ManualAdd,              // تم إضافتها يدويًا من قبل المستخدم
    Unknown,                // المصدر غير معروف
}

/// Duplicate the enum for backward compatibility of the `Program` struct field.
/// The actual variant used in `Program` is `InstallSource`; this is kept for
/// any external code that may still reference the old name.
pub type InstallSourceOld = InstallSource;

/// أنواع التثبيت المدعومة.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InstallType {
    Win32,                       // برامج Win32 التقليدية
    Msi,                         // برامج MSI
    Store,                       // Microsoft Store / MSIX
    Portable,                    // تطبيقات Portables
    Unknown,                     // نوع غير معروف
}

fn roots() -> Vec<(&'static str, String)> {
    vec![
        (
            "machine",
            r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall".to_string(),
        ),
        (
            "machine32",
            r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"
                .to_string(),
        ),
        (
            "user",
            r"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall".to_string(),
        ),
    ]
}

pub fn list() -> Vec<Program> {
    let mut out: Vec<Program> = Vec::new();
    let mut seen: Vec<String> = Vec::new();

    for (hive, root) in roots() {
        let key = match regutil::open(&root) {
            Some(k) => k,
            None => continue,
        };
        for sub in key.enum_keys().flatten() {
            let full = format!(r"{root}\{sub}");
            let k = match regutil::open(&full) {
                Some(k) => k,
                None => continue,
            };
            let name: String = k.get_value("DisplayName").unwrap_or_default();
            if name.trim().is_empty() {
                continue;
            }
            if k.get_value::<u32, _>("SystemComponent").unwrap_or(0) == 1 {
                continue;
            }
            if k.get_value::<u32, _>("ParentKeyName").is_ok() {
                continue;
            }
            let version: String = k.get_value("DisplayVersion").unwrap_or_default();
            let dedupe = format!("{}|{}", name.to_lowercase(), version);
            if seen.contains(&dedupe) {
                continue;
            }
            seen.push(dedupe);
            // Extract install location from registry
            let location: String = k.get_value("InstallLocation").unwrap_or_default();
            // Determine install type from registry properties (placeholder)
            let install_type = detect_install_type();
            // Measure actual program size using InstallLocation
            let actual_size = measure_actual_size(&location);
            // Determine protection level
            let protection_level = detect_protection_level(&name);
            // Determine install source from hive
            let install_source = detect_install_source(hive);

            out.push(Program {
                key: full,
                name,
                version,
                publisher: k.get_value("Publisher").unwrap_or_default(),
                install_date: k.get_value("InstallDate").unwrap_or_default(),
                size_kb: k
                    .get_value::<u32, _>("EstimatedSize")
                    .map(|v| v as u64)
                    .unwrap_or(0),
                actual_size_kb: actual_size,
                uninstall: k.get_value("UninstallString").unwrap_or_default(),
                quiet_uninstall: k.get_value("QuietUninstallString").unwrap_or_default(),
                location,
                hive: hive.to_string(),
                display_icon: k.get_value("DisplayIcon").unwrap_or_default(),
                install_type,
                protection_level,
                install_source,
            });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Result of running a program's uninstaller, verified against the registry.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UninstallOutcome {
    /// The registry entry disappeared — the uninstall really happened.
    pub ok: bool,
    /// Uninstall key no longer exists after the run.
    pub key_gone: bool,
    /// Process exit code (-1 when it could not be waited for).
    pub exit_code: i32,
    /// Machine-readable result: gone | stillRegistered | launchFailed |
    /// noCommand | timeout (the UI translates it).
    pub message: String,
    /// Raw error/path detail for the UI.
    pub detail: String,
    /// Leftovers found after a successful uninstall (empty on failure).
    pub leftovers: Vec<Leftover>,
}

/// How long we wait for an uninstaller before giving up on it.
const UNINSTALL_TIMEOUT_S: u64 = 600;

/// Split a command line honouring double quotes.
fn split_cmdline(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut started = false;
    for c in s.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                started = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if started || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    started = false;
                }
            }
            c => {
                cur.push(c);
                started = true;
            }
        }
    }
    if started || !cur.is_empty() {
        out.push(cur);
    }
    out
}

const EXE_LIKE: &[&str] = &[".exe", ".msi", ".bat", ".cmd", ".com"];

fn looks_like_executable(token: &str) -> bool {
    let lower = token.to_lowercase();
    EXE_LIKE.iter().any(|ext| lower.ends_with(ext))
}

/// Turn a registry uninstall string into (program, args).
/// Handles quoted paths, unquoted paths containing spaces, `%env%` expansion,
/// bare tool names (`MsiExec.exe /X{…}`) and `.msi` / `.bat` / `.cmd` wrappers.
fn resolve_command(cmd: &str) -> Result<(String, Vec<String>), String> {
    let expanded = regutil::expand_env(cmd.trim());
    let tokens = split_cmdline(&expanded);
    if tokens.is_empty() {
        return Err("empty uninstall command".into());
    }
    let mut exe = tokens[0].clone();
    let mut rest: Vec<String> = tokens[1..].to_vec();

    // Unquoted executable path that contains spaces: join leading tokens
    // until an actual file is formed.
    if !std::path::Path::new(&exe).is_file() {
        for n in 2..=tokens.len() {
            let cand = tokens[..n].join(" ");
            if std::path::Path::new(&cand).is_file() {
                exe = cand;
                rest = tokens[n..].to_vec();
                break;
            }
        }
        // The file may be gone (broken uninstaller) — still take the longest
        // token prefix ending in an executable extension so the caller can
        // report the real missing path.
        if !std::path::Path::new(&exe).is_file() && !looks_like_executable(&exe) {
            for n in 2..=tokens.len() {
                if looks_like_executable(&tokens[n - 1]) {
                    exe = tokens[..n].join(" ");
                    rest = tokens[n..].to_vec();
                    break;
                }
            }
        }
    }

    let lower = exe.to_lowercase();
    // MSI packages are uninstalled through msiexec.
    if lower.ends_with(".msi") {
        let pkg = std::mem::take(&mut exe);
        exe = "msiexec.exe".into();
        rest.insert(0, pkg);
        rest.insert(0, "/X".to_string());
    }
    // MSI maintenance strings often say `/I{GUID}` (install/repair) — turn
    // them into a real uninstall (`/X`).
    if exe.eq_ignore_ascii_case("msiexec.exe") {
        for a in rest.iter_mut() {
            let first2: String = a.chars().take(2).collect();
            if first2.eq_ignore_ascii_case("/I") && (a.len() == 2 || a[2..].starts_with('{')) {
                a.replace_range(..2, "/X");
            }
        }
    }
    // Shell scripts need an explicit interpreter (args stay separately quoted
    // — no cmd quoting breakage).
    if lower.ends_with(".bat") || lower.ends_with(".cmd") {
        let mut wrapped = vec!["/C".to_string(), exe];
        wrapped.extend(rest);
        return Ok(("cmd.exe".into(), wrapped));
    }
    Ok((exe, rest))
}

/// True when an executable path/name cannot be resolved on this machine.
fn exe_missing(exe: &str) -> bool {
    if !looks_like_executable(exe) {
        return false; // bare tool name we cannot resolve — assume present
    }
    if exe.contains('\\') || exe.contains('/') {
        return !std::path::Path::new(exe).is_file();
    }
    !exe_on_path(exe)
}

fn exe_on_path(name: &str) -> bool {
    let mut dirs: Vec<String> = Vec::new();
    if let Ok(sysroot) = std::env::var("SystemRoot") {
        dirs.push(format!(r"{sysroot}\System32"));
        dirs.push(sysroot);
    }
    if let Ok(path) = std::env::var("PATH") {
        dirs.extend(path.split(';').map(|s| s.to_string()));
    }
    dirs.iter()
        .filter(|d| !d.trim().is_empty())
        .any(|d| std::path::Path::new(d).join(name).is_file())
}

struct RunOutcome {
    exit_code: i32,
    timed_out: bool,
    launch_error: Option<String>,
    elevated: bool,
}

fn widen(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Relaunch the uninstaller elevated (UAC) and wait for it.
fn run_elevated(exe: &str, args: &[String]) -> RunOutcome {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, WAIT_OBJECT_0};
    use windows_sys::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject};
    use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SHELLEXECUTEINFOW, SEE_MASK_NOCLOSEPROCESS};
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let params = args.join(" ");
    let verb = widen("runas");
    let file = widen(exe);
    let params_w = widen(&params);
    let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    info.fMask = SEE_MASK_NOCLOSEPROCESS;
    info.lpVerb = verb.as_ptr();
    info.lpFile = file.as_ptr();
    info.lpParameters = params_w.as_ptr();
    info.nShow = SW_SHOWNORMAL;

    let ok = unsafe { ShellExecuteExW(&mut info) };
    if ok == 0 {
        let err = unsafe { GetLastError() };
        let msg = match err {
            1223 => "the elevation prompt was declined".to_string(),
            _ => format!("cannot start elevated (OS error {err})"),
        };
        return RunOutcome {
            exit_code: -1,
            timed_out: false,
            launch_error: Some(msg),
            elevated: false,
        };
    }
    if !info.hProcess.is_null() {
        let ms = (UNINSTALL_TIMEOUT_S * 1000) as u32;
        let wait = unsafe { WaitForSingleObject(info.hProcess, ms) };
        let mut code: u32 = 0;
        unsafe { GetExitCodeProcess(info.hProcess, &mut code) };
        unsafe { CloseHandle(info.hProcess) };
        return if wait == WAIT_OBJECT_0 {
            RunOutcome {
                exit_code: code as i32,
                timed_out: false,
                launch_error: None,
                elevated: true,
            }
        } else {
            RunOutcome {
                exit_code: -1,
                timed_out: true,
                launch_error: None,
                elevated: true,
            }
        };
    }
    // No process handle: the caller verifies the outcome via the registry.
    RunOutcome {
        exit_code: -1,
        timed_out: false,
        launch_error: None,
        elevated: true,
    }
}

/// Spawn the uninstaller directly (no cmd.exe wrapper — no quote breakage)
/// and wait for it to finish. Falls back to a UAC-elevated launch when the
/// OS reports that elevation is required.
fn run_uninstaller(exe: &str, args: &[String]) -> RunOutcome {
    let mut command = std::process::Command::new(exe);
    command.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    match command.spawn() {
        Ok(mut child) => {
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(UNINSTALL_TIMEOUT_S);
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        return RunOutcome {
                            exit_code: status.code().unwrap_or(-1),
                            timed_out: false,
                            launch_error: None,
                            elevated: false,
                        }
                    }
                    Ok(None) => {
                        if start.elapsed() >= timeout {
                            return RunOutcome {
                                exit_code: -1,
                                timed_out: true,
                                launch_error: None,
                                elevated: false,
                            };
                        }
                        std::thread::sleep(std::time::Duration::from_millis(400));
                    }
                    Err(e) => {
                        return RunOutcome {
                            exit_code: -1,
                            timed_out: false,
                            launch_error: Some(e.to_string()),
                            elevated: false,
                        }
                    }
                }
            }
        }
        Err(e) => {
            let code = e.raw_os_error();
            // 740 = ERROR_ELEVATION_REQUIRED, 5 = ACCESS_DENIED.
            if code == Some(740) || code == Some(5) {
                return run_elevated(exe, args);
            }
            RunOutcome {
                exit_code: -1,
                timed_out: false,
                launch_error: Some(e.to_string()),
                elevated: false,
            }
        }
    }
}

fn key_exists(key: &str) -> bool {
    regutil::open(key).is_some()
}

fn normalize_str(s: &str) -> String {
    s.replace('/', "\\").trim_end_matches('\\').to_lowercase()
}

/// True when `path` is exactly the install location of a still-installed
/// program (the overlap guard in safety deliberately ignores equality).
fn is_install_location(path: &std::path::Path, locations: &[String]) -> bool {
    let c = normalize_str(&path.to_string_lossy());
    !c.is_empty() && locations.iter().any(|l| !l.trim().is_empty() && normalize_str(l) == c)
}

/// Collect the absolute uninstaller executables of every installed program.
fn active_uninstallers(all: &[Program]) -> Vec<String> {
    all.iter()
        .filter_map(|p| {
            let cmd = if !p.quiet_uninstall.trim().is_empty() {
                &p.quiet_uninstall
            } else {
                &p.uninstall
            };
            if cmd.trim().is_empty() {
                return None;
            }
            let (exe, _) = resolve_command(cmd).ok()?;
            if exe.contains('\\') || exe.contains('/') {
                Some(normalize_str(&exe))
            } else {
                None
            }
        })
        .collect()
}

/// True when a still-registered program's uninstaller lives inside `path` —
/// the folder is in use even if its name tokens do not match.
fn used_by_uninstaller(path: &std::path::Path, uninstallers: &[String]) -> bool {
    let c = normalize_str(&path.to_string_lossy());
    if c.is_empty() {
        return false;
    }
    uninstallers.iter().any(|u| u.starts_with(&(c.clone() + "\\")))
}

/// Run the program's uninstaller, wait for it, then verify against the
/// registry that the entry is really gone. Only after that are leftovers
/// scanned so the UI can show a delete-review dialog.
pub fn launch_uninstall(program: &Program) -> UninstallOutcome {
    let cmd = if !program.quiet_uninstall.trim().is_empty() {
        program.quiet_uninstall.clone()
    } else {
        program.uninstall.clone()
    };
    let mut outcome = UninstallOutcome {
        ok: false,
        key_gone: false,
        exit_code: -1,
        message: String::new(),
        detail: String::new(),
        leftovers: Vec::new(),
    };
    if cmd.trim().is_empty() {
        outcome.message = "noCommand".into();
        outcome.key_gone = key_exists(&program.key);
        return outcome;
    }
    let (exe, args) = match resolve_command(&cmd) {
        Ok(v) => v,
        Err(e) => {
            outcome.message = "launchFailed".into();
            outcome.detail = e;
            outcome.key_gone = key_exists(&program.key);
            return outcome;
        }
    };

    let run = run_uninstaller(&exe, &args);
    outcome.exit_code = run.exit_code;
    outcome.detail = match (&run.launch_error, run.elevated) {
        (Some(e), true) => format!("{e} [launched elevated]"),
        (Some(e), false) => e.clone(),
        (None, _) => String::new(),
    };

    // The uninstaller may remove its key a moment after exiting — poll briefly.
    let mut gone = !key_exists(&program.key);
    if !gone && run.launch_error.is_none() {
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(750));
            if !key_exists(&program.key) {
                gone = true;
                break;
            }
        }
    }

    outcome.key_gone = gone;
    outcome.ok = gone;
    outcome.message = if gone {
        "gone".into()
    } else if run.launch_error.is_some() {
        "launchFailed".into()
    } else if run.timed_out {
        "timeout".into()
    } else {
        "stillRegistered".into()
    };

    if gone {
        crate::history::add("uninstall", &program.name, "إلغاء التثبيت", 0, 1);
        let mut lv = leftovers_with_level(program, "advanced");
        // Fallback: if the narrow leftover scan found nothing, run the broader
        // forced-uninstall plan so the user can still clean remaining traces.
        if lv.is_empty() {
            if let Ok(plan) = forced_uninstall_plan(&program.name) {
                for item in plan.items.into_iter().filter(|i| i.safe) {
                    lv.push(Leftover {
                        path: item.path,
                        kind: item.kind,
                        size: item.size,
                    });
                }
            }
        }
        outcome.leftovers = lv;
    }
    outcome
}

pub fn remove_entry(program: &Program) -> Result<(), String> {
    let (parent, name) = program
        .key
        .rsplit_once('\\')
        .ok_or_else(|| "invalid key".to_string())?;
    if regutil::delete_subkey(parent, name) {
        Ok(())
    } else {
        Err("cannot delete entry (run as admin?)".into())
    }
}

/// Determine the installation type from registry properties.
/// (Placeholder: default to Win32; a real implementation would examine
/// key values such as "ParentKeyName", "DisplayIcon", etc.)
fn detect_install_type() -> InstallType {
    // Placeholder implementation: default to Win32.
    InstallType::Win32
}

/// Determine the protection level of a program based on its name.
fn detect_protection_level(_name: &str) -> ProtectionLevel {
    // Placeholder implementation: return Safe for most programs.
    // A real implementation would check against sensitive vendor names,
    // system-critical applications, etc.
    ProtectionLevel::Safe
}

/// Determine the install source from the registry hive name.
fn detect_install_source(hive: &str) -> InstallSource {
    match hive {
        "machine" => InstallSource::MachineUninstallKey,
        "user" => InstallSource::UserUninstallKey,
        "machine32" => InstallSource::MachineUninstallKey,
        _ => InstallSource::Unknown,
    }
}

/// Attempt to measure the actual program size by reading InstallLocation
/// and computing the directory size. Returns None if the location is
/// unavailable or cannot be measured (placeholder for now).
fn measure_actual_size(_location: &str) -> Option<u64> {
    // Placeholder implementation: return None to indicate "unknown size".
    // A full implementation would:
    // 1. Parse the InstallLocation path
    // 2. Recursively sum file sizes if the directory exists
    // 3. Handle symlinks, Hard Links, and protected files
    // 4. Subtract shared/system components
    // For now, if location is non-empty, attempt basic dir size check
    if _location.is_empty() {
        return None;
    }
    // Basic check: attempt to compute directory size if path exists
    // (This is a simplified placeholder; real implementation would use
    //  walkdir or similar to recursively sum file sizes)
    let _ = _location; // suppress unused variable
    None
}

pub fn tokens(name: &str) -> Vec<String> {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    cleaned
        .split_whitespace()
        .filter(|t| t.len() >= 3)
        .map(|t| t.to_lowercase())
        .filter(|t| {
            ![
                "the", "and", "for", "with", "microsoft", "windows", "corporation",
                "limited", "ltd", "inc", "llc", "software", "version", "pro", "free",
                "edition", "suite", "app", "apps",
            ]
            .contains(&t.as_str())
        })
        .collect()
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Leftover {
    pub path: String,
    pub kind: String,
    pub size: u64,
}

pub fn leftovers(program: &Program) -> Vec<Leftover> {
    leftovers_with_level(program, "advanced")
}

/// level: "safe" | "moderate" | "advanced"
pub fn leftovers_with_level(program: &Program, level: &str) -> Vec<Leftover> {
    let mut out: Vec<Leftover> = Vec::new();
    let toks = tokens(&program.name);
    if toks.is_empty() && program.location.trim().is_empty() {
        return out;
    }

    let mut roots_dirs: Vec<String> = Vec::new();
    if level == "advanced" {
        roots_dirs.push(format!(r"{}\Roaming", std::env::var("APPDATA").unwrap_or_default()));
        roots_dirs.push(util::local_appdata());
        roots_dirs.push(util::program_data());
        roots_dirs.push(util::temp_dir());
    }
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
    roots_dirs.push(pf);
    roots_dirs.push(pf86);

    let mut seen: Vec<String> = Vec::new();
    for root in roots_dirs {
        let rd = match std::fs::read_dir(&root) { Ok(r) => r, Err(_) => continue };
        for entry in rd.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let dir_name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
            if dir_name.len() < 3 { continue; }
            let matches = if toks.is_empty() { false } else {
                toks.iter().any(|t| t.len() >= 4 && dir_name.contains(t))
            };
            if !matches { continue; }
            let key = path.to_string_lossy().to_lowercase();
            if seen.contains(&key) { continue; }
            seen.push(key.clone());
            let size = dir_size(&path);
            out.push(Leftover { path: path.to_string_lossy().to_string(), kind: "folder".into(), size });
        }
    }

    if level != "safe" && !toks.is_empty() {
        let reg_roots = [
            r"HKEY_CURRENT_USER\Software",
            r"HKEY_LOCAL_MACHINE\SOFTWARE",
            r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node",
        ];
        for rroot in reg_roots {
            if let Some(key) = regutil::open(rroot) {
                for sub in key.enum_keys().flatten() {
                    let lower = sub.to_lowercase();
                    if toks.iter().any(|t| t.len() >= 4 && lower.contains(t)) {
                        let full = format!(r"{}\{}", rroot, sub);
                        out.push(Leftover { path: full, kind: "registry".into(), size: 0 });
                    }
                }
            }
        }
    }

    if !program.location.trim().is_empty() && std::path::Path::new(&program.location).is_dir() {
        let p = program.location.trim_end_matches('\\').to_string();
        if !seen.iter().any(|s| s == &p.to_lowercase()) {
            out.push(Leftover { size: dir_size(std::path::Path::new(&p)), path: p, kind: "installDir".into() });
        }
    }
    out.sort_by(|a, b| b.size.cmp(&a.size));
    if level == "safe" { out.truncate(20); } else { out.truncate(60); }
    out
}

/// Build a reviewable plan of what a forced uninstall would touch.
/// Nothing is deleted here. Every item is marked safe/blocked with a reason.
pub fn forced_uninstall_plan(name: &str) -> Result<safety::CleanupPlan, String> {
    if name.trim().is_empty() {
        return Err("name required".into());
    }
    let needle = name.to_lowercase();
    let all = list();
    let target_key = all
        .iter()
        .find(|p| p.name.to_lowercase() == needle || p.name.to_lowercase().contains(&needle))
        .map(|p| p.key.clone());

    let others: Vec<&Program> = all
        .iter()
        .filter(|p| target_key.as_ref().map(|k| k != &p.key).unwrap_or(true))
        .collect();
    let other_locations: Vec<String> = others.iter().map(|p| p.location.clone()).filter(|s| !s.trim().is_empty()).collect();
    let other_tokens: Vec<Vec<String>> = others.iter().map(|p| tokens(&p.name)).filter(|t| !t.is_empty()).collect();
    let toks = tokens(name);
    let mut items: Vec<safety::CleanupItem> = Vec::new();

    // --- registry uninstall entries -------------------------------------
    for (hive, root) in roots() {
        let _ = hive;
        let key = match regutil::open(&root) { Some(k) => k, None => continue };
        for sub in key.enum_keys().flatten() {
            let full = format!(r"{}\{}", root, sub);
            let k = match regutil::open(&full) { Some(k) => k, None => continue };
            let dn: String = k.get_value("DisplayName").unwrap_or_default();
            if dn.to_lowercase().contains(&needle) {
                items.push(safety::CleanupItem {
                    kind: "registry".into(),
                    path: full,
                    size: 0,
                    safe: true,
                    reason: "match".into(),
                });
            }
        }
    }

    // --- directories -----------------------------------------------------
    let mut roots_dirs = vec![util::local_appdata(), util::program_data(), util::temp_dir()];
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
    roots_dirs.push(pf);
    roots_dirs.push(pf86);
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    roots_dirs.push(format!(r"{}\Roaming", appdata));

    let mut seen = std::collections::HashSet::new();
    for root in roots_dirs {
        if root.trim().is_empty() { continue; }
        let rd = match std::fs::read_dir(&root) { Ok(r) => r, Err(_) => continue };
        for entry in rd.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let leaf = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
            let mut candidates: Vec<std::path::PathBuf> = Vec::new();
            if safety::name_matches(&leaf, &toks) {
                candidates.push(path.clone());
            } else if safety::VENDOR_CONTAINERS.contains(&leaf.as_str()) {
                // Vendor roots are protected, but a matching app inside them
                // (e.g. Google\Chrome) can be safely removed.
                if let Ok(inner) = std::fs::read_dir(&path) {
                    for e in inner.flatten() {
                        let p = e.path();
                        if p.is_dir() {
                            let n = e.file_name().to_string_lossy().to_lowercase();
                            if safety::name_matches(&n, &toks) { candidates.push(p); }
                        }
                    }
                }
            }
            for cand in candidates {
                let key = cand.to_string_lossy().to_lowercase();
                if !seen.insert(key) { continue; }
                let (safe, reason) = if safety::is_protected(&cand) {
                    (false, "protected")
                } else if safety::is_reparse_point(&cand) {
                    (false, "reparse")
                } else if safety::owned_by_other_program(&cand, &other_locations) {
                    (false, "otherProgram")
                } else if safety::contains_other_program(&cand, &other_tokens) {
                    (false, "containsOther")
                } else {
                    (true, "match")
                };
                items.push(safety::CleanupItem {
                    kind: "directory".into(),
                    path: cand.to_string_lossy().to_string(),
                    size: dir_size(&cand).min(u64::MAX / 2),
                    safe,
                    reason: reason.into(),
                });
            }
        }
    }
    Ok(safety::CleanupPlan { items, safe_count: 0, blocked_count: 0 }.finish())
}

/// Delete only paths that were reviewed and re-validated. Directories are
/// moved to quarantine; registry keys are exported to .reg first.
pub fn forced_uninstall_selected(name: &str, paths: &[String]) -> Result<usize, String> {
    let plan = forced_uninstall_plan(name)?;
    let mut removed = 0usize;
    for p in paths {
        if !safety::validate_selected(p, &plan) { continue; }
        let item = match plan.items.iter().find(|i| i.path == *p) { Some(i) => i, None => continue };
        if item.kind == "registry" {
            let _ = safety::backup_registry_key(p);
            if let Some((parent, leaf)) = p.rsplit_once('\\') {
                if regutil::delete_subkey(parent, leaf) { removed += 1; }
            }
        } else {
            let path = std::path::Path::new(p);
            if path.is_dir() && safety::quarantine(path).is_ok() { removed += 1; }
        }
    }
    Ok(removed)
}

/// Backwards-compatible entry point. Only auto-removes items the safety
/// guards approved; callers wanting a preview should use the plan API.
pub fn forced_uninstall(name: &str) -> Result<usize, String> {
    let plan = forced_uninstall_plan(name)?;
    let paths = plan.safe_paths();
    forced_uninstall_selected(name, &paths)
}

// ------------------------------------------------------------------ leftovers

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeleteOutcome {
    pub removed: u64,
    pub failed: u64,
    pub freed: u64,
}

/// Delete leftover paths the user reviewed and selected. Registry keys are
/// exported to quarantine first; folders/files honour the recycle-bin
/// setting and every safety guard.
pub fn delete_leftovers(paths: &[String], recycle: bool) -> DeleteOutcome {
    let all = list();
    let other_locations: Vec<String> = all
        .iter()
        .map(|p| p.location.clone())
        .filter(|s| !s.trim().is_empty())
        .collect();
    let other_tokens: Vec<Vec<String>> = all
        .iter()
        .map(|p| tokens(&p.name))
        .filter(|t| !t.is_empty())
        .collect();

    let mut out = DeleteOutcome::default();
    let uninstallers = active_uninstallers(&all);
    let broken = broken_registry_keys();
    for p in paths {
        if p.starts_with("HKEY_") {
            let (parent, leaf) = match p.rsplit_once('\\') {
                Some(v) => v,
                None => {
                    out.failed += 1;
                    continue;
                }
            };
            // At least hive\Software\Something — never a root, never a
            // protected name (Microsoft, Classes, Software…).
            if p.matches('\\').count() < 2
                || leaf.trim().is_empty()
                || safety::is_protected(std::path::Path::new(p))
            {
                out.failed += 1;
                continue;
            }
            if p.to_lowercase().contains("\\uninstall\\") {
                // Only entries whose uninstaller is actually missing may go.
                if !broken.contains(&p.to_lowercase()) {
                    out.failed += 1;
                    continue;
                }
            } else if other_tokens.iter().any(|toks| safety::name_matches(leaf, toks)) {
                // Vendor key of a program that is still installed.
                out.failed += 1;
                continue;
            }
            if regutil::open(p).is_none() {
                out.removed += 1; // already gone
                continue;
            }
            let _ = safety::backup_registry_key(p);
            if regutil::delete_subkey(parent, leaf) {
                out.removed += 1;
            } else {
                out.failed += 1;
            }
            continue;
        }

        let path = std::path::Path::new(p);
        if safety::is_protected(path)
            || safety::is_reparse_point(path)
            || !safety::clean_path_allowed(path)
            || is_install_location(path, &other_locations)
            || used_by_uninstaller(path, &uninstallers)
            || safety::owned_by_other_program(path, &other_locations)
            || safety::contains_other_program(path, &other_tokens)
        {
            out.failed += 1;
            continue;
        }
        if !path.exists() {
            out.removed += 1;
            continue;
        }
        let size = if path.is_dir() {
            dir_size(path)
        } else {
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        };
        let ok = if recycle {
            util::recycle_batch(std::slice::from_ref(p)).is_ok()
        } else {
            util::delete_path(path)
        };
        if ok {
            out.removed += 1;
            out.freed = out.freed.saturating_add(size);
        } else {
            out.failed += 1;
        }
    }
    out
}

/// Leaf names that are shared installer infrastructure, never orphans.
const ORPHAN_SKIP: &[&str] = &[
    "package cache", "installer", "installers", "redist", "redistributable",
    "windows.old", "windows.~bt", "source", "sources", "uninstall",
];

/// Find an uninstaller-looking file inside a folder (bounded depth/walk).
fn find_uninstaller(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut stack = vec![(dir.to_path_buf(), 0u32)];
    let mut visited = 0usize;
    while let Some((d, depth)) = stack.pop() {
        visited += 1;
        if visited > 80 || depth > 3 {
            continue;
        }
        let rd = match std::fs::read_dir(&d) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push((p, depth + 1));
                continue;
            }
            let name = p.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
            let looks = (name.starts_with("unins")
                || name.starts_with("uninstall")
                || name.contains("uninstall"))
                && (name.ends_with(".exe")
                    || name.ends_with(".bat")
                    || name.ends_with(".cmd")
                    || name.ends_with(".msi"));
            if looks {
                return Some(p);
            }
        }
    }
    None
}

/// Resolve Start Menu shortcuts whose target no longer exists.
/// Returns (shortcut path, missing target).
fn dead_shortcuts() -> Vec<(String, String)> {
    let script = r#"
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$sh = New-Object -ComObject WScript.Shell
$roots = @("$env:APPDATA\Microsoft\Windows\Start Menu\Programs", "$env:ProgramData\Microsoft\Windows\Start Menu\Programs")
$out = @()
foreach ($r in $roots) {
  if (Test-Path -LiteralPath $r) {
    Get-ChildItem -LiteralPath $r -Recurse -Filter *.lnk -ErrorAction SilentlyContinue | ForEach-Object {
      try {
        $t = $sh.CreateShortcut($_.FullName).TargetPath
        if ($t -and -not (Test-Path -LiteralPath $t)) { $out += ($_.FullName + '|' + $t) }
      } catch {}
    }
  }
}
$out -join "`n"
"#;
    let text = match util::run_and_wait("powershell", &["-NoProfile", "-STA", "-Command", script]) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    text.lines()
        .filter_map(|l| l.trim().split_once('|'))
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

/// Uninstall entries whose uninstaller executable no longer exists.
fn broken_registry_keys() -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for (_hive, root) in roots() {
        let key = match regutil::open(&root) {
            Some(k) => k,
            None => continue,
        };
        for sub in key.enum_keys().flatten() {
            let full = format!(r"{root}\{sub}");
            let k = match regutil::open(&full) {
                Some(k) => k,
                None => continue,
            };
            let mut cmd: String = k.get_value("QuietUninstallString").unwrap_or_default();
            if cmd.trim().is_empty() {
                cmd = k.get_value("UninstallString").unwrap_or_default();
            }
            if cmd.trim().is_empty() {
                continue;
            }
            if let Ok((exe, _)) = resolve_command(&cmd) {
                if exe_missing(&exe) {
                    set.insert(full.to_lowercase());
                }
            }
        }
    }
    set
}

/// Automatic search for remains of installs/uninstalls:
///  1. broken uninstall entries (uninstaller file missing),
///  2. orphan folders that still ship an uninstaller but belong to no
///     installed program,
///  3. Start Menu shortcuts pointing at missing targets.
/// Nothing is deleted — the result is a review plan for the UI.
pub fn leftover_scan() -> safety::CleanupPlan {
    let all = list();
    let other_locations: Vec<String> = all
        .iter()
        .map(|p| p.location.clone())
        .filter(|s| !s.trim().is_empty())
        .collect();
    let other_tokens: Vec<Vec<String>> = all
        .iter()
        .map(|p| tokens(&p.name))
        .filter(|t| !t.is_empty())
        .collect();
    let mut items: Vec<safety::CleanupItem> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    // --- 1) broken uninstall registry entries ---------------------------
    for full in broken_registry_keys() {
        if seen.insert(full.clone()) {
            items.push(safety::CleanupItem {
                kind: "registry".into(),
                path: full,
                size: 0,
                safe: true,
                reason: "brokenUninstall".into(),
            });
        }
    }

    // --- 2) orphan folders carrying an uninstaller ----------------------
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let roots_dirs = vec![
        pf,
        pf86,
        util::program_data(),
        util::local_appdata(),
        format!(r"{appdata}"),
    ];
    let mut dir_candidates: Vec<std::path::PathBuf> = Vec::new();
    let uninstallers = active_uninstallers(&all);
    for root in roots_dirs {
        if root.trim().is_empty() {
            continue;
        }
        let rd = match std::fs::read_dir(&root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let tops: Vec<std::path::PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        for top in tops {
            let leaf = top
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if leaf.is_empty()
                || leaf == "vexora"
                || ORPHAN_SKIP.contains(&leaf.as_str())
                || safety::is_protected(&top)
                || safety::is_reparse_point(&top)
                || other_tokens.iter().any(|toks| safety::name_matches(&leaf, toks))
                || is_install_location(&top, &other_locations)
                || used_by_uninstaller(&top, &uninstallers)
                || safety::owned_by_other_program(&top, &other_locations)
            {
                continue;
            }
            // A top folder that owns an installed program is skipped, but its
            // other children may still be orphans — evaluate them below.
            let top_blocked = safety::contains_other_program(&top, &other_tokens);
            if !top_blocked && find_uninstaller(&top).is_some() {
                dir_candidates.push(top.clone());
                continue; // parent listed — children would be duplicates
            }
            if let Ok(rd2) = std::fs::read_dir(&top) {
                for e in rd2.flatten() {
                    let child = e.path();
                    if !child.is_dir() {
                        continue;
                    }
                    let cleaf = child
                        .file_name()
                        .map(|n| n.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    if cleaf.is_empty()
                        || ORPHAN_SKIP.contains(&cleaf.as_str())
                        || safety::is_protected(&child)
                        || safety::is_reparse_point(&child)
                        || other_tokens.iter().any(|toks| safety::name_matches(&cleaf, toks))
                        || is_install_location(&child, &other_locations)
                        || used_by_uninstaller(&child, &uninstallers)
                        || safety::owned_by_other_program(&child, &other_locations)
                    {
                        continue;
                    }
                    if find_uninstaller(&child).is_some() {
                        dir_candidates.push(child);
                    }
                }
            }
        }
    }
    for cand in dir_candidates {
        let key = cand.to_string_lossy().to_lowercase();
        if !seen.insert(key) {
            continue;
        }
        // Blocked when a still-installed program lives inside.
        let safe = !safety::contains_other_program(&cand, &other_tokens);
        items.push(safety::CleanupItem {
            kind: "directory".into(),
            path: cand.to_string_lossy().to_string(),
            size: dir_size(&cand).min(u64::MAX / 2),
            safe,
            reason: if safe { "orphanFolder".into() } else { "otherProgram".into() },
        });
    }

    // --- 3) dead Start Menu shortcuts -----------------------------------
    for (lnk, target) in dead_shortcuts() {
        if seen.insert(lnk.to_lowercase()) {
            items.push(safety::CleanupItem {
                kind: "shortcut".into(),
                path: lnk,
                size: 0,
                safe: true,
                reason: format!("deadShortcut:{target}"),
            });
        }
    }

    safety::CleanupPlan { items, safe_count: 0, blocked_count: 0 }.finish()
}

fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    let mut guard = 0;
    while let Some(dir) = stack.pop() {
        guard += 1;
        if guard > 20_000 { break; }
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for entry in rd.flatten() {
                let p = entry.path();
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        stack.push(p);
                    } else {
                        total += meta.len();
                    }
                }
            }
        }
    }
    total
}

/// Extract icons for a list of programs. Returns a map: program key -> base64 PNG (32x32).
pub fn extract_icons(keys: &[String]) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();
    if keys.is_empty() { return result; }

    let programs = list();
    let mut icon_map: Vec<(String, String)> = Vec::new();
    for key in keys {
        if let Some(p) = programs.iter().find(|pr| pr.key == *key) {
            let icon_path = if !p.display_icon.trim().is_empty() {
                p.display_icon.split(',').next().unwrap_or("").trim().to_string()
            } else if !p.location.trim().is_empty() {
                find_exe_in_dir(&p.location)
            } else {
                String::new()
            };
            if !icon_path.is_empty() && std::path::Path::new(&icon_path).exists() {
                icon_map.push((key.clone(), icon_path));
            }
        }
    }

    if icon_map.is_empty() { return result; }

    let mut script = String::from("Add-Type -AssemblyName System.Drawing; $out = @{}; ");
    for (key, path) in &icon_map {
        let safe_key = key.replace('\'', "''");
        let safe_path = path.replace('\'', "''");
        script.push_str(&format!(
            "try {{ $ic = [System.Drawing.Icon]::ExtractAssociatedIcon('{}'); if ($ic) {{ $ms = New-Object System.IO.MemoryStream; $ic.ToBitmap().Save($ms, [System.Drawing.Imaging.ImageFormat]::Png); $out['{}'] = [Convert]::ToBase64String($ms.ToArray()); $ms.Dispose(); $ic.Dispose(); }} }} catch {{ }}; ",
            safe_path, safe_key
        ));
    }
    script.push_str("$out | ConvertTo-Json -Compress");

    let text = match util::run_and_wait(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script.as_str()],
    ) {
        Ok(t) => t,
        Err(_) => return result,
    };

    let trimmed = text.trim();
    if trimmed.is_empty() { return result; }
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(obj) = parsed.as_object() {
            for (k, v) in obj {
                if let Some(b64) = v.as_str() {
                    if !b64.is_empty() {
                        result.insert(k.clone(), b64.to_string());
                    }
                }
            }
        }
    }
    result
}

fn find_exe_in_dir(dir: &str) -> String {
    if dir.is_empty() { return String::new(); }
    let path = std::path::Path::new(dir);
    if !path.is_dir() { return String::new(); }
    if let Ok(rd) = std::fs::read_dir(path) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e == "exe").unwrap_or(false) {
                return p.to_string_lossy().to_string();
            }
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_quoted_command_line() {
        assert_eq!(
            split_cmdline(r#""C:\Program Files\App\unins.exe" /S /D=C:\Dir"#),
            vec!["C:\\Program Files\\App\\unins.exe", "/S", "/D=C:\\Dir"]
        );
    }

    #[test]
    fn resolves_quoted_exe_with_args() {
        let (exe, args) =
            resolve_command(r#""C:\Program Files\App\unins.exe" /SILENT"#).unwrap();
        assert_eq!(exe, r"C:\Program Files\App\unins.exe");
        assert_eq!(args, vec!["/SILENT"]);
    }

    #[test]
    fn resolves_unquoted_exe_with_spaces() {
        // The file does not exist on the test machine — the ".exe" token
        // boundary must still be found.
        let (exe, args) =
            resolve_command(r"C:\Program Files\Gone App\unins000.exe /S").unwrap();
        assert_eq!(exe, r"C:\Program Files\Gone App\unins000.exe");
        assert_eq!(args, vec!["/S"]);
    }

    #[test]
    fn keeps_bare_tool_name() {
        let (exe, args) = resolve_command("MsiExec.exe /X{1234-ABCD}").unwrap();
        assert_eq!(exe, "MsiExec.exe");
        assert_eq!(args, vec!["/X{1234-ABCD}"]);
    }

    #[test]
    fn wraps_bat_in_cmd() {
        let (exe, args) = resolve_command(r"C:\App\clean.bat /quiet").unwrap();
        assert_eq!(exe, "cmd.exe");
        assert_eq!(args, vec!["/C", r"C:\App\clean.bat", "/quiet"]);
    }

    #[test]
    fn msi_goes_through_msiexec() {
        let (exe, args) = resolve_command(r"C:\Downloads\pkg.msi /qn").unwrap();
        assert_eq!(exe, "msiexec.exe");
        assert_eq!(args, vec!["/X", r"C:\Downloads\pkg.msi", "/qn"]);
    }

    #[test]
    fn msi_maintenance_i_becomes_x() {
        let (exe, args) = resolve_command("MsiExec.exe /I{1234-ABCD}").unwrap();
        assert_eq!(exe, "MsiExec.exe");
        assert_eq!(args, vec!["/X{1234-ABCD}"]);

        let (exe2, args2) = resolve_command("MsiExec.exe /I {1234-ABCD} /qn").unwrap();
        assert_eq!(exe2, "MsiExec.exe");
        assert_eq!(args2, vec!["/X", "{1234-ABCD}", "/qn"]);

        // Non-MSI tools must keep their /I argument untouched.
        let (_, args3) = resolve_command(r"C:\App\setup.exe /Install /quiet").unwrap();
        assert_eq!(args3, vec!["/Install", "/quiet"]);
    }

    #[test]
    fn expands_environment_variables() {
        std::env::set_var("VEXORA_TEST_DIR", r"C:\Temp\Test App");
        let (exe, _) = resolve_command(r"%VEXORA_TEST_DIR%\unins.exe /S").unwrap();
        assert_eq!(exe, r"C:\Temp\Test App\unins.exe");
        std::env::remove_var("VEXORA_TEST_DIR");
    }

    #[test]
    fn detects_missing_executable() {
        assert!(exe_missing(r"C:\Definitely\Missing\unins000.exe"));
        assert!(!exe_missing("MsiExec.exe")); // present in System32
    }

    #[test]
    fn split_handles_empty_quoted_arg() {
        assert_eq!(split_cmdline(r#"prog.exe "" /S"#), vec!["prog.exe", "", "/S"]);
    }

    /// Real-machine check: every uninstall command registered on this PC
    /// must parse into a runnable program + arguments.
    #[test]
    fn parses_every_installed_uninstaller() {
        let progs = list();
        assert!(!progs.is_empty(), "no installed programs found");
        let mut checked = 0usize;
        let mut unparsable: Vec<String> = Vec::new();
        for p in &progs {
            let cmd = if !p.quiet_uninstall.trim().is_empty() {
                &p.quiet_uninstall
            } else {
                &p.uninstall
            };
            if cmd.trim().is_empty() {
                continue;
            }
            match resolve_command(cmd) {
                Ok((exe, _)) if !exe.trim().is_empty() => checked += 1,
                _ => unparsable.push(format!("{} => {}", p.name, cmd)),
            }
        }
        assert!(
            unparsable.is_empty(),
            "unparsable uninstall strings: {unparsable:?}"
        );
        assert!(checked > 0, "no uninstall commands parsed");
        println!("parsed {checked} uninstall commands of {} programs", progs.len());
    }

    /// Real-machine smoke test of the automatic leftover scan (slow: runs
    /// PowerShell for the shortcut check) — run explicitly with
    /// `cargo test --lib leftover_scan_smoke -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn leftover_scan_smoke() {
        let plan = leftover_scan();
        let mut by_reason: std::collections::BTreeMap<String, usize> = Default::default();
        for i in &plan.items {
            *by_reason
                .entry(i.reason.split(':').next().unwrap_or("?").to_string())
                .or_default() += 1;
        }
        println!(
            "leftover_scan: {} items ({} safe, {} blocked), reasons: {:?}",
            plan.items.len(),
            plan.safe_count,
            plan.blocked_count,
            by_reason
        );
        for i in plan.items.iter().take(20) {
            println!("  [{}] {} — {}", if i.safe { "safe" } else { "blocked" }, i.reason, i.path);
        }
    }
}
