//! Context menu handler manager (read/disable/enable) via registry.

use serde::Serialize;
use winreg::enums::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, KEY_CREATE_SUB_KEY, KEY_SET_VALUE};
use winreg::RegKey;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CtxItem {
    pub scope: String,
    pub name: String,
    pub clsid: String,
    pub friendly: String,
    pub enabled: bool,
}

const SCOPES: [&str; 3] = [
    r"*\shellex\ContextMenuHandlers",
    r"Directory\shellex\ContextMenuHandlers",
    r"Directory\Background\shellex\ContextMenuHandlers",
];

const BACKUP_ROOT: &str = r"Software\VEXORA\ContextMenuDisabled";

fn friendly_name(clsid: &str) -> String {
    if clsid.is_empty() { return String::new(); }
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let path = format!(r"CLSID\{}\ (Default)", clsid); // placeholder, replaced below
    let _ = path;
    let keypath = format!(r"CLSID\{}", clsid);
    if let Ok(k) = hkcr.open_subkey_with_flags(&keypath, KEY_READ) {
        if let Ok(v) = k.get_value::<String, _>("") {
            if !v.trim().is_empty() { return v; }
        }
    }
    String::new()
}

pub fn list() -> Vec<CtxItem> {
    let mut out = Vec::new();
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    // enabled entries
    for scope in SCOPES.iter() {
        if let Ok(key) = hkcr.open_subkey_with_flags(scope, KEY_READ) {
            for name in key.enum_keys().filter_map(|k| k.ok()) {
                let clsid = key
                    .open_subkey_with_flags(&name, KEY_READ)
                    .ok()
                    .and_then(|k| k.get_value::<String, _>("").ok())
                    .unwrap_or_default();
                out.push(CtxItem {
                    scope: scope.to_string(),
                    name: name.clone(),
                    clsid: clsid.clone(),
                    friendly: friendly_name(&clsid),
                    enabled: true,
                });
            }
        }
    }
    // disabled (backed up) entries
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(root) = hkcu.open_subkey_with_flags(BACKUP_ROOT, KEY_READ) {
        for scope in SCOPES.iter() {
            let sc = scope.replace('\\', "_");
            if let Ok(k) = root.open_subkey_with_flags(&sc, KEY_READ) {
                for name in k.enum_keys().filter_map(|x| x.ok()) {
                    let sub = match k.open_subkey_with_flags(&name, KEY_READ) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    let clsid = sub.get_value::<String, _>("clsid").unwrap_or_default();
                    out.push(CtxItem {
                        scope: scope.to_string(),
                        name: name.clone(),
                        clsid: clsid.clone(),
                        friendly: friendly_name(&clsid),
                        enabled: false,
                    });
                }
            }
        }
    }
    out
}

pub fn set(scope: &str, name: &str, clsid: &str, enable: bool) -> bool {
    if !SCOPES.contains(&scope) { return false; }
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let full = format!(r"{}\{}", scope, name);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let sc = scope.replace('\\', "_");
    let _backup_scope = format!(r"{}\{}", BACKUP_ROOT, sc);

    if enable {
        // recreate key
        if let Ok((k, _)) = hkcr.create_subkey_with_flags(&full, KEY_WRITE) {
            let _ = k.set_value("", &clsid);
        } else {
            return false;
        }
        // remove backup entry
        if let Ok(root) = hkcu.open_subkey_with_flags(BACKUP_ROOT, KEY_WRITE) {
            if let Ok(k) = root.open_subkey_with_flags(&sc, KEY_WRITE) {
                let _ = k.delete_subkey(name);
            }
        }
        true
    } else {
        // backup clsid
        if let Ok((root, _)) = hkcu.create_subkey_with_flags(BACKUP_ROOT, KEY_WRITE) {
            if let Ok((sk, _)) = root.create_subkey_with_flags(&sc, KEY_WRITE) {
                if let Ok((bk, _)) = sk.create_subkey_with_flags(name, KEY_WRITE) {
                    let _ = bk.set_value("clsid", &clsid);
                }
            }
        }
        // delete original
        if let Ok(parent) = hkcr.open_subkey_with_flags(scope, KEY_WRITE) {
            let _ = parent.delete_subkey(name);
            true
        } else {
            false
        }
    }
}

pub fn win11_classic(enabled: bool) -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32";
    if enabled {
        if let Ok((k, _)) = hkcu.create_subkey_with_flags(path, KEY_WRITE | KEY_CREATE_SUB_KEY | KEY_SET_VALUE) {
            let _ = k.set_value("", &"");
            return true;
        }
    } else {
        // remove the whole CLSID override
        let parent = r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}";
        let _ = hkcu.delete_subkey_all(parent);
        return true;
    }
    false
}

pub fn win11_classic_status() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32";
    hkcu.open_subkey_with_flags(path, KEY_READ).is_ok()
}
