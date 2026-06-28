use keepass::Database;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

pub type VaultId = Uuid;

pub struct OpenVault {
    pub db: Database,
    pub _path: PathBuf,
    pub _name: String,
    pub _master_key: Vec<u8>,
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
