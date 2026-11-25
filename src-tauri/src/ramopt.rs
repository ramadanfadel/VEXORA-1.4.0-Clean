//! RAM working-set trimming + standby list purge.

use std::sync::atomic::{AtomicBool, Ordering};

pub static RAM_AUTO_CLEAN: AtomicBool = AtomicBool::new(false);

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RamStatus {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub load: u32,
}

pub fn status() -> RamStatus {
    let (total, avail, load) = memory_info();
    RamStatus {
        total,
        available: avail,
        used: total.saturating_sub(avail),
        load,
    }
}

pub fn clean() -> (u64, u32) {
    let before = available_bytes();
    let mut trimmed = 0u32;
    // Enable privilege (best-effort).
    enable_profile_privilege();
    // Empty working set of every accessible process.
    let pids = enum_pids();
    for pid in pids {
        if pid == 0 { continue; }
        unsafe {
            use windows_sys::Win32::Foundation::CloseHandle;
            use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA};
            let h = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA, 0, pid);
            if !h.is_null() {
                // EmptyWorkingSet from psapi
                #[link(name = "psapi")]
                extern "system" {
                    fn EmptyWorkingSet(hProcess: *mut std::ffi::c_void) -> i32;
                }
                if EmptyWorkingSet(h) != 0 {
                    trimmed += 1;
                }
                CloseHandle(h);
            }
        }
    }
    // Purge standby / flush modified / empty working sets globally.
    purge_standby();
    let after = available_bytes();
    (after.saturating_sub(before), trimmed)
}

fn memory_info() -> (u64, u64, u32) {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct MemoryStatusEx {
            dw_length: u32,
            dw_memory_load: u32,
            ull_total_phys: u64,
            ull_avail_phys: u64,
            ull_total_page_file: u64,
            ull_avail_page_file: u64,
            ull_total_virtual: u64,
            ull_avail_virtual: u64,
            ull_avail_extended_virtual: u64,
        }
        extern "system" {
            fn GlobalMemoryStatusEx(lpBuffer: *mut MemoryStatusEx) -> i32;
        }
        let mut ms = MemoryStatusEx {
            dw_length: std::mem::size_of::<MemoryStatusEx>() as u32,
            dw_memory_load: 0,
            ull_total_phys: 0,
            ull_avail_phys: 0,
            ull_total_page_file: 0,
            ull_avail_page_file: 0,
            ull_total_virtual: 0,
            ull_avail_virtual: 0,
            ull_avail_extended_virtual: 0,
        };
        unsafe {
            if GlobalMemoryStatusEx(&mut ms) != 0 {
                return (ms.ull_total_phys, ms.ull_avail_phys, ms.dw_memory_load);
            }
        }
    }
    (0, 0, 0)
}

fn available_bytes() -> u64 {
    memory_info().1
}

fn enum_pids() -> Vec<u32> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::ProcessStatus::K32EnumProcesses;
    let mut pids = vec![0u32; 4096];
    let mut needed = 0u32;
    unsafe {
        if K32EnumProcesses(pids.as_mut_ptr(), (pids.len() * 4) as u32, &mut needed) != 0 {
            let count = (needed as usize) / 4;
            pids.truncate(count);
        } else {
            pids.clear();
        }
    }
    let _ = CloseHandle;
    pids
}

fn enable_profile_privilege() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::Security::{
            AdjustTokenPrivileges, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY,
        };
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
        unsafe {
            let mut token = std::ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token) != 0 {
                let name = "SeProfileSingleProcessPrivilege\0".encode_utf16().collect::<Vec<u16>>();
                let mut luid = windows_sys::Win32::Foundation::LUID { LowPart: 0, HighPart: 0 };
                if LookupPrivilegeValueW(std::ptr::null(), name.as_ptr(), &mut luid) != 0 {
                    #[repr(C)]
                    struct Tp {
                        count: u32,
                        luid: windows_sys::Win32::Foundation::LUID,
                        attrs: u32,
                    }
                    let mut tp = Tp { count: 1, luid, attrs: SE_PRIVILEGE_ENABLED };
                    AdjustTokenPrivileges(token, 0, &mut tp as *mut _ as *mut _, 0, std::ptr::null_mut(), std::ptr::null_mut());
                }
                CloseHandle(token);
            }
        }
    }
}

fn purge_standby() {
    #[cfg(windows)]
    {
        #[link(name = "ntdll")]
        extern "system" {
            fn NtSetSystemInformation(
                SystemInformationClass: i32,
                SystemInformation: *mut std::ffi::c_void,
                SystemInformationLength: u32,
            ) -> i32;
        }
        for cmd in [1i32, 2, 3, 4] {
            let mut c = cmd;
            unsafe {
                NtSetSystemInformation(80, &mut c as *mut _ as *mut _, 4);
            }
        }
    }
}
