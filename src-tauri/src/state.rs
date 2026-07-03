use keepass::{Database, DatabaseKey};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

pub type VaultId = Uuid;

pub struct OpenVault {
    pub db: Database,
    pub path: PathBuf,
    pub db_key: DatabaseKey,
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
        // Path is not sensitive; no need to scrub.
    }
}

pub struct AppState {
    pub vaults: Mutex<HashMap<VaultId, OpenVault>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            vaults: Mutex::new(HashMap::new()),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
