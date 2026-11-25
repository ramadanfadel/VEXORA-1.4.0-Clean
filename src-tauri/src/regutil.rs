//! Thin helpers around the Windows registry (read / enumerate / delete).

use winreg::enums::*;
use winreg::RegKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hive {
    Hkcu,
    Hklm,
    Hkcr,
    Hku,
}

pub fn predef(hive: Hive) -> RegKey {
    match hive {
        Hive::Hkcu => RegKey::predef(HKEY_CURRENT_USER),
        Hive::Hklm => RegKey::predef(HKEY_LOCAL_MACHINE),
        Hive::Hkcr => RegKey::predef(HKEY_CLASSES_ROOT),
        Hive::Hku => RegKey::predef(HKEY_USERS),
    }
}

pub fn split_path(full: &str) -> Option<(Hive, String)> {
    let (head, rest) = full.split_once('\\')?;
    let hive = match head.to_uppercase().as_str() {
        "HKEY_CURRENT_USER" | "HKCU" => Hive::Hkcu,
        "HKEY_LOCAL_MACHINE" | "HKLM" => Hive::Hklm,
        "HKEY_CLASSES_ROOT" | "HKCR" => Hive::Hkcr,
        "HKEY_USERS" | "HKU" => Hive::Hku,
        _ => return None,
    };
    Some((hive, rest.to_string()))
}

pub fn open(full: &str) -> Option<RegKey> {
    let (hive, path) = split_path(full)?;
    let root = predef(hive);
    for flags in [KEY_READ | KEY_WOW64_64KEY, KEY_READ] {
        let flags = if hive == Hive::Hkcu {
            KEY_READ
        } else {
            flags
        };
        if let Ok(k) = root.open_subkey_with_flags(&path, flags) {
            return Some(k);
        }
    }
    None
}

pub fn open_write(full: &str) -> Option<RegKey> {
    let (hive, path) = split_path(full)?;
    let root = predef(hive);
    let flags = if hive == Hive::Hkcu {
        KEY_ALL_ACCESS
    } else {
        KEY_ALL_ACCESS | KEY_WOW64_64KEY
    };
    root.open_subkey_with_flags(&path, flags).ok()
}

pub fn value_names(key: &RegKey) -> Vec<String> {
    key.enum_values()
        .flatten()
        .map(|(n, _)| n)
        .filter(|n| !n.is_empty())
        .collect()
}

pub fn value_string(key: &RegKey, name: &str) -> String {
    key.get_value::<String, _>(name)
        .or_else(|_| key.get_value::<String, _>(name).map(|_| String::new()))
        .unwrap_or_default()
}

pub fn value_u64(key: &RegKey, name: &str) -> u64 {
    key.get_value::<u32, _>(name).map(|v| v as u64).unwrap_or(0)
}

pub fn subkey_count(key: &RegKey) -> usize {
    key.enum_keys().count()
}

/// Enumerate values of a registry path (one level), returning (name, data).
pub fn enum_values_at(full: &str) -> Vec<(String, String)> {
    match open(full) {
        Some(k) => k
            .enum_values()
            .flatten()
            .filter(|(n, _)| !n.is_empty())
            .map(|(n, v)| {
                let data = match v {
                    winreg::RegValue { bytes, vtype } => {
                        if vtype == REG_SZ || vtype == REG_EXPAND_SZ || vtype == REG_MULTI_SZ {
                            decode_sz(&bytes)
                        } else {
                            String::new()
                        }
                    }
                };
                (n, data)
            })
            .collect(),
        None => Vec::new(),
    }
}

fn decode_sz(bytes: &[u8]) -> String {
    let mut units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    while units.last() == Some(&0) {
        units.pop();
    }
    String::from_utf16_lossy(&units)
}

pub fn delete_value(full: &str, name: &str) -> bool {
    if let Some(k) = open_write(full) {
        return k.delete_value(name).is_ok();
    }
    false
}

pub fn delete_subkey(parent: &str, name: &str) -> bool {
    if let Some(k) = open_write(parent) {
        return k.delete_subkey_all(name).is_ok();
    }
    false
}

pub fn set_string(full: &str, name: &str, value: &str) -> bool {
    if let Some(k) = open_write(full) {
        return k.set_value(name, &value.to_string()).is_ok();
    }
    false
}

pub fn create_key(full: &str) -> Option<RegKey> {
    let (hive, path) = split_path(full)?;
    let root = predef(hive);
    let flags = if hive == Hive::Hkcu {
        KEY_ALL_ACCESS
    } else {
        KEY_ALL_ACCESS | KEY_WOW64_64KEY
    };
    root.create_subkey_with_flags(&path, flags).ok().map(|(k, _)| k)
}

/// Expand a REG_EXPAND_SZ style command (used for startup entries).
pub fn expand_env(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();
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
