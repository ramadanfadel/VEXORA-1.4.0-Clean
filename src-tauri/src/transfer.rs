//! Verified local file copy. The destination must not already exist unless
//! overwrite is explicitly requested; symlinks/junctions are never followed.

use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    pub bytes: u64,
    pub verified: bool,
    pub src_hash: String,
    pub dst_hash: String,
}

pub fn copy_verified(src: &str, dst: &str, overwrite: bool) -> Result<TransferResult, String> {
    let source = crate::safety::validate_path_for_read(src)?;
    if source.is_dir() {
        return Err("source is a directory".into());
    }
    let destination = Path::new(dst);
    if destination.as_os_str().is_empty() || destination.is_dir() {
        return Err("invalid destination".into());
    }
    if crate::safety::is_reparse_point(destination) {
        return Err("destination is a reparse point".into());
    }
    if destination.exists() && !overwrite {
        return Err("destination already exists".into());
    }
    let parent = destination.parent().ok_or("invalid destination parent")?;
    if !crate::safety::clean_path_allowed(parent) {
        return Err("destination parent is protected".into());
    }
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;

    let src_hash = crate::hyperscan::hash_file(&source).ok_or("cannot read source")?;
    let bytes = std::fs::copy(&source, destination).map_err(|e| e.to_string())?;
    let dst_hash = crate::hyperscan::hash_file(destination).ok_or("cannot read destination")?;
    if src_hash != dst_hash {
        let _ = std::fs::remove_file(destination);
        return Err("copy verification failed — destination removed".into());
    }
    Ok(TransferResult {
        bytes,
        verified: true,
        src_hash,
        dst_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_and_verifies() {
        let dir = std::env::temp_dir().join(format!("vexora_transfer_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let a = dir.join("a.bin");
        let b = dir.join("b.bin");
        std::fs::write(&a, b"transfer payload 1234").unwrap();
        let r = copy_verified(
            &a.to_string_lossy(),
            &b.to_string_lossy(),
            false,
        )
        .unwrap();
        assert!(r.verified);
        assert_eq!(r.bytes, 21);
        assert_eq!(r.src_hash, r.dst_hash);
        assert!(copy_verified(&a.to_string_lossy(), &b.to_string_lossy(), false).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
