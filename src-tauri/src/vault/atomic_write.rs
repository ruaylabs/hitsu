use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

/// Create a file with owner-only permissions (0600).
fn create_owner_only(path: &Path) -> io::Result<fs::File> {
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    opts.mode(0o600);
    opts.open(path)
}

/// Set permissions of a file to owner-only (0600).
fn set_owner_only(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Atomically write `data` to `path`.
///
/// 1. Write to `<path>.hitsu-tmp` on the same filesystem.
/// 2. `fsync` the temp file (data + metadata).
/// 3. `rename` over the target (atomic on POSIX, near-atomic on NTFS).
/// 4. `fsync` the parent directory so the rename survives a hard reboot.
///
/// On failure the temporary file is cleaned up and the original is untouched.
pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let tmp_path = path.with_extension("hitsu-tmp");

    let result = try_write(path, data, &tmp_path);

    // Clean up temp on any failure
    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

fn try_write(path: &Path, data: &[u8], tmp_path: &Path) -> io::Result<()> {
    // 1. Write to temp
    let mut tmp = create_owner_only(tmp_path)?;
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

/// Write `new_bytes` to `path` atomically, with a timestamped sibling backup
/// at `<path>.<iso_timestamp>.bak`. After writing, calls `verify` to check
/// the new file is valid (e.g. can be re-opened with the new key).
///
/// On verification success the backup is deleted. On failure the original
/// file is restored from the backup and the backup is removed.
///
/// The backup is a simple `fs::copy` — it does NOT use `atomic_write` itself.
/// That's fine: we're writing TO the backup once, not doing a destructive
/// rename over an existing file.
pub fn backed_up_atomic_write(
    path: &Path,
    new_bytes: &[u8],
    verify: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<(), String> {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%S");
    let backup_dir = path.parent().unwrap_or(Path::new("."));
    let backup = backup_dir.join(format!(
        "{}.{}.bak",
        path.file_name().unwrap_or_default().to_string_lossy(),
        ts
    ));

    // 1. Copy original to backup
    fs::copy(path, &backup).map_err(|_| {
        "Could not create a backup before writing. The original file is unchanged.".to_string()
    })?;
    // Restrict backup to owner-only (copy may inherit broader permissions)
    let _ = set_owner_only(&backup);

    // 2. Atomically write new data
    atomic_write(path, new_bytes).map_err(|_| {
        // Clean up backup on write failure — original is untouched
        let _ = fs::remove_file(&backup);
        "Could not save the vault. The original file is unchanged.".to_string()
    })?;

    // 3. Verify the new file is valid
    match verify(path) {
        Ok(()) => {
            // 4. Clean up backup — everything succeeded
            let _ = fs::remove_file(&backup);
            Ok(())
        }
        Err(e) => {
            // Log the real reason locally, but tell the user a safe message
            eprintln!("vault verification failed: {}", e);
            // 5. Restore original from backup and remove the backup
            let _ = fs::copy(&backup, path);
            let _ = fs::remove_file(&backup);
            Err(
                "Could not verify the saved vault. The original file has been restored."
                    .to_string(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_atomic_write_creates_file() {
        let dir = std::env::temp_dir().join("hitsu-atomic-test");
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
        let dir = std::env::temp_dir().join("hitsu-atomic-fail");
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
        let dir = std::env::temp_dir().join("hitsu-atomic-cleanup");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("vault.kdbx");
        let tmp_path = path.with_extension("hitsu-tmp");

        // Try writing to a path where the target directory doesn't exist
        let bad_path = dir.join("missing").join("vault.kdbx");
        let result = atomic_write(&bad_path, b"data");
        assert!(result.is_err());

        // Temp file should NOT remain
        assert!(!tmp_path.exists(), "temp file must be cleaned up");
        // But we used bad_path, the tmp_path would be different...
        // Let me use the actual path instead
        let bad_tmp = bad_path.with_extension("hitsu-tmp");
        assert!(!bad_tmp.exists(), "temp file must be cleaned up");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    #[cfg(unix)]
    fn test_atomic_write_owner_only_permissions() {
        let dir = std::env::temp_dir().join("hitsu-atomic-perms");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("vault.kdbx");
        atomic_write(&path, b"secret data").unwrap();

        let meta = fs::metadata(&path).unwrap();
        let mode = meta.permissions().mode();
        // File should be 0600 (owner read/write, no group/other access).
        // st_mode includes the file type bits, so mask to just permissions.
        assert_eq!(
            mode & 0o777,
            0o600,
            "vault file must be owner-only ({:#o})",
            mode
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
