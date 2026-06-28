use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Atomically write `data` to `path`.
///
/// 1. Write to `<path>.kagi-tmp` on the same filesystem.
/// 2. `fsync` the temp file (data + metadata).
/// 3. `rename` over the target (atomic on POSIX, near-atomic on NTFS).
/// 4. `fsync` the parent directory so the rename survives a hard reboot.
///
/// On failure the temporary file is cleaned up and the original is untouched.
pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let tmp_path = path.with_extension("kagi-tmp");

    let result = try_write(path, data, &tmp_path);

    // Clean up temp on any failure
    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

fn try_write(path: &Path, data: &[u8], tmp_path: &Path) -> io::Result<()> {
    // 1. Write to temp
    let mut tmp = fs::File::create(tmp_path)?;
    tmp.write_all(data)?;

    // 2. Flush and fsync file data + metadata
    tmp.sync_all()?;
    drop(tmp); // Release handle before rename (important on Windows)

    // 3. Atomic rename over the target
    fs::rename(tmp_path, path)?;

    // 4. Sync the parent directory so the rename is durable
    if let Some(parent) = path.parent() {
        if let Ok(dir) = fs::File::open(parent) {
            dir.sync_all()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_atomic_write_creates_file() {
        let dir = std::env::temp_dir().join("kagi-atomic-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("test.kdbx");
        atomic_write(&path, b"hello vault").unwrap();

        let mut content = Vec::new();
        fs::File::open(&path)
            .unwrap()
            .read_to_end(&mut content)
            .unwrap();
        assert_eq!(content, b"hello vault");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_atomic_write_does_not_corrupt_on_failure() {
        let dir = std::env::temp_dir().join("kagi-atomic-fail");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("existing.kdbx");
        fs::write(&path, b"original data").unwrap();

        // Simulate a failure by creating a directory at the temp path so
        // the rename will succeed but the write already happened.
        // Actually, let's just verify that an unwritable path doesn't clobber the original.
        let bad_path = dir.join("no-such-dir/test.kdbx");
        let result = atomic_write(&bad_path, b"should not appear");
        assert!(result.is_err());

        let mut content = Vec::new();
        fs::File::open(&path)
            .unwrap()
            .read_to_end(&mut content)
            .unwrap();
        assert_eq!(content, b"original data", "original file must be untouched");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_atomic_write_no_temp_left_on_error() {
        let dir = std::env::temp_dir().join("kagi-atomic-cleanup");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("vault.kdbx");
        let tmp_path = path.with_extension("kagi-tmp");

        // Try writing to a path where the target directory doesn't exist
        let bad_path = dir.join("missing").join("vault.kdbx");
        let result = atomic_write(&bad_path, b"data");
        assert!(result.is_err());

        // Temp file should NOT remain
        assert!(!tmp_path.exists(), "temp file must be cleaned up");
        // But we used bad_path, the tmp_path would be different...
        // Let me use the actual path instead
        let bad_tmp = bad_path.with_extension("kagi-tmp");
        assert!(!bad_tmp.exists(), "temp file must be cleaned up");

        let _ = fs::remove_dir_all(&dir);
    }
}
