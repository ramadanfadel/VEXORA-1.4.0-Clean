//! Real parallel duplicate scanner. Unlike the removed experimental adapter,
//! this fills full groups from verified BLAKE3 hashes — no placeholder data.

use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;

use crate::tools::{DupGroup, DupResult};

/// Full-file BLAKE3 hash. Returns None for unreadable files.
pub fn hash_file(path: &Path) -> Option<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 128 * 1024];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Some(hasher.finalize().to_hex().to_string())
}

fn collect(roots: &[String], min_size: u64) -> (Vec<(String, u64)>, u64) {
    let mut files = Vec::new();
    let mut errors = 0u64;
    let mut seen = std::collections::HashSet::new();
    for root in roots {
        if root.trim().is_empty() {
            continue;
        }
        let p = Path::new(root);
        if crate::safety::is_reparse_point(p) || !crate::safety::clean_path_allowed(p) {
            continue;
        }
        for entry in walkdir::WalkDir::new(p).follow_links(false) {
            match entry {
                Ok(e) => {
                    if !e.file_type().is_file() || e.file_type().is_symlink() {
                        continue;
                    }
                    let path = e.path();
                    if crate::safety::is_reparse_point(path)
                        || !crate::safety::clean_path_allowed(path)
                    {
                        continue;
                    }
                    let size = match e.metadata() {
                        Ok(m) => m.len(),
                        Err(_) => {
                            errors += 1;
                            continue;
                        }
                    };
                    if size < min_size {
                        continue;
                    }
                    let text = path.to_string_lossy().to_string();
                    if seen.insert(text.to_lowercase()) {
                        files.push((text, size));
                    }
                }
                Err(_) => errors += 1,
            }
        }
    }
    (files, errors)
}

/// Parallel scan: group by size first, hash only candidate files.
pub fn scan(roots: &[String], min_size: u64) -> DupResult {
    let (files, errors) = collect(roots, min_size);
    let scanned = files.len() as u64;
    let mut by_size: HashMap<u64, Vec<String>> = HashMap::new();
    for (path, size) in files {
        by_size.entry(size).or_default().push(path);
    }

    let pairs: Vec<(u64, String)> = by_size
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .flat_map(|(size, paths)| {
            paths
                .into_iter()
                .map(move |p| (size, p))
                .collect::<Vec<_>>()
        })
        .collect();
    let candidates: Vec<(u64, (String, String))> = pairs
        .par_iter()
        .filter_map(|(size, path)| {
            hash_file(Path::new(path)).map(|h| {
                let mut key = h;
                key.push('|');
                key.push_str(&size.to_string());
                (*size, (key, path.clone()))
            })
        })
        .collect();

    let mut by_hash: HashMap<String, (u64, Vec<String>)> = HashMap::new();
    for (size, (key, path)) in candidates {
        by_hash
            .entry(key)
            .or_insert((size, Vec::new()))
            .1
            .push(path);
    }

    let mut groups: Vec<DupGroup> = by_hash
        .into_iter()
        .filter(|(_, (_, paths))| paths.len() > 1)
        .map(|(key, (size, mut paths))| {
            paths.sort();
            DupGroup {
                size,
                hash: key.split('|').next().unwrap_or("").to_string(),
                files: paths,
            }
        })
        .collect();
    groups.sort_by(|a, b| b.size.cmp(&a.size));
    let reclaimable = groups
        .iter()
        .map(|g| g.size * (g.files.len().saturating_sub(1) as u64))
        .sum();
    DupResult {
        groups,
        reclaimable,
        scanned_files: scanned,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_real_duplicates_in_temp_dir() {
        let dir = std::env::temp_dir().join(format!("vexora_hyper_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let a = dir.join("a.bin");
        let b = dir.join("b.bin");
        let c = dir.join("c.bin");
        std::fs::write(&a, b"same content 1234567890").unwrap();
        std::fs::write(&b, b"same content 1234567890").unwrap();
        std::fs::write(&c, b"different content").unwrap();

        let r = scan(&[dir.to_string_lossy().to_string()], 4);
        assert!(r.scanned_files >= 3);
        assert_eq!(r.groups.len(), 1);
        assert_eq!(r.groups[0].files.len(), 2);
        assert_eq!(r.groups[0].hash.len(), 64);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hash_changes_with_content() {
        let dir = std::env::temp_dir().join(format!("vexora_hash_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("x.bin");
        std::fs::write(&f, b"one").unwrap();
        let h1 = hash_file(&f).unwrap();
        std::fs::write(&f, b"two").unwrap();
        let h2 = hash_file(&f).unwrap();
        assert_ne!(h1, h2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
