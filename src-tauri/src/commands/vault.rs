use keepass::config::KdfConfig;
use std::fs::File;
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;
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

/// Ensure the database uses KDBX4.1 format so saving works.
/// keepass 0.13's dump_kdbx4 requires exactly `KDB4(1)`.
fn ensure_kdbx4(db: &mut keepass::Database) {
    db.config.version = keepass::config::DatabaseVersion::KDB4(1);
}

fn count_entries(db: &keepass::Database) -> usize {
    db.iter_all_entries().count()
}

/// Check whether the KDF is below the recommended 64 MiB threshold.
/// Returns `true` if the vault should be upgraded.
fn needs_kdf_upgrade(kdf: &KdfConfig) -> bool {
    match kdf {
        KdfConfig::Argon2 { memory, .. } | KdfConfig::Argon2id { memory, .. } => {
            *memory < 64 * 1024 * 1024
        }
        _ => true, // AES or unknown → needs upgrade
    }
}

fn validate_kdf(kdf: &KdfConfig) -> KagiResult<()> {
    match kdf {
        KdfConfig::Argon2 {
            memory,
            iterations,
            parallelism,
            ..
        }
        | KdfConfig::Argon2id {
            memory,
            iterations,
            parallelism,
            ..
        } => {
            if *memory < 1024 * 1024 {
                return Err(KagiError::Custom("KDF memory menor a 1 MiB".to_string()));
            }
            if *iterations < 2 {
                return Err(KagiError::Custom("KDF iterations menor a 2".to_string()));
            }
            if *parallelism < 1 {
                return Err(KagiError::Custom("KDF parallelism inválido".to_string()));
            }
            Ok(())
        }
        KdfConfig::Aes { .. } => Err(KagiError::Custom(
            "AES-KDF no aceptado, usar Argon2id".to_string(),
        )),
        _ => Err(KagiError::Custom("KDF no soportado".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_kdf_rejects_aes() {
        let kdf = KdfConfig::Aes { rounds: 6000 };
        let result = validate_kdf(&kdf);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("AES-KDF"));
    }

    #[test]
    fn test_validate_kdf_rejects_argon2_low_memory() {
        let kdf = KdfConfig::Argon2 {
            memory: 512 * 1024,
            iterations: 50,
            parallelism: 4,
            version: argon2::Version::Version13,
        };
        let result = validate_kdf(&kdf);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1 MiB"));
    }

    #[test]
    fn test_validate_kdf_rejects_argon2id_low_memory() {
        let kdf = KdfConfig::Argon2id {
            memory: 512 * 1024,
            iterations: 50,
            parallelism: 4,
            version: argon2::Version::Version13,
        };
        let result = validate_kdf(&kdf);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1 MiB"));
    }

    #[test]
    fn test_validate_kdf_rejects_low_iterations() {
        let kdf = KdfConfig::Argon2id {
            memory: 64 * 1024 * 1024,
            iterations: 1,
            parallelism: 4,
            version: argon2::Version::Version13,
        };
        let result = validate_kdf(&kdf);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("iterations"));
    }

    #[test]
    fn test_validate_kdf_rejects_low_parallelism() {
        let kdf = KdfConfig::Argon2id {
            memory: 64 * 1024 * 1024,
            iterations: 2,
            parallelism: 0,
            version: argon2::Version::Version13,
        };
        let result = validate_kdf(&kdf);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parallelism"));
    }

    #[test]
    fn test_validate_kdf_accepts_argon2id_valid() {
        let kdf = KdfConfig::Argon2id {
            memory: 64 * 1024 * 1024,
            iterations: 2,
            parallelism: 1,
            version: argon2::Version::Version13,
        };
        assert!(validate_kdf(&kdf).is_ok());
    }

    #[test]
    fn test_validate_kdf_accepts_argon2_valid() {
        let kdf = KdfConfig::Argon2 {
            memory: 64 * 1024 * 1024,
            iterations: 2,
            parallelism: 1,
            version: argon2::Version::Version13,
        };
        assert!(validate_kdf(&kdf).is_ok());
    }

    #[test]
    fn test_needs_kdf_upgrade_below_64_mib() {
        let kdf = KdfConfig::Argon2id {
            memory: 1024 * 1024,
            iterations: 2,
            parallelism: 1,
            version: argon2::Version::Version13,
        };
        assert!(needs_kdf_upgrade(&kdf));
    }

    #[test]
    fn test_needs_kdf_upgrade_at_64_mib() {
        let kdf = KdfConfig::Argon2id {
            memory: 64 * 1024 * 1024,
            iterations: 2,
            parallelism: 1,
            version: argon2::Version::Version13,
        };
        assert!(!needs_kdf_upgrade(&kdf));
    }

    #[test]
    fn test_needs_kdf_upgrade_aes() {
        let kdf = KdfConfig::Aes { rounds: 6000 };
        assert!(needs_kdf_upgrade(&kdf));
    }
}

#[tauri::command]
pub async fn vault_open(
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> KagiResult<VaultMeta> {
    // Wrap immediately so the string buffer is zeroized on any early return
    let mut password = Zeroizing::new(password);

    let path = PathBuf::from(&path);
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unnamed")
        .to_string();

    let mut file = File::open(&path)?;
    let key = keepass::DatabaseKey::new().with_password(&password);
    let mut db =
        keepass::Database::open(&mut file, key).map_err(|e| KagiError::Vault(e.to_string()))?;

    validate_kdf(&db.config.kdf_config)?;

    // keepass 0.13 only supports saving KDBX4 — upgrade if needed
    ensure_kdbx4(&mut db);

    let entry_count = count_entries(&db);
    let id = uuid::Uuid::new_v4();

    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    // Swap the String with an empty one via DerefMut, then convert to bytes.
    // The Zeroizing<String> (now holding "") drops harmlessly later.
    let kdf_needs_upgrade = needs_kdf_upgrade(&db.config.kdf_config);

    let pw_str = std::mem::take(&mut *password);
    vaults.insert(
        id,
        OpenVault {
            db,
            path: path.clone(),
            master_key: Zeroizing::new(pw_str.into_bytes()),
        },
    );

    Ok(VaultMeta {
        path: path.to_string_lossy().to_string(),
        name,
        item_count: entry_count,
        sync_provider: detect_sync_provider(&path),
        kdf_needs_upgrade,
    })
}

#[tauri::command]
pub async fn vault_upgrade_kdf(state: State<'_, AppState>) -> KagiResult<()> {
    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    let (_id, vault): (&VaultId, &mut OpenVault) =
        vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

    // Upgrade KDF to Argon2id with 64 MiB
    vault.db.config.kdf_config = KdfConfig::Argon2id {
        memory: 64 * 1024 * 1024,
        iterations: 2,
        parallelism: 4,
        version: argon2::Version::Version13,
    };

    // Re-save with current master key
    let pw = String::from_utf8_lossy(&vault.master_key);
    let key = keepass::DatabaseKey::new().with_password(&pw);
    let mut buf = std::io::Cursor::new(Vec::new());
    vault
        .db
        .save(&mut buf, key)
        .map_err(|e| KagiError::Vault(e.to_string()))?;
    let bytes = buf.into_inner();
    crate::vault::atomic_write(&vault.path, &bytes)?;

    Ok(())
}

#[tauri::command]
pub async fn vault_create(
    state: State<'_, AppState>,
    path: String,
    password: String,
    name: String,
) -> KagiResult<VaultMeta> {
    // Wrap immediately so the string buffer is zeroized on any early return
    let mut password = Zeroizing::new(password);

    let path = PathBuf::from(&path);
    let vault_name = if name.is_empty() {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unnamed")
            .to_string()
    } else {
        name
    };

    let mut db = keepass::Database::new();
    db.meta.database_name = Some(vault_name.clone());

    // Serialise to buffer first, then atomic-write — never truncate the
    // target directly; a crash mid-save leaves the original file intact.
    let key = keepass::DatabaseKey::new().with_password(&password);
    let mut buf = std::io::Cursor::new(Vec::new());
    db.save(&mut buf, key)?;
    let bytes = buf.into_inner();
    crate::vault::atomic_write(&path, &bytes)?;

    // Re-open from buffer to verify and obtain the in-memory DB
    let key = keepass::DatabaseKey::new().with_password(&password);
    let mut db = keepass::Database::open(&mut std::io::Cursor::new(bytes), key)
        .map_err(|e| KagiError::Vault(e.to_string()))?;

    // keepass 0.13 only supports saving KDBX4 — upgrade if needed
    ensure_kdbx4(&mut db);

    let entry_count = 0;
    let id = uuid::Uuid::new_v4();

    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    // Swap the String with an empty one via DerefMut, then convert to bytes.
    // The Zeroizing<String> (now holding "") drops harmlessly later.
    let pw_str = std::mem::take(&mut *password);
    vaults.insert(
        id,
        OpenVault {
            db,
            path: path.clone(),
            master_key: Zeroizing::new(pw_str.into_bytes()),
        },
    );

    Ok(VaultMeta {
        path: path.to_string_lossy().to_string(),
        name: vault_name,
        item_count: entry_count,
        sync_provider: detect_sync_provider(&path),
        kdf_needs_upgrade: false,
    })
}

#[tauri::command]
pub async fn vault_change_password(
    state: State<'_, AppState>,
    old_password: String,
    new_password: String,
) -> KagiResult<()> {
    // Wrap both immediately so they're zeroized on any early return
    let old_password = Zeroizing::new(old_password);
    let mut new_password = Zeroizing::new(new_password);

    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    // Find the single open vault
    let (_id, vault): (&VaultId, &mut OpenVault) =
        vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

    // Verify old password matches stored key (constant-time comparison)
    let stored_key = String::from_utf8_lossy(&vault.master_key);
    if !bool::from(stored_key.as_bytes().ct_eq(old_password.as_bytes())) {
        return Err(KagiError::Custom("Wrong password".to_string()));
    }

    let new_key = keepass::DatabaseKey::new().with_password(&new_password);
    let mut buf = std::io::Cursor::new(Vec::new());
    vault.db.save(&mut buf, new_key)?;
    let bytes = buf.into_inner();
    crate::vault::atomic_write(&vault.path, &bytes)?;

    // Swap the String with an empty one via DerefMut, then convert to bytes.
    // The Zeroizing<String> (now holding "") drops harmlessly later.
    let pw_str = std::mem::take(&mut *new_password);
    vault.master_key = Zeroizing::new(pw_str.into_bytes());
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
