use std::fs::File;
use std::path::{Path, PathBuf};
use tauri::State;
use zeroize::Zeroizing;

use crate::error::{KagiError, KagiResult};
use crate::models::VaultMeta;
use crate::state::{AppState, OpenVault, VaultId};

fn detect_sync_provider(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if path_str.contains("Mobile Documents") || path_str.contains("CloudDocs") {
        "icloud".to_string()
    } else if path_str.contains("Dropbox") {
        "dropbox".to_string()
    } else {
        "local".to_string()
    }
}

fn count_entries(db: &keepass::Database) -> usize {
    let mut count = 0;
    for node in &db.root {
        if matches!(node, keepass::db::NodeRef::Entry(_)) {
            count += 1;
        }
    }
    count
}

#[tauri::command]
pub async fn vault_open(
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> KagiResult<VaultMeta> {
    let path = PathBuf::from(&path);
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unnamed")
        .to_string();

    let mut file = File::open(&path)?;
    let key = keepass::DatabaseKey::new().with_password(&password);
    let db =
        keepass::Database::open(&mut file, key).map_err(|e| KagiError::Vault(e.to_string()))?;

    let entry_count = count_entries(&db);
    let id = uuid::Uuid::new_v4();

    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    vaults.insert(
        id,
        OpenVault {
            db,
            path: path.clone(),
            master_key: Zeroizing::new(password.into_bytes()),
        },
    );

    Ok(VaultMeta {
        path: path.to_string_lossy().to_string(),
        name,
        item_count: entry_count,
        sync_provider: detect_sync_provider(&path),
    })
}

#[tauri::command]
pub async fn vault_create(
    state: State<'_, AppState>,
    path: String,
    password: String,
    name: String,
) -> KagiResult<VaultMeta> {
    let path = PathBuf::from(&path);
    let vault_name = if name.is_empty() {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unnamed")
            .to_string()
    } else {
        name
    };

    let mut db = keepass::Database::new(Default::default());
    db.meta.database_name = Some(vault_name.clone());

    let key = keepass::DatabaseKey::new().with_password(&password);
    let mut file = File::create(&path)?;
    db.save(&mut file, key)?;

    // Re-open to verify and store
    let mut file = File::open(&path)?;
    let key = keepass::DatabaseKey::new().with_password(&password);
    let db =
        keepass::Database::open(&mut file, key).map_err(|e| KagiError::Vault(e.to_string()))?;

    let entry_count = 0;
    let id = uuid::Uuid::new_v4();

    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    vaults.insert(
        id,
        OpenVault {
            db,
            path: path.clone(),
            master_key: Zeroizing::new(password.into_bytes()),
        },
    );

    Ok(VaultMeta {
        path: path.to_string_lossy().to_string(),
        name: vault_name,
        item_count: entry_count,
        sync_provider: detect_sync_provider(&path),
    })
}

#[tauri::command]
pub async fn vault_change_password(
    state: State<'_, AppState>,
    old_password: String,
    new_password: String,
) -> KagiResult<()> {
    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    // Find the single open vault
    let (_id, vault): (&VaultId, &mut OpenVault) =
        vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

    // Verify old password matches stored key
    let stored_key = String::from_utf8_lossy(&vault.master_key);
    if stored_key != old_password {
        return Err(KagiError::Custom("Wrong password".to_string()));
    }

    let new_key = keepass::DatabaseKey::new().with_password(&new_password);
    let mut file = File::create(&vault.path)?;
    vault.db.save(&mut file, new_key)?;

    vault.master_key = Zeroizing::new(new_password.into_bytes());
    Ok(())
}

#[tauri::command]
pub async fn vault_lock(state: State<'_, AppState>) -> KagiResult<()> {
    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    // Clear all open vaults — this drops each OpenVault, which zeroizes
    // the master-key buffer via Zeroizing's Drop impl.
    vaults.clear();

    Ok(())
}
