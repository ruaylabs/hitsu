//! On-disk change detection for open vaults.
//!
//! The vault file can be replaced underneath us by a sync client (iCloud,
//! Dropbox) or another KeePass app. Every save therefore checks that the
//! file still hashes to what we last read or wrote; a mismatch aborts the
//! save with `KagiError::ExternalModification` instead of silently
//! clobbering the other writer's changes (last-writer-wins).

use sha2::{Digest, Sha256};
use std::path::Path;

use crate::error::{KagiError, KagiResult};

/// SHA-256 of a byte buffer (vault bytes about to be written).
pub fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// SHA-256 of a file's current contents.
pub fn sha256_file(path: &Path) -> std::io::Result<[u8; 32]> {
    Ok(sha256_bytes(&std::fs::read(path)?))
}

/// Verify the on-disk file still matches `expected` (the hash recorded when
/// we last read or wrote it). Any difference — including the file having
/// been deleted or made unreadable — means another program touched the
/// vault, so the caller must not overwrite it.
pub fn ensure_unmodified(path: &Path, expected: &[u8; 32]) -> KagiResult<()> {
    let actual = sha256_file(path).map_err(|_| KagiError::ExternalModification)?;
    if actual != *expected {
        return Err(KagiError::ExternalModification);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(data: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("kagi-disk-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vault.kdbx");
        std::fs::write(&path, data).unwrap();
        path
    }

    #[test]
    fn test_unmodified_file_passes() {
        let path = temp_file(b"vault bytes");
        let hash = sha256_file(&path).unwrap();
        assert!(ensure_unmodified(&path, &hash).is_ok());
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_modified_file_is_detected() {
        let path = temp_file(b"vault bytes");
        let hash = sha256_file(&path).unwrap();
        std::fs::write(&path, b"replaced by a sync client").unwrap();
        assert!(matches!(
            ensure_unmodified(&path, &hash),
            Err(KagiError::ExternalModification)
        ));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_deleted_file_is_detected() {
        let path = temp_file(b"vault bytes");
        let hash = sha256_file(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert!(matches!(
            ensure_unmodified(&path, &hash),
            Err(KagiError::ExternalModification)
        ));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}
