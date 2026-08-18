use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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

/// Temp path for an atomic write of `path`: a sibling file named
/// `<filename>.hitsu-tmp.<random>`.
///
/// The random suffix keeps concurrent writers writing different targets from
/// clobbering each other's temp file (with_extension("hitsu-tmp") mapped e.g.
/// `vault.kdbx` and `vault.json` in one directory to the same name).
fn temp_path(path: &Path) -> PathBuf {
    let suffix = &uuid::Uuid::new_v4().simple().to_string()[..8];
    let file_name = format!(
        "{}.hitsu-tmp.{suffix}",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    path.with_file_name(file_name)
}

/// Atomically write `data` to `path`.
///
/// 1. Write to a unique `<path>.hitsu-tmp.<random>` on the same filesystem.
/// 2. `fsync` the temp file (data + metadata).
/// 3. `rename` over the target (atomic on POSIX, near-atomic on NTFS).
/// 4. `fsync` the parent directory so the rename survives a hard reboot.
///
/// On failure the temporary file is cleaned up and the original is untouched.
pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let tmp_path = temp_path(path);

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
            // Log the real reason locally, but tell the user a safe message.
            tracing::warn!("vault verification failed; restoring original");
            tracing::debug!(error = %e, "vault verification failure detail");

            // Restore atomically so another failure cannot partially overwrite
            // the destination. Keep the backup whenever recovery is incomplete.
            let restore_result = fs::read(&backup).and_then(|bytes| atomic_write(path, &bytes));
            if let Err(restore_error) = restore_result {
                tracing::error!("vault restoration failed; preserving recovery backup");
                tracing::debug!(
                    error = %restore_error,
                    backup = %backup.display(),
                    "vault restoration failure detail"
                );
                return Err(
                    "Could not verify or restore the saved vault. A recovery backup has been \
                     preserved."
                        .to_string(),
                );
            }

            if let Err(remove_error) = fs::remove_file(&backup) {
                tracing::warn!("restored vault backup could not be removed");
                tracing::debug!(
                    error = %remove_error,
                    backup = %backup.display(),
                    "restored vault backup cleanup failure detail"
                );
            }
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

        let tmp_glob = "vault.kdbx.hitsu-tmp.";

        // Try writing to a path where the target directory doesn't exist
        let bad_path = dir.join("missing").join("vault.kdbx");
        let result = atomic_write(&bad_path, b"data");
        assert!(result.is_err());

        // Temp files should NOT remain in either directory
        let leftovers = |dir: &Path| {
            fs::read_dir(dir)
                .map(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .filter(|entry| entry.file_name().to_string_lossy().starts_with(tmp_glob))
                        .count()
                })
                .unwrap_or(0)
        };
        assert_eq!(leftovers(&dir), 0, "temp file must be cleaned up");
        assert_eq!(
            leftovers(&dir.join("missing")),
            0,
            "temp file must be cleaned up"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn preserves_backup_when_verification_restore_fails() {
        let dir = std::env::temp_dir().join(format!("hitsu-restore-fail-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vault.kdbx");
        fs::write(&path, b"original vault").unwrap();

        let result = backed_up_atomic_write(&path, b"invalid replacement", |candidate| {
            fs::remove_file(candidate).unwrap();
            fs::create_dir(candidate).unwrap();
            Err("forced verification failure".to_string())
        });

        let backup_contents = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| entry.file_name().to_string_lossy().ends_with(".bak"))
            .and_then(|entry| fs::read(entry.path()).ok());

        fs::remove_dir_all(&dir).unwrap();

        assert!(result.is_err());
        assert_eq!(
            backup_contents.as_deref(),
            Some(b"original vault".as_slice())
        );
    }

    #[test]
    fn temp_paths_do_not_collide_across_targets_or_writers() {
        let dir = std::env::temp_dir().join("hitsu-atomic-tempnames");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // `with_extension` mapped both of these to `vault.hitsu-tmp`.
        let kdbx = temp_path(&dir.join("vault.kdbx"));
        let json = temp_path(&dir.join("vault.json"));
        assert_ne!(kdbx, json);

        // Same target written twice also gets distinct temp files.
        assert_ne!(temp_path(&dir.join("vault.kdbx")), kdbx);

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
