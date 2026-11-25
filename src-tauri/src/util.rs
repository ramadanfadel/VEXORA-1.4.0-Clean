//! Shared low-level helpers (Win32 wrappers, path expansion, deletion).

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

static OP_LOCK: Mutex<()> = Mutex::new(());

/// Try to acquire the global "long operation" lock. Returns None if another
/// long-running operation is already in progress.
pub fn try_op_lock() -> Option<MutexGuard<'static, ()>> {
    OP_LOCK.try_lock().ok()
}

pub fn expand(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut name = String::new();
            let mut closed = false;
            for c2 in chars.by_ref() {
                if c2 == '%' {
                    closed = true;
                    break;
                }
                name.push(c2);
            }
            if closed {
                match std::env::var(&name) {
                    Ok(v) => out.push_str(&v),
                    Err(_) => {
                        out.push('%');
                        out.push_str(&name);
                        out.push('%');
                    }
                }
            } else {
                out.push('%');
                out.push_str(&name);
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn local_appdata() -> String {
    std::env::var("LOCALAPPDATA").unwrap_or_else(|_| r"C:\Users\Public".into())
}

pub fn appdata() -> String {
    std::env::var("APPDATA").unwrap_or_else(|_| r"C:\Users\Public".into())
}

pub fn program_data() -> String {
    std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".into())
}

pub fn windir() -> String {
    std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into())
}

pub fn user_profile() -> String {
    std::env::var("USERPROFILE").unwrap_or_else(|_| r"C:\Users\Public".into())
}

pub fn temp_dir() -> String {
    std::env::var("TEMP").unwrap_or_else(|_| windir() + r"\Temp")
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    let p = PathBuf::from(base).join("VEXORA");
    let _ = std::fs::create_dir_all(&p);
    p
}

/// Copy settings/history from the old Abu Maryam Cleaner profile on first run.
pub fn migrate_old_data() {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    let old = PathBuf::from(&base).join("AbuMaryamCleaner");
    let new = PathBuf::from(&base).join("VEXORA");
    if !old.is_dir() {
        return;
    }
    let _ = std::fs::create_dir_all(&new);
    for name in ["settings.json", "history.jsonl", "history.json"] {
        let src = old.join(name);
        let dst = new.join(name);
        if src.is_file() && !dst.exists() {
            let _ = std::fs::copy(&src, &dst);
        }
    }
    let old_backups = old.join("backups");
    let new_backups = new.join("backups");
    if old_backups.is_dir() && !new_backups.exists() {
        let _ = std::fs::create_dir_all(&new_backups);
        if let Ok(rd) = std::fs::read_dir(&old_backups) {
            for e in rd.flatten() {
                if e.path().is_file() {
                    let _ = std::fs::copy(e.path(), new_backups.join(e.file_name()));
                }
            }
        }
    }
}

pub fn reports_dir() -> PathBuf {
    let p = data_dir().join("reports");
    let _ = std::fs::create_dir_all(&p);
    p
}

pub fn backups_dir() -> PathBuf {
    let p = data_dir().join("backups");
    let _ = std::fs::create_dir_all(&p);
    p
}

pub fn is_admin() -> bool {
    unsafe { windows_sys::Win32::UI::Shell::IsUserAnAdmin() != 0 }
}

fn widen(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// Find a window by title and bring it to the foreground (used by the
/// single-instance guard).
pub fn focus_window(title: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
    };
    let w = widen(title);
    let hwnd = unsafe { FindWindowW(std::ptr::null(), w.as_ptr()) };
    if !hwnd.is_null() {
        unsafe {
            if ShowWindow(hwnd, SW_RESTORE) == 0 {
                ShowWindow(hwnd, SW_SHOW);
            }
            SetForegroundWindow(hwnd);
        }
    }
}

/// Prevent two VEXORA windows from running at once. Returns after focusing
/// the existing instance when this process is a duplicate.
pub fn ensure_single_instance() {
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    let name = widen(r"Global\VEXORA.SingleInstance");
    let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    if handle.is_null() {
        return;
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        focus_window("VEXORA");
        std::process::exit(0);
    }
    // Intentionally keep the handle for the lifetime of the process.
}

/// Case-insensitive path matching with optional `*` / `?` wildcards.
/// A plain directory also matches everything below it.
pub fn path_matches(pattern: &str, path: &str) -> bool {
    let pat = pattern.trim().to_lowercase().replace('/', "\\");
    let p = path.to_lowercase().replace('/', "\\");
    if pat.is_empty() {
        return false;
    }
    if pat.contains('*') || pat.contains('?') {
        if glob_match(&pat, &p) {
            return true;
        }
        // `C:\Games\*` should also protect the folder itself.
        let trimmed = pat.trim_end_matches('\\').trim_end_matches('*').trim_end_matches('\\');
        if !trimmed.is_empty() && (p == trimmed || p.starts_with(&format!("{trimmed}\\"))) {
            return true;
        }
        return false;
    }
    p == pat || p.starts_with(&format!("{pat}\\"))
}

pub fn glob_match(pattern: &str, text: &str) -> bool {
    let pb = pattern.as_bytes();
    let tb = text.as_bytes();
    let (mut pi, mut ti) = (0usize, 0usize);
    let (mut star, mut mark) = (usize::MAX, 0usize);
    while ti < tb.len() {
        if pi < pb.len() && (pb[pi] == b'?' || pb[pi] == tb[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < pb.len() && pb[pi] == b'*' {
            star = pi;
            mark = ti;
            pi += 1;
        } else if star != usize::MAX {
            pi = star + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < pb.len() && pb[pi] == b'*' {
        pi += 1;
    }
    pi == pb.len()
}

/// Send a batch of paths to the recycle bin using SHFileOperationW.
pub fn recycle_batch(paths: &[String]) -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::{
        SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT,
        FO_DELETE, SHFILEOPSTRUCTW,
    };

    if paths.is_empty() {
        return Ok(());
    }
    let mut body: Vec<u16> = Vec::new();
    for p in paths {
        body.extend(OsStr::new(p).encode_wide());
        body.push(0);
    }
    body.push(0);

    let mut op = SHFILEOPSTRUCTW {
        hwnd: std::ptr::null_mut(),
        wFunc: FO_DELETE as u32,
        pFrom: body.as_ptr(),
        pTo: std::ptr::null(),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT) as u16,
        fAnyOperationsAborted: 0,
        hNameMappings: std::ptr::null_mut(),
        lpszProgressTitle: std::ptr::null(),
    };
    let rc = unsafe { SHFileOperationW(&mut op) };
    if rc != 0 {
        return Err(format!("SHFileOperation failed: 0x{rc:04X}"));
    }
    if op.fAnyOperationsAborted != 0 {
        return Err("aborted".into());
    }
    Ok(())
}

pub fn empty_recycle_bin() -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND};
    let flags = SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND;
    let rc = unsafe { SHEmptyRecycleBinW(std::ptr::null_mut(), std::ptr::null(), flags) };
    // S_FALSE (1) means the bin was already empty.
    if rc == 0 || rc == 1 || rc == -2147418113 {
        Ok(())
    } else {
        Err(format!("SHEmptyRecycleBin failed: 0x{rc:08X}"))
    }
}

pub fn force_writable(path: &Path) {
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        if perms.readonly() {
            perms.set_readonly(false);
            let _ = std::fs::set_permissions(path, perms);
        }
    }
}

pub fn delete_path(path: &Path) -> bool {
    force_writable(path);
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return true,
    };
    if meta.is_dir() {
        std::fs::remove_dir_all(path).is_ok() || !path.exists()
    } else {
        std::fs::remove_file(path).is_ok() || !path.exists()
    }
}

pub fn shred_path(path: &Path, passes: u32) -> bool {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return true,
    };
    if meta.is_dir() {
        if let Ok(rd) = std::fs::read_dir(path) {
            for entry in rd.flatten() {
                let _ = shred_path(&entry.path(), passes);
            }
        }
        return std::fs::remove_dir(path).is_ok() || !path.exists();
    }
    let size = meta.len();
    force_writable(path);
    let ok = (|| -> std::io::Result<()> {
        use std::io::{Seek, SeekFrom, Write};
        let mut f = std::fs::OpenOptions::new().write(true).open(path)?;
        let mut buf = vec![0u8; 256 * 1024];
        for pass in 0..passes.max(1) {
            f.seek(SeekFrom::Start(0))?;
            let mut left = size;
            while left > 0 {
                let n = left.min(buf.len() as u64) as usize;
                fill_random(&mut buf[..n], pass as u8);
                f.write_all(&buf[..n])?;
                left -= n as u64;
            }
            f.flush()?;
            let _ = f.sync_all();
        }
        Ok(())
    })();
    let _ = ok;
    std::fs::remove_file(path).is_ok() || !path.exists()
}

fn fill_random(buf: &mut [u8], salt: u8) {
    // xorshift PRNG seeded from the clock — good enough for overwriting.
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut state = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B97F4A7C15)
        ^ ((buf.len() as u64) << 17)
        ^ (salt as u64);
    for b in buf.iter_mut() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *b = (state >> 24) as u8;
    }
}

pub fn clear_clipboard() -> Result<(), String> {
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard,
    };
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err("clipboard busy".into());
        }
        let ok = EmptyClipboard() != 0;
        CloseClipboard();
        if ok {
            Ok(())
        } else {
            Err("could not empty clipboard".into())
        }
    }
}

pub fn run_command(program: &str, args: &[String]) -> Result<(), String> {
    let mut cmd = std::process::Command::new(program);
    for a in args {
        cmd.arg(a);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("{program}: {e}"))
}

pub fn run_and_wait(program: &str, args: &[&str]) -> Result<String, String> {
    let owned: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    run_and_wait_vec(program, &owned)
}

pub fn run_and_wait_vec(program: &str, args: &[String]) -> Result<String, String> {
    let mut cmd = std::process::Command::new(program);
    for a in args {
        cmd.arg(a);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd.output().map_err(|e| format!("{program}: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).to_string()
        + &String::from_utf8_lossy(&out.stderr);
    if out.status.success() {
        Ok(text)
    } else {
        Err(text)
    }
}

pub fn relaunch_as_admin() -> bool {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let exe_w = widen(&exe.to_string_lossy());
    let verb = widen("runas");
    let rc = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            exe_w.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    rc as isize > 32
}

pub fn open_path(path: &str) {
    let _ = std::process::Command::new("explorer")
        .arg(path)
        .spawn();
}

pub fn pick_folder() -> Result<String, String> {
    let script = "Add-Type -AssemblyName System.Windows.Forms; $owner = New-Object System.Windows.Forms.Form; $owner.TopMost = $true; $d = New-Object System.Windows.Forms.FolderBrowserDialog; $d.Description = 'Select folder'; if ($d.ShowDialog($owner) -eq [System.Windows.Forms.DialogResult]::OK) { Write-Output $d.SelectedPath }; $owner.Dispose()";
    run_and_wait("powershell", &["-NoProfile", "-STA", "-Command", script])
        .map(|s| s.trim().to_string())
}

pub fn pick_file() -> Result<String, String> {
    let script = "Add-Type -AssemblyName System.Windows.Forms; $owner = New-Object System.Windows.Forms.Form; $owner.TopMost = $true; $d = New-Object System.Windows.Forms.OpenFileDialog; $d.Filter = 'All files|*.*'; if ($d.ShowDialog($owner) -eq [System.Windows.Forms.DialogResult]::OK) { Write-Output $d.FileName }; $owner.Dispose()";
    run_and_wait("powershell", &["-NoProfile", "-STA", "-Command", script])
        .map(|s| s.trim().to_string())
}

pub fn reveal_path(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    if p.is_file() {
        let parent = p.parent().map(|x| x.to_string_lossy().to_string());
        if let Some(parent) = parent {
            let _ = std::process::Command::new("explorer")
                .arg(format!("/select,{path}"))
                .spawn();
            let _ = parent;
            return Ok(());
        }
    }
    open_path(path);
    Ok(())
}

pub fn disk_space(path: &str) -> (u64, u64, u64) {
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let w = widen(path);
    let mut free_avail: u64 = 0;
    let mut total: u64 = 0;
    let mut total_free: u64 = 0;
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            w.as_ptr(),
            &mut free_avail,
            &mut total,
            &mut total_free,
        )
    };
    if ok == 0 {
        (0, 0, 0)
    } else {
        (total, total - total_free, total_free)
    }
}

/// Capture the window currently under the cursor: returns (process name, window title, exe path).
pub fn hunter_capture() -> (String, String, String) {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetWindowTextW, WindowFromPoint};
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    use windows_sys::Win32::System::ProcessStatus::K32GetModuleBaseNameW;

    let mut pt = POINT { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut pt) };
    if ok == 0 {
        return (String::new(), String::new(), String::new());
    }
    let hwnd = unsafe { WindowFromPoint(pt) };
    if hwnd.is_null() {
        return (String::new(), String::new(), String::new());
    }
    // window title
    let mut title_buf = vec![0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32) };
    let title = String::from_utf16_lossy(&title_buf[..len as usize]).to_string();

    // pid
    let mut pid: u32 = 0;
    unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, &mut pid) };
    if pid == 0 {
        return (String::new(), title, String::new());
    }
    let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid) };
    if handle.is_null() {
        return (String::new(), title, String::new());
    }
    let mut name_buf = vec![0u16; 512];
    let n = unsafe { K32GetModuleBaseNameW(handle, std::ptr::null_mut(), name_buf.as_mut_ptr(), name_buf.len() as u32) };
    let name = if n > 0 { String::from_utf16_lossy(&name_buf[..n as usize]).to_string() } else { String::new() };
    // exe path via QueryFullProcessImageNameW
    let mut path_buf = vec![0u16; 1024];
    let mut sz = path_buf.len() as u32;
    let qok = unsafe {
        windows_sys::Win32::System::Threading::QueryFullProcessImageNameW(handle, 0, path_buf.as_mut_ptr(), &mut sz)
    };
    let path = if qok != 0 { String::from_utf16_lossy(&path_buf[..sz as usize]).to_string() } else { String::new() };
    unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
    (name, title, path)
}

pub fn drives() -> Vec<String> {
    use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;
    let mask = unsafe { GetLogicalDrives() };
    let mut out = Vec::new();
    for i in 0..26u32 {
        if mask & (1 << i) != 0 {
            let letter = (b'A' + i as u8) as char;
            out.push(format!("{letter}:\\"));
        }
    }
    out
}

pub fn file_time_to_unix(ft: u64) -> u64 {
    // FILETIME: 100ns intervals since 1601-01-01
    if ft < 116_444_736_000_000_000 {
        return 0;
    }
    (ft - 116_444_736_000_000_000) / 10_000_000
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn exe_path() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Human readable size using binary units.
pub fn fmt_bytes(n: u64) -> String {
    const KB: f64 = 1024.0;
    let v = n as f64;
    if v >= KB * KB * KB {
        format!("{:.2} GB", v / (KB * KB * KB))
    } else if v >= KB * KB {
        format!("{:.1} MB", v / (KB * KB))
    } else if v >= KB {
        format!("{:.1} KB", v / KB)
    } else {
        format!("{n} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_lock_excludes_concurrent_operations() {
        let _guard = try_op_lock().expect("first lock should succeed");
        assert!(try_op_lock().is_none(), "second lock must fail while first is held");
    }
}
