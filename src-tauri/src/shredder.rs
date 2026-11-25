//! File unlocker (Restart Manager) + secure shredder.

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LockingProcess {
    pub pid: u32,
    pub name: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShredResult {
    pub ok: bool,
    pub message: String,
}

/// Find processes locking a file via Restart Manager.
pub fn locking_processes(path: &str) -> Vec<LockingProcess> {
    let mut out = Vec::new();
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        // FFI
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct RmUniqueProcess {
            dw_process_id: u32,
            process_start_time: windows_sys::Win32::Foundation::FILETIME,
        }
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct RmProcessInfo {
            process: RmUniqueProcess,
            str_app_name: [u16; 256],
            str_service_short_name: [u16; 64],
            application_type: i32,
            app_status: u32,
            tssession_id: u32,
            b_restartable: i32,
        }
        #[link(name = "rstrtmgr")]
        extern "system" {
            fn RmStartSession(pSessionHandle: *mut u32, dwSessionFlags: u32, strSessionKey: *mut u16) -> i32;
            fn RmEndSession(dwSessionHandle: u32) -> i32;
            fn RmRegisterResources(
                dwSessionHandle: u32,
                nFiles: u32,
                rgsFileNames: *const *const u16,
                nApplications: u32,
                rgApplications: *const std::ffi::c_void,
                nServices: u32,
                rgsServiceNames: *const *const u16,
            ) -> i32;
            fn RmGetList(
                dwSessionHandle: u32,
                pnProcInfoNeeded: *mut u32,
                pnProcInfo: *mut u32,
                rgAffectedApps: *mut RmProcessInfo,
                lpdwRebootReasons: *mut u32,
            ) -> i32;
        }

        // session key buffer (must be RM_SESSION_KEY_LEN = 32 chars incl null? use 33)
        let mut key = vec![0u16; 33];
        let mut session: u32 = 0;
        unsafe {
            if RmStartSession(&mut session, 0, key.as_mut_ptr()) != 0 {
                return out;
            }
            // register the file
            let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();
            let ptr = wide.as_ptr();
            let arr = [ptr];
            if RmRegisterResources(session, 1, arr.as_ptr(), 0, std::ptr::null(), 0, std::ptr::null()) != 0 {
                RmEndSession(session);
                return out;
            }
            // get list
            let mut needed: u32 = 0;
            let mut count: u32 = 0;
            let mut reboot: u32 = 0;
            // first call to get count
            RmGetList(session, &mut needed, &mut count, std::ptr::null_mut(), &mut reboot);
            if needed == 0 {
                RmEndSession(session);
                return out;
            }
            let mut buf = vec![RmProcessInfo {
                process: RmUniqueProcess { dw_process_id: 0, process_start_time: windows_sys::Win32::Foundation::FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 } },
                str_app_name: [0; 256],
                str_service_short_name: [0; 64],
                application_type: 0,
                app_status: 0,
                tssession_id: 0,
                b_restartable: 0,
            }; needed as usize];
            count = needed;
            if RmGetList(session, &mut needed, &mut count, buf.as_mut_ptr(), &mut reboot) == 0 {
                for i in 0..count as usize {
                    let pid = buf[i].process.dw_process_id;
                    let name = String::from_utf16_lossy(&buf[i].str_app_name)
                        .trim_end_matches('\0')
                        .to_string();
                    if pid != 0 {
                        out.push(LockingProcess { pid, name });
                    }
                }
            }
            RmEndSession(session);
        }
    }
    out
}

/// Securely shred a file (or directory) with N passes then delete.
pub fn shred(path: &str, passes: u32) -> ShredResult {
    use std::fs::{File, OpenOptions};
    use std::io::{Seek, SeekFrom, Write};
    let p = std::path::Path::new(path);
    if !p.exists() {
        return ShredResult { ok: false, message: "File not found".into() };
    }
    // Security: resolve symlinks/junctions and reject protected/reparse paths.
    if crate::safety::is_reparse_point(p) {
        return ShredResult { ok: false, message: "reparse point (junction/symlink) not allowed".into() };
    }
    let canon = match p.canonicalize() {
        Ok(c) => c,
        Err(e) => return ShredResult { ok: false, message: format!("cannot resolve path: {e}") },
    };
    if crate::safety::is_reparse_point(&canon) || crate::safety::is_protected(&canon) || !crate::safety::clean_path_allowed(&canon) {
        return ShredResult { ok: false, message: "protected path".into() };
    }
    let p = canon.as_path();
    let passes = passes.clamp(1, 3);
    let is_dir = p.is_dir();

    if is_dir {
        // shred each file recursively
        if let Ok(walker) = walkdir::WalkDir::new(p).into_iter().collect::<Result<Vec<_>, _>>() {
            for e in walker {
                if e.file_type().is_file() {
                    let _ = shred_file(e.path(), passes);
                }
            }
        }
        let _ = std::fs::remove_dir_all(p);
        return ShredResult { ok: true, message: "Directory shredded".into() };
    }

    match shred_file(p, passes) {
        Ok(()) => ShredResult { ok: true, message: "Shredded".into() },
        Err(e) => ShredResult { ok: false, message: e },
    }
}

fn shred_file(path: &std::path::Path, passes: u32) -> Result<(), String> {
    use std::fs::{File, OpenOptions};
    use std::io::{Seek, SeekFrom, Write};
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let size = meta.len();
    let mut file = OpenOptions::new().write(true).open(path).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 1024 * 1024];
    for pass in 0..passes {
        // pattern: pass 0 zeros, pass 1 0xFF, pass 2 random
        file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        let mut remaining = size;
        while remaining > 0 {
            let n = std::cmp::min(remaining, buf.len() as u64) as usize;
            match pass {
                0 => for b in &mut buf[..n] { *b = 0x00; },
                1 => for b in &mut buf[..n] { *b = 0xFF; },
                _ => {
                    // simple PRNG fill
                    let mut x: u64 = 0x9E3779B97F4A7C15;
                    for b in &mut buf[..n] {
                        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
                        *b = (x >> 32) as u8;
                    }
                }
            }
            file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            remaining -= n as u64;
        }
        file.flush().map_err(|e| e.to_string())?;
        let _ = file.sync_all();
    }
    drop(file);
    std::fs::remove_file(path).map_err(|e| e.to_string())?;
    Ok(())
}
