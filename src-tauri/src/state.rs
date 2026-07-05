use keepass::{Database, DatabaseKey};
// parking_lot::Mutex cannot be poisoned: a panic while holding the lock
// simply releases it, so commands don't need per-call poison handling.
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

pub type VaultId = Uuid;

pub struct OpenVault {
    pub db: Database,
    pub path: PathBuf,
    pub db_key: DatabaseKey,
    /// SHA-256 of the raw master-password bytes, stored for constant-time
    /// verification on password change.
    pub password_hash: [u8; 32],
    /// SHA-256 of the vault file's bytes as we last read or wrote them.
    /// Checked before every save to detect external modification (sync
    /// clients, other KeePass apps) instead of silently clobbering it.
    pub disk_hash: [u8; 32],
}

// Zeroize sensitive key material when the vault is dropped (lock, close, …)
impl Drop for OpenVault {
    fn drop(&mut self) {
        // DatabaseKey implements ZeroizeOnDrop — its password field is zeroized
        // automatically when the struct is dropped.
        // Replace the Database with an empty one so decrypted entry data is
        // released from the heap. Note: the allocator may not immediately
        // overwrite the freed pages — a proper scrub would require the keepass
        // crate to implement Zeroize internally.
        self.db = keepass::Database::new();
        // Scrub the cached password hash.
        self.password_hash.iter_mut().for_each(|b| *b = 0);
        // Path is not sensitive; no need to scrub.
    }
}

pub struct AppState {
    pub vaults: Mutex<HashMap<VaultId, OpenVault>>,
    /// Serializes all vault-file writers (entry update/delete, password
    /// change, KDF upgrade). Writers snapshot the database under a brief
    /// `vaults` lock and run KDF + serialize + fsync on a blocking thread;
    /// holding this lock across snapshot *and* write keeps saves from
    /// hitting the disk out of order.
    ///
    /// Lock ordering: acquire `save_lock` BEFORE `vaults`, and never await
    /// while holding `vaults` (it is a sync mutex).
    pub save_lock: tokio::sync::Mutex<()>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            vaults: Mutex::new(HashMap::new()),
            save_lock: tokio::sync::Mutex::new(()),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the on-disk hash after a successful save. No-op if the vault
    /// was locked or swapped for a different file while the save ran.
    pub fn commit_disk_hash(&self, path: &std::path::Path, hash: [u8; 32]) {
        let mut vaults = self.vaults.lock();
        if let Some((_id, vault)) = vaults.iter_mut().next() {
            if vault.path == path {
                vault.disk_hash = hash;
            }
        }
    }
}
