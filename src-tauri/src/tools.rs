//! Maintenance tools: duplicates, large files, empty folders, disk map, wiping.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;

use crate::safety;
use crate::util;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DupGroup {
    pub size: u64,
    pub hash: String,
    pub files: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DupResult {
    pub groups: Vec<DupGroup>,
    pub reclaimable: u64,
    pub scanned_files: u64,
    pub errors: u64,
}

/// Set to true by the UI to stop a long duplicate scan early.
pub static DUP_CANCEL: AtomicBool = AtomicBool::new(false);

fn walk(root: &Path, mut visit: impl FnMut(&Path, u64)) -> (u64, u64) {
    walk_inner(root, None, &mut visit)
}

fn walk_cancellable(root: &Path, mut visit: impl FnMut(&Path, u64)) -> (u64, u64) {
    walk_inner(root, Some(&DUP_CANCEL), &mut visit)
}

fn walk_inner(
    root: &Path,
    cancel: Option<&AtomicBool>,
    visit: &mut dyn FnMut(&Path, u64),
) -> (u64, u64) {
    let mut files = 0u64;
    let mut errors = 0u64;
    let walker = walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if let Some(c) = cancel {
                if c.load(Ordering::Relaxed) {
                    return false;
                }
            }
            let name = e.file_name().to_string_lossy().to_lowercase();
            !(e.file_type().is_dir()
                && (name == "$recycle.bin"
                    || name == "system volume information"
                    || name == "node_modules"
                    || name == ".git"))
        });
    for entry in walker {
        match entry {
            Ok(e) => {
                if e.file_type().is_file() {
                    if let Ok(meta) = e.metadata() {
                        files += 1;
                        visit(e.path(), meta.len());
                    }
                }
            }
            Err(_) => errors += 1,
        }
    }
    (files, errors)
}

/// Fast partial hash: head + tail + size, FNV-1a 64.
fn partial_hash(path: &Path, size: u64) -> String {
    const CHUNK: usize = 128 * 1024;
    let mut hasher: u64 = 0xcbf29ce484222325;
    let mut buf = vec![0u8; CHUNK];
    if let Ok(mut f) = std::fs::File::open(path) {
        if let Ok(n) = f.read(&mut buf) {
            for b in &buf[..n] {
                hasher ^= *b as u64;
                hasher = hasher.wrapping_mul(0x100000001b3);
            }
        }
        if size > CHUNK as u64 * 2 {
            use std::io::{Seek, SeekFrom};
            if f.seek(SeekFrom::End(-(CHUNK as i64))).is_ok() {
                if let Ok(n) = f.read(&mut buf) {
                    for b in &buf[..n] {
                        hasher ^= *b as u64;
                        hasher = hasher.wrapping_mul(0x100000001b3);
                    }
                }
            }
        }
    }
    format!("{hasher:016x}-{size}")
}

/// Full-content verification hash using SHA-256. Only duplicates that pass
/// this check are ever reported, so identical size + partial hash can never
/// cause a wrong deletion.
fn full_hash(path: &Path) -> Option<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut f = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = f.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Some(format!("{:x}", hasher.finalize()))
}

pub fn find_duplicates<F: Fn(u64)>(roots: &[String], min_size: u64, progress: F) -> DupResult {
    DUP_CANCEL.store(false, Ordering::Relaxed);
    let settings = crate::settings::Settings::load();
    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut scanned = 0u64;
    let mut errors = 0u64;
    let mut tick = 0u64;
    for root in roots {
        if DUP_CANCEL.load(Ordering::Relaxed) {
            break;
        }
        let (s, e) = walk_cancellable(Path::new(root), |path, size| {
            tick += 1;
            if tick % 500 == 0 {
                progress(scanned + tick);
            }
            if size >= min_size.max(1) && !settings.is_excluded(&path.to_string_lossy()) {
                by_size.entry(size).or_default().push(path.to_path_buf());
            }
        });
        scanned += s;
        errors += e;
        progress(scanned);
    }

    let mut groups: Vec<DupGroup> = Vec::new();
    let mut reclaimable = 0u64;
    for (size, files) in by_size {
        if files.len() < 2 {
            continue;
        }
        let mut by_hash: HashMap<String, Vec<String>> = HashMap::new();
        for f in &files {
            let h = partial_hash(f, size);
            by_hash
                .entry(h)
                .or_default()
                .push(f.to_string_lossy().to_string());
        }
        for (_, list) in by_hash {
            if list.len() < 2 {
                continue;
            }
            // Confirm every candidate with a full-content hash before reporting.
            let mut by_full: HashMap<String, Vec<String>> = HashMap::new();
            for p in list {
                match full_hash(Path::new(&p)) {
                    Some(h) => by_full.entry(h).or_default().push(p),
                    None => errors += 1,
                }
            }
            for (hash, mut verified) in by_full {
                if verified.len() < 2 {
                    continue;
                }
                verified.sort();
                reclaimable += size * (verified.len() as u64 - 1);
                groups.push(DupGroup {
                    size,
                    hash,
                    files: verified,
                });
            }
        }
    }
    if groups.len() > 500 {
        groups.truncate(500);
    }
    groups.sort_by(|a, b| (b.size * b.files.len() as u64).cmp(&(a.size * a.files.len() as u64)));
    DupResult {
        groups,
        reclaimable,
        scanned_files: scanned,
        errors,
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BigFile {
    pub path: String,
    pub size: u64,
    pub modified: u64,
}

pub fn big_files(root: &str, min_size: u64, limit: usize) -> Vec<BigFile> {
    let settings = crate::settings::Settings::load();
    let mut files: Vec<BigFile> = Vec::new();
    walk(Path::new(root), |path, size| {
        if size >= min_size && !settings.is_excluded(&path.to_string_lossy()) {
            let modified = std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            files.push(BigFile {
                path: path.to_string_lossy().to_string(),
                size,
                modified,
            });
        }
    });
    files.sort_by(|a, b| b.size.cmp(&a.size));
    files.truncate(limit);
    files
}

pub fn empty_dirs(root: &str, limit: usize) -> Vec<String> {
    let root = Path::new(root);
    let settings = crate::settings::Settings::load();
    let mut dirs: Vec<PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            !settings.is_excluded(&e.path().to_string_lossy())
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            dirs.push(entry.path().to_path_buf());
        }
    }
    dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    let mut out: Vec<String> = Vec::new();
    for d in dirs {
        let is_empty = std::fs::read_dir(&d)
            .map(|mut rd| rd.next().is_none())
            .unwrap_or(false);
        if is_empty {
            out.push(d.to_string_lossy().to_string());
            if out.len() >= limit {
                break;
            }
        }
    }
    out
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapNode {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub children: Vec<MapNode>,
}

pub fn disk_map(root: &str, depth: u32) -> MapNode {
    let settings = crate::settings::Settings::load();
    fn build(path: &Path, depth: u32, level: u32, settings: &crate::settings::Settings) -> MapNode {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let mut node = MapNode {
            name,
            path: path.to_string_lossy().to_string(),
            size: 0,
            is_dir: path.is_dir(),
            children: Vec::new(),
        };
        if !node.is_dir {
            node.size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            return node;
        }
        let rd = match std::fs::read_dir(path) {
            Ok(r) => r,
            Err(_) => return node,
        };
        for entry in rd.flatten() {
            let p = entry.path();
            if settings.is_excluded(&p.to_string_lossy()) {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_dir() {
                if level >= depth {
                    let size = dir_size(&p, 60_000);
                    node.size += size;
                    node.children.push(MapNode {
                        name: p
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        path: p.to_string_lossy().to_string(),
                        size,
                        is_dir: true,
                        children: Vec::new(),
                    });
                } else {
                    let child = build(&p, depth, level + 1, settings);
                    node.size += child.size;
                    node.children.push(child);
                }
            } else {
                node.size += meta.len();
                node.children.push(MapNode {
                    name: p
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    path: p.to_string_lossy().to_string(),
                    size: meta.len(),
                    is_dir: false,
                    children: Vec::new(),
                });
            }
        }
        node.children.sort_by(|a, b| b.size.cmp(&a.size));
        node.children.truncate(80);
        node
    }
    build(Path::new(root), depth.max(1), 0, &settings)
}

fn dir_size(path: &Path, limit: usize) -> u64 {
    let mut total = 0u64;
    let mut count = 0usize;
    for entry in walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            count += 1;
            if count >= limit {
                break;
            }
        }
    }
    total
}

pub fn delete_many(paths: &[String], shred: bool, passes: u32) -> (u64, u64, u64) {
    let settings = crate::settings::Settings::load();
    let mut removed = 0u64;
    let mut freed = 0u64;
    let mut failed = 0u64;
    for p in paths {
        if settings.is_excluded(p) {
            failed += 1;
            continue;
        }
        let path = Path::new(p);
        if safety::is_critical_root(path) || safety::is_reparse_point(path) {
            failed += 1;
            continue;
        }
        let size = if path.is_dir() {
            dir_size(path, 200_000)
        } else {
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        };
        let ok = if shred {
            util::shred_path(path, passes.max(1))
        } else {
            util::delete_path(path)
        };
        if ok {
            removed += 1;
            freed += size;
        } else {
            failed += 1;
        }
    }
    (removed, freed, failed)
}

/// Overwrite the free space on a drive so deleted files cannot be recovered.
pub fn wipe_free_space(
    drive: &str,
    passes: u32,
    cancel: &AtomicBool,
    mut progress: impl FnMut(u64, u64),
) -> Result<u64, String> {
    let (_, _, free) = util::disk_space(drive);
    if free == 0 {
        return Err("drive not ready".into());
    }
    // Keep a safety margin so Windows stays usable while wiping.
    let target = free.saturating_sub(512 * 1024 * 1024);
    if target == 0 {
        return Err("not enough free space".into());
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    candidates.push(Path::new(drive).join("VEXORAWipe.tmp"));
    let temp = std::env::temp_dir();
    if temp.starts_with(drive) {
        candidates.push(temp.join("VEXORAWipe.tmp"));
    }

    let mut written_total = 0u64;
    let chunk = vec![0xA5u8; 4 * 1024 * 1024];

    for pass in 0..passes.max(1) {
        progress(written_total, target * passes.max(1) as u64);
        let mut written_pass = 0u64;
        'outer: for attempt in 0..2 {
            let path = match candidates.get(attempt) {
                Some(p) => p.clone(),
                None => break,
            };
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path);
            let mut file = match file {
                Ok(f) => f,
                Err(_) => continue,
            };
            use std::io::Write;
            loop {
                if cancel.load(Ordering::Relaxed) {
                    drop(file);
                    let _ = std::fs::remove_file(&path);
                    return Ok(written_total);
                }
                if written_pass >= target {
                    break;
                }
                if file.write_all(&chunk).is_err() {
                    break;
                }
                written_pass += chunk.len() as u64;
                written_total += chunk.len() as u64;
                progress(written_total, target * passes.max(1) as u64);
                if pass + 1 < passes.max(1) {
                    // only keep filling on the last pass bookkeeping; the file is
                    // truncated for each pass so we just stop at the target
                }
            }
            let _ = file.flush();
            drop(file);
            let _ = std::fs::remove_file(&path);
            if written_pass >= target {
                break 'outer;
            }
        }
        if pass + 1 < passes.max(1) {
            // Remove any leftovers before the next pass.
            for c in &candidates {
                let _ = std::fs::remove_file(c);
            }
        }
    }
    Ok(written_total)
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimilarImage {
    pub path: String,
    pub size: u64,
    pub modified: u64,
    pub width: u32,
    pub height: u32,
    /// Small base64 PNG preview so the UI can show thumbnails.
    pub thumb: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimilarImageGroup {
    pub images: Vec<SimilarImage>,
    pub difference: u32,
}

pub fn find_similar_images(root: &str, max_diff: u32) -> Vec<SimilarImageGroup> {
    use image::GenericImageView;
    use image_hasher::{FilterType, HashAlg, HasherConfig};

    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Mean)
        .hash_size(16, 16)
        .resize_filter(FilterType::Lanczos3)
        .to_hasher();

    let mut entries: Vec<(SimilarImage, Vec<u8>)> = Vec::new();
    let image_exts = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif"];
    let walker = walkdir::WalkDir::new(root).follow_links(false).into_iter().filter_map(|e| e.ok());
    for entry in walker {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if !image_exts.contains(&ext.as_str()) {
            continue;
        }
        let img = match image::open(path) {
            Ok(i) => i,
            Err(_) => continue,
        };
        let (w, h) = img.dimensions();
        let hash = hasher.hash_image(&img);
        let bytes = hash.as_bytes().to_vec();
        let thumb = make_thumb(&img);
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let modified = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        entries.push((
            SimilarImage {
                path: path.to_string_lossy().to_string(),
                size,
                modified,
                width: w,
                height: h,
                thumb,
            },
            bytes,
        ));
    }

    let mut groups: Vec<(Vec<usize>, u32)> = Vec::new();
    let n = entries.len();
    let mut used = vec![false; n];
    for i in 0..n {
        if used[i] {
            continue;
        }
        let mut group = vec![i];
        used[i] = true;
        let mut max_d = 0u32;
        for j in (i + 1)..n {
            if used[j] {
                continue;
            }
            let d = hamming(&entries[i].1, &entries[j].1);
            if d <= max_diff {
                group.push(j);
                used[j] = true;
                if d > max_d {
                    max_d = d;
                }
            }
        }
        if group.len() >= 2 {
            groups.push((group, max_d));
        }
    }

    groups
        .into_iter()
        .map(|(idxs, d)| SimilarImageGroup {
            images: idxs.into_iter().map(|i| entries[i].0.clone()).collect(),
            difference: d,
        })
        .collect()
}
fn hamming(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}

/// Build a small base64 PNG preview for the similar-images tab.
fn make_thumb(img: &image::DynamicImage) -> String {
    use std::io::Cursor;
    let small = img.thumbnail(80, 80);
    let mut buf = Cursor::new(Vec::new());
    if small.write_to(&mut buf, image::ImageFormat::Png).is_err() {
        return String::new();
    }
    b64_encode(&buf.into_inner())
}

/// Minimal standard base64 encoder (no external crate needed).
fn b64_encode(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(T[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(T[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

// ---------------------------------------------------------------- adobe cache
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CacheItem {
    pub path: String,
    pub size: u64,
    pub kind: String,
}

pub fn adobe_running() -> bool {
    let names = [
        "photoshop", "premiere", "afterfx", "illustrator", "media encoder",
        "adobe", "lightroom", "audition", "indesign",
    ];
    for p in crate::boost::process_list(2000) {
        let lower = p.name.to_lowercase();
        if names.iter().any(|n| lower.contains(n)) {
            return true;
        }
    }
    false
}

fn dir_size_full(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(walker) = walkdir::WalkDir::new(path).into_iter().collect::<Result<Vec<_>, _>>() {
        for e in walker {
            if e.file_type().is_file() {
                if let Ok(m) = e.metadata() {
                    total += m.len();
                }
            }
        }
    }
    total
}

pub fn adobe_cache_items() -> Vec<CacheItem> {
    let mut out = Vec::new();
    let mut add = |path: std::path::PathBuf, kind: &str| {
        if path.exists() {
            let size = if path.is_dir() { dir_size_full(&path) } else { std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) };
            if size > 0 {
                out.push(CacheItem { path: path.to_string_lossy().to_string(), size, kind: kind.to_string() });
            }
        }
    };

    if let Ok(appdata) = std::env::var("APPDATA") {
        let base = std::path::PathBuf::from(appdata).join("Adobe").join("Common");
        add(base.join("Media Cache Files"), "mediaCacheFiles");
        add(base.join("Media Cache"), "mediaCache");
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let adobe = std::path::PathBuf::from(&local).join("Adobe");
        if let Ok(rd) = std::fs::read_dir(&adobe) {
            for e in rd.flatten() {
                let p = e.path();
                if !p.is_dir() { continue; }
                let name = e.file_name().to_string_lossy().to_lowercase();
                if name.contains("autorecover") || name.contains("cache") || name.contains("preview") || name.contains("media") {
                    add(p, "adobeLocal");
                }
            }
        }
        // thumbcache files
        let explorer = std::path::PathBuf::from(&local).join("Microsoft").join("Windows").join("Explorer");
        if let Ok(rd) = std::fs::read_dir(&explorer) {
            for e in rd.flatten() {
                let p = e.path();
                let name = e.file_name().to_string_lossy().to_lowercase();
                if name.starts_with("thumbcache_") && name.ends_with(".db") {
                    add(p, "thumbcache");
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------- drivers / runtimes
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub name: String,
    pub installed: bool,
    pub version: String,
    pub winget_id: String,
}

pub fn runtimes_status() -> Vec<RuntimeStatus> {
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;
    let mut out = Vec::new();

    // VC++ 2015-2022 x64
    let mut vc_x64 = RuntimeStatus {
        name: "Visual C++ Redistributable 2015-2022 (x64)".into(),
        installed: false,
        version: String::new(),
        winget_id: "Microsoft.VCRedist.2015+.x64".into(),
    };
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(k) = hkcu.open_subkey_with_flags(r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64", KEY_READ) {
        vc_x64.installed = true;
        vc_x64.version = k.get_value::<String, _>("Version").unwrap_or_default();
        if vc_x64.version.is_empty() {
            if let Ok(v) = k.get_value::<u32, _>("Installed") { if v == 1 { vc_x64.version = "installed".into(); } }
        }
    }
    out.push(vc_x64);

    // VC++ x86
    let mut vc_x86 = RuntimeStatus {
        name: "Visual C++ Redistributable 2015-2022 (x86)".into(),
        installed: false,
        version: String::new(),
        winget_id: "Microsoft.VCRedist.2015+.x86".into(),
    };
    if let Ok(k) = hkcu.open_subkey_with_flags(r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x86", KEY_READ) {
        vc_x86.installed = true;
        vc_x86.version = k.get_value::<String, _>("Version").unwrap_or_default();
        if vc_x86.version.is_empty() {
            if let Ok(v) = k.get_value::<u32, _>("Installed") { if v == 1 { vc_x86.version = "installed".into(); } }
        }
    }
    out.push(vc_x86);

    // .NET Desktop Runtime (x64)
    let mut net = RuntimeStatus {
        name: ".NET Desktop Runtime (x64)".into(),
        installed: false,
        version: String::new(),
        winget_id: "Microsoft.DotNet.DesktopRuntime.8".into(),
    };
    if let Ok(k) = hkcu.open_subkey_with_flags(r"SOFTWARE\dotnet\Setup\InstalledVersions\x64\sharedfx\Microsoft.WindowsDesktop.App", KEY_READ) {
        net.installed = true;
        net.version = k.get_value::<String, _>("Version").unwrap_or_default();
    }
    out.push(net);

    // DirectX
    let mut dx = RuntimeStatus {
        name: "DirectX".into(),
        installed: false,
        version: String::new(),
        winget_id: String::new(),
    };
    if let Ok(k) = hkcu.open_subkey_with_flags(r"SOFTWARE\Microsoft\DirectX", KEY_READ) {
        dx.installed = true;
        dx.version = k.get_value::<String, _>("Version").unwrap_or_default();
    }
    out.push(dx);

    out
}

pub fn drivers_backup(folder: &str) -> (bool, String) {
    let f = folder.trim();
    if f.is_empty() {
        return (false, "No folder selected".into());
    }
    let _ = std::fs::create_dir_all(f);
    let out = crate::util::run_and_wait("pnputil", &["/export-driver", "*", f]);
    match out {
        Ok(t) => (true, t),
        Err(e) => (false, e),
    }
}

pub fn winget_install(id: &str) -> (bool, String) {
    if id.is_empty() {
        return (false, "No package id".into());
    }
    let out = crate::util::run_and_wait("winget", &["install", "--id", id, "-e", "--silent", "--accept-package-agreements", "--accept-source-agreements"]);
    match out {
        Ok(t) => (true, t),
        Err(e) => (false, e),
    }
}
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    pub letter: String,
    pub label: String,
    pub fs: String,
    pub total: u64,
    pub free: u64,
    pub drive_type: String,
    pub health: String,
    pub disk_health: String,
    pub model: String,
    pub bus: String,
    pub temp: i32,
    pub wear: i32,
    pub power_on: i64,
}

/// Enumerate every lettered volume (internal disks and USB flash drives)
/// with capacity, filesystem, health status and SMART data when available.
pub fn drives_info() -> Vec<DriveInfo> {
    let script = r#"
$out = @()
foreach ($v in (Get-Volume | Where-Object { $_.DriveLetter })) {
  $disk = $null; $rel = $null
  try {
    $part = Get-Partition -DriveLetter $v.DriveLetter -ErrorAction Stop
    $disk = Get-PhysicalDisk -DeviceNumber $part.DiskNumber -ErrorAction Stop
  } catch {}
  if ($disk) { try { $rel = $disk | Get-StorageReliabilityCounter -ErrorAction SilentlyContinue } catch {} }
  $type = [string]$v.DriveType
  if (-not $type) { $type = "Unknown" }
  $cloud = $false
  $hay = ([string]$v.FileSystemLabel + " " + $(if ($disk) { [string]$disk.FriendlyName } else { "" })).ToLower()
  foreach ($n in @("pcloud","onedrive","google drive","dropbox","icloud","mega","box")) {
    if ($hay.Contains($n)) { $cloud = $true }
  }
  if ($type -eq "Network") { $cloud = $true }
  if ($cloud) { $type = "Cloud" }
  $dhealth = ""; $temp = -1; $wear = -1; $power = -1
  if ($disk) { $dhealth = [string]$disk.HealthStatus }
  if ($rel) {
    if ($null -ne $rel.Temperature) { $temp = [int]$rel.Temperature }
    if ($null -ne $rel.Wear) { $wear = [int]$rel.Wear }
    if ($null -ne $rel.PowerOnHours) { $power = [int64]$rel.PowerOnHours }
  }
  $out += [pscustomobject]@{
    letter = "$($v.DriveLetter):"
    label = [string]$v.FileSystemLabel
    fs = [string]$v.FileSystem
    total = [uint64]$v.Size
    free = [uint64]$v.SizeRemaining
    driveType = $type
    health = [string]$v.HealthStatus
    diskHealth = $dhealth
    model = $(if ($disk) { [string]$disk.FriendlyName } else { "" })
    bus = $(if ($disk) { [string]$disk.BusType } else { "" })
    temp = $temp
    wear = $wear
    powerOn = $power
  }
}
$out | ConvertTo-Json -Compress -Depth 3
"#;
    let text = match crate::util::run_and_wait(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script],
    ) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let value: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let arr = match &value {
        serde_json::Value::Array(a) => a.clone(),
        serde_json::Value::Object(_) => vec![value],
        _ => return Vec::new(),
    };
    arr.into_iter()
        .map(|v| {
            let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
            let n = |k: &str| v.get(k).and_then(|x| x.as_u64()).unwrap_or(0);
            let i = |k: &str| v.get(k).and_then(|x| x.as_i64()).unwrap_or(-1);
            DriveInfo {
                letter: s("letter"),
                label: s("label"),
                fs: s("fs"),
                total: n("total"),
                free: n("free"),
                drive_type: s("driveType"),
                health: s("health"),
                disk_health: s("diskHealth"),
                model: s("model"),
                bus: s("bus"),
                temp: i("temp") as i32,
                wear: i("wear") as i32,
                power_on: i("powerOn"),
            }
        })
        .collect()
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DriveCheck {
    pub letter: String,
    pub ok: bool,
    pub code: i32,
    /// Machine-readable outcome: clean | errors | cleaned | unavailable | unknown
    pub status: String,
    pub raw: String,
}

/// Run a command and capture its raw exit code plus combined output.
fn run_status(program: &str, args: &[&str]) -> (i32, String) {
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
    match cmd.output() {
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            let text = String::from_utf8_lossy(&out.stdout).to_string()
                + &String::from_utf8_lossy(&out.stderr);
            (code, text)
        }
        Err(e) => (-1, format!("{program}: {e}")),
    }
}

pub fn spooler_pending_count() -> u32 {
    let dir = std::path::Path::new(r"C:\Windows\System32\spool\PRINTERS");
    std::fs::read_dir(dir).map(|d| d.count() as u32).unwrap_or(0)
}

pub fn spooler_fix() -> (bool, String) {
    let script = r#"
try {
  Stop-Service -Name Spooler -Force -ErrorAction Stop
  Remove-Item -Path "C:\Windows\System32\spool\PRINTERS\*" -Force -ErrorAction SilentlyContinue
  Start-Service -Name Spooler -ErrorAction Stop
  "OK"
} catch { "ERR: " + $_.Exception.Message }
"#;
    match crate::util::run_and_wait(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script],
    ) {
        Ok(t) => {
            let trimmed = t.trim().to_string();
            let ok = trimmed.eq_ignore_ascii_case("OK") || trimmed.to_uppercase().contains("OK");
            (ok, trimmed)
        }
        Err(e) => (false, e),
    }
}
/// 0 = no errors, 1 = errors found, 2 = cleanup done, 3 = could not check.
pub fn disk_check(letter: &str) -> DriveCheck {
    let l = letter.trim().trim_end_matches(':').to_string();
    let mut result = DriveCheck {
        letter: format!("{l}:"),
        ok: false,
        code: -1,
        status: "unknown".into(),
        raw: String::new(),
    };
    if l.is_empty() || l.len() > 2 {
        result.status = "unavailable".into();
        result.raw = "Invalid drive letter".into();
        return result;
    }
    // Switch the console to UTF-8 so localized output decodes correctly.
    let line = format!("chcp 65001>nul & chkdsk {l}: /scan");
    let (code, raw) = run_status("cmd", &["/C", line.as_str()]);
    result.code = code;
    result.raw = raw;
    let low = result.raw.to_lowercase();
    let looks_like_access = low.contains("access is denied")
        || low.contains("requires elevation")
        || low.contains("must be run")
        || low.contains("cannot lock");
    result.status = match code {
        0 => "clean",
        1 => "errors",
        2 => "cleaned",
        3 => "unavailable",
        _ => {
            if looks_like_access {
                "unavailable"
            } else {
                "unknown"
            }
        }
    }
    .into();
    result.ok = result.status == "clean" || result.status == "cleaned";
    result
}
