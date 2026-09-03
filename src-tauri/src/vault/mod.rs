pub mod atomic_write;
pub mod disk;
mod file_lock;
pub use atomic_write::{atomic_write, backed_up_atomic_write};
pub use disk::{ensure_unmodified, sha256_bytes};
pub use file_lock::lock_vault_for_save;

use std::path::Path;

use crate::error::{HitsuError, HitsuResult};

/// Persist a vault with external-modification protection.
///
/// Under an exclusive advisory lock on the vault file (when the platform
/// supports it, see `file_lock`):
///
/// 1. Verify the on-disk hash still matches `expected_disk_hash` — aborts
///    with `ExternalModification` before the expensive KDF work when a sync
///    client or another KeePass app replaced the file.
/// 2. Run `serialize` (Argon2 + encode).
/// 3. Re-verify the hash immediately before the rename so the clobber
///    window shrinks from "serialize time" to "one file read".
/// 4. Write atomically.
///
/// Runs on a blocking thread; blocking briefly on the advisory lock is
/// acceptable there. Returns the payload from `serialize` plus the SHA-256
/// of the written bytes.
pub(crate) fn save_protected<T>(
    path: &Path,
    expected_disk_hash: &[u8; 32],
    serialize: impl FnOnce() -> HitsuResult<(T, Vec<u8>)>,
) -> HitsuResult<(T, [u8; 32])> {
    let _lock = lock_vault_for_save(path);
    ensure_unmodified(path, expected_disk_hash)?;
    let (payload, bytes) = serialize()?;
    ensure_unmodified(path, expected_disk_hash)?;
    atomic_write(path, &bytes)?;
    Ok((payload, sha256_bytes(&bytes)))
}

/// `save_protected` with a timestamped backup and post-write verification
/// (see `backed_up_atomic_write`); the original is restored on failure.
pub(crate) fn save_protected_with_backup<T>(
    path: &Path,
    expected_disk_hash: &[u8; 32],
    serialize: impl FnOnce() -> HitsuResult<(T, Vec<u8>)>,
    verify: impl FnOnce(&Path) -> Result<(), String>,
) -> HitsuResult<(T, [u8; 32])> {
    let _lock = lock_vault_for_save(path);
    ensure_unmodified(path, expected_disk_hash)?;
    let (payload, bytes) = serialize()?;
    ensure_unmodified(path, expected_disk_hash)?;
    backed_up_atomic_write(path, &bytes, verify).map_err(HitsuError::Custom)?;
    Ok((payload, sha256_bytes(&bytes)))
}
