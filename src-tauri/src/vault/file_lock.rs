//! Best-effort advisory locking for vault saves.
//!
//! The save path is: verify on-disk hash → serialize (Argon2, slow) →
//! rename. An external writer landing between the check and the rename is
//! silently clobbered, so saves hold an exclusive advisory lock on the vault
//! file across the whole span and re-verify the hash immediately before the
//! rename. This closes the race against flock-aware writers entirely
//! (including a second Hitsu instance) and shrinks it to a single file read
//! against everything else (sync clients rarely take locks).
//!
//! Locks are best-effort: if the platform or filesystem does not support
//! them (some network mounts), saving proceeds with the re-check alone.
//! The lock lives on an open file descriptor, so a crashed process can
//! never leave a stale lock behind.

use std::fs::{File, OpenOptions};
use std::path::Path;

/// Exclusive advisory lock on the vault file, released on drop.
pub struct VaultFileLock {
    _file: File,
}

/// Lock the vault file for the span of a save. Returns `None` when the file
/// cannot be opened (missing on first save, unreadable) — the caller's hash
/// check reports those conditions with the established error.
pub fn lock_vault_for_save(path: &Path) -> Option<VaultFileLock> {
    // Read-only access is enough: flock does not require write permission.
    let file = OpenOptions::new().read(true).open(path).ok()?;
    lock_exclusive(&file);
    Some(VaultFileLock { _file: file })
}

#[cfg(unix)]
fn lock_exclusive(file: &File) {
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();
    // SAFETY: flock takes only a valid fd and lock constants.
    if unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) } == 0 {
        return;
    }
    let error = std::io::Error::last_os_error();
    let busy = error.raw_os_error() == Some(libc::EWOULDBLOCK)
        || error.raw_os_error() == Some(libc::EAGAIN);
    if busy {
        // Another writer (e.g. a second Hitsu instance) holds the lock for
        // the duration of one save — waiting briefly beats failing.
        tracing::debug!("vault file is locked by another program; waiting");
        // SAFETY: as above.
        if unsafe { libc::flock(fd, libc::LOCK_EX) } != 0 {
            tracing::debug!(error = %std::io::Error::last_os_error(), "vault lock wait failed");
        }
        return;
    }
    tracing::debug!(error = %error, "vault file locking unavailable");
}

#[cfg(not(unix))]
fn lock_exclusive(_file: &File) {
    // Windows byte-range locks are mandatory and would block Hitsu's own
    // reads through separate handles, so saves there rely on the re-check.
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("hitsu-flock-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn missing_file_yields_no_lock() {
        let path = temp_path("missing.kdbx");
        assert!(lock_vault_for_save(&path).is_none());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn lock_is_released_on_drop() {
        let path = temp_path("released.kdbx");
        std::fs::write(&path, b"vault").unwrap();

        drop(lock_vault_for_save(&path).unwrap());

        // A second exclusive lock must be acquirable right away; flock
        // conflicts across open file descriptions even in one process, so
        // this hangs if the first lock was not released.
        drop(lock_vault_for_save(&path).unwrap());

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn second_writer_waits_for_the_first() {
        let path = temp_path("contended.kdbx");
        std::fs::write(&path, b"vault").unwrap();

        let first = lock_vault_for_save(&path).unwrap();
        let worker_path = path.clone();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let guard = lock_vault_for_save(&worker_path).unwrap();
            done_tx.send(()).unwrap();
            drop(guard);
        });

        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(
            done_rx
                .recv_timeout(std::time::Duration::from_millis(50))
                .is_err(),
            "second lock must wait while the first is held"
        );

        drop(first);
        done_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        worker.join().unwrap();

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
