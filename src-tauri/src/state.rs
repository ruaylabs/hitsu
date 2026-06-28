use keepass::Database;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;
use zeroize::Zeroizing;

pub type VaultId = Uuid;

pub struct OpenVault {
    pub db: Database,
    pub path: PathBuf,
    pub master_key: Zeroizing<Vec<u8>>,
}

// Zeroize sensitive key material when the vault is dropped (lock, close, …)
impl Drop for OpenVault {
    fn drop(&mut self) {
        // Zeroizing<Vec<u8>> zeros itself on drop via its own Drop impl.
        // The Database (which holds decrypted entries in heap memory) is dropped
        // automatically; heap contents aren't explicitly scrubbed here.
    }
}

pub struct AppState {
    pub vaults: Mutex<HashMap<VaultId, OpenVault>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vaults: Mutex::new(HashMap::new()),
        }
    }
}
