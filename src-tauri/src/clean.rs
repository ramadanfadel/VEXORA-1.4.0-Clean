//! Deletion engine: recycle / permanent / shred, with per-cleaner reporting.

use serde::Serialize;

use crate::catalog::{Cleaner, Kind};
use crate::regutil;
use crate::scan;
use crate::util;

#[derive(Clone, Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CleanOutcome {
    pub removed: u64,
    pub freed: u64,
    pub skipped: u64,
    pub errors: Vec<String>,
}

pub struct CleanOptions {
    pub recycle: bool,
    pub shred: bool,
    pub passes: u32,
    pub exclusions: Vec<String>,
}

pub fn clean_one(c: &Cleaner, opts: &CleanOptions) -> CleanOutcome {
    let mut out = CleanOutcome::default();
    match c.kind {
        Kind::Files => {
            let scanned = scan::scan_files(c, &opts.exclusions, 300_000);
            out.freed = 0;
            let paths: Vec<String> = scanned.items.iter().map(|i| i.path.clone()).collect();
            let size_of: std::collections::HashMap<String, u64> = scanned
                .items
                .iter()
                .map(|i| (i.path.clone(), i.size))
                .collect();

            if opts.shred {
                for p in &paths {
                    if !crate::safety::clean_path_allowed(std::path::Path::new(p)) {
                        out.skipped += 1;
                        continue;
                    }
                    if util::shred_path(std::path::Path::new(p), opts.passes.max(1)) {
                        out.removed += 1;
                        out.freed += size_of.get(p).copied().unwrap_or(0);
                    } else {
                        out.skipped += 1;
                        if out.errors.len() < 8 {
                            out.errors.push(p.clone());
                        }
                    }
                }
            } else if opts.recycle {
                // Default policy: cleaning sends files to the Recycle Bin so
                // mistakes stay recoverable. If the shell refuses a file
                // (locked temp file), it is skipped and reported instead of
                // being deleted without a trace.
                for chunk in paths.chunks(100) {
                    if util::recycle_batch(chunk).is_ok() {
                        for p in chunk {
                            out.removed += 1;
                            out.freed += size_of.get(p).copied().unwrap_or(0);
                        }
                    } else {
                        for p in chunk {
                            if !crate::safety::clean_path_allowed(std::path::Path::new(p)) {
                                out.skipped += 1;
                                continue;
                            }
                            if util::recycle_batch(std::slice::from_ref(p)).is_ok() {
                                out.removed += 1;
                                out.freed += size_of.get(p).copied().unwrap_or(0);
                            } else {
                                out.skipped += 1;
                                if out.errors.len() < 8 {
                                    out.errors.push(p.clone());
                                }
                            }
                        }
                    }
                }
            } else {
                // "Use recycle bin" is switched off in settings: delete for
                // real, still guarded by the safety allow-list.
                for p in &paths {
                    if !crate::safety::clean_path_allowed(std::path::Path::new(p)) {
                        out.skipped += 1;
                        continue;
                    }
                    if util::delete_path(std::path::Path::new(p)) {
                        out.removed += 1;
                        out.freed += size_of.get(p).copied().unwrap_or(0);
                    } else {
                        out.skipped += 1;
                        if out.errors.len() < 8 {
                            out.errors.push(p.clone());
                        }
                    }
                }
            }
        }
        Kind::RegistryValues => {
            let scanned = scan::scan_registry_values(c);
            let key = c.reg_key.clone().unwrap_or_default();
            for item in scanned.items {
                let name = item
                    .path
                    .rsplit('\\')
                    .next()
                    .unwrap_or_default()
                    .to_string();
                if regutil::delete_value(&key, &name) {
                    out.removed += 1;
                    out.freed += item.size;
                } else {
                    out.skipped += 1;
                    if out.errors.len() < 8 {
                        out.errors.push(item.path.clone());
                    }
                }
            }
        }
        Kind::Recycle => {
            // Query the bin BEFORE emptying it, otherwise the counts are
            // always zero and the user thinks nothing happened.
            let before = scan::scan_recycle_bin();
            match util::empty_recycle_bin() {
                Ok(()) => {
                    out.removed = before.count;
                    out.freed = before.size;
                }
                Err(e) => out.errors.push(e),
            }
        }
        Kind::Clipboard => {
            if let Err(e) = util::clear_clipboard() {
                out.errors.push(e);
            } else {
                out.removed = 1;
            }
        }
        Kind::Command => {
            if c.cmd.is_empty() {
                return out;
            }
            if c.cmd[0].eq_ignore_ascii_case("wevtutil") {
                // wevtutil cl <log> for each log listed after "el"
                let logs: Vec<String> = if c.cmd.len() > 2 && c.cmd[1] == "el" {
                    c.cmd[2..].to_vec()
                } else {
                    vec!["System".into()]
                };
                for log in logs {
                    match util::run_and_wait("wevtutil", &["cl", &log]) {
                        Ok(_) => out.removed += 1,
                        Err(e) => {
                            if out.errors.len() < 6 {
                                out.errors.push(e)
                            }
                        }
                    }
                }
            } else {
                let program = c.cmd[0].clone();
                let args: Vec<String> = c.cmd[1..].to_vec();
                match util::run_command(&program, &args) {
                    Ok(()) => out.removed += 1,
                    Err(e) => out.errors.push(e),
                }
            }
        }
    }
    out
}

pub fn clean_many(ids: &[String], opts: &CleanOptions) -> Vec<(String, CleanOutcome)> {
    let catalog = crate::catalog::cleaners();
    let mut results = Vec::new();
    for id in ids {
        if let Some(c) = catalog.iter().find(|c| &c.id == id) {
            let r = clean_one(c, opts);
            results.push((id.clone(), r));
        }
    }
    results
}
