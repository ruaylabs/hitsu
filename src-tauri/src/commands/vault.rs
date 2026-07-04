use std::io::Read;

use keepass::config::KdfConfig;
use std::fs::File;
use std::path::{Path, PathBuf};
use tauri::State;
use zeroize::{Zeroize, Zeroizing};

use crate::error::{KagiError, KagiResult};
use crate::models::VaultMeta;
use crate::state::{AppState, OpenVault, VaultId};

// Compile-time assertion: confirm the rust-argon2 re-export resolves
// and argon2::Version::Version13 has the expected value (0x13).
// If rust-argon2 changes its API or the argon2 alias breaks, this will
// fail at compile time with a clear message.
const _: () = assert!(argon2::Version::Version13 as u32 == 0x13);

/// KDBX file format constants
const KDBX_MAGIC: [u8; 4] = [0x03, 0xd9, 0xa2, 0x9a];
const KEEPASS_2_ID: u32 = 0xb54bfb66;
const KEEPASS_LATEST_ID: u32 = 0xb54bfb67;

/// Validate the KDBX file header (magic bytes + version).
///
/// Reads the first 12 bytes of the file and checks:
/// - Magic bytes match `03 D9 A2 9A` (KDBX signature)
/// - Version identifier is `0xb54bfb66` (pre-release) or `0xb54bfb67` (KDBX 3/4)
/// - Major version is 3 or 4
///
/// This provides early rejection of non-KeePass files before the
/// `keepass` crate attempts to parse and decrypt.
pub fn validate_header(path: &Path) -> KagiResult<()> {
    let mut file = File::open(path)?;
    let mut buf = [0u8; 12];
    file.read_exact(&mut buf)?;
    drop(file);

    // Magic bytes: 03 D9 A2 9A
    if buf[0..4] != KDBX_MAGIC {
        return Err(KagiError::Custom(
            "Invalid KDBX identifier — not a KeePass file".to_string(),
        ));
    }

    // Version ID (bytes 4-7, little-endian u32)
    let version_id = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    if version_id != KEEPASS_2_ID && version_id != KEEPASS_LATEST_ID {
        return Err(KagiError::Custom(format!(
            "Unsupported KDBX version identifier: {:#010x}",
            version_id
        )));
    }

    // Major version (bytes 10-11, little-endian u16)
    let major = u16::from_le_bytes([buf[10], buf[11]]);
    if major != 3 && major != 4 {
        return Err(KagiError::Custom(format!(
            "Unsupported KDBX major version: {} (only 3 and 4 are supported)",
            major
        )));
    }

    Ok(())
}

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

/// Minimum master-password length enforced by the backend.
/// Frontend also gates on the strength meter, but this is the hard floor.
const MIN_MASTER_PASSWORD_LEN: usize = 8;

/// Validate a master-password candidate on the backend side.
/// Rejects anything too short regardless of what the frontend allows.
pub fn validate_master_password(password: &str) -> KagiResult<()> {
    if password.len() < MIN_MASTER_PASSWORD_LEN {
        return Err(KagiError::Custom(format!(
            "Master password must be at least {} characters",
            MIN_MASTER_PASSWORD_LEN
        )));
    }
    Ok(())
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

    // ── validate_header tests ────────────────────────────────────────────

    /// Helper: return the raw 12-byte header from a freshly created KDBX4.1 file.
    fn kdbx4_header_bytes() -> Vec<u8> {
        use std::io::Cursor;
        let db = keepass::Database::new();
        let mut buf = Cursor::new(Vec::new());
        db.save(&mut buf, keepass::DatabaseKey::new().with_password("t"))
            .unwrap();
        let bytes = buf.into_inner();
        bytes[..12].to_vec()
    }

    /// Write `data` to a temp file and return its path + parent guard.
    fn write_temp_file(data: &[u8]) -> (PathBuf, std::fs::File) {
        let dir = std::env::temp_dir().join(format!("kagi-hdr-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.kdbx");
        let mut f = File::create(&path).unwrap();
        use std::io::Write;
        f.write_all(data).unwrap();
        (path, f)
    }

    #[test]
    fn test_validate_header_accepts_kdbx4() {
        let header = kdbx4_header_bytes();
        let (path, _f) = write_temp_file(&header);
        let result = validate_header(&path);
        assert!(
            result.is_ok(),
            "real KDBX4 header should pass: {:?}",
            result
        );
    }

    #[test]
    fn test_validate_header_accepts_kdbx3() {
        // KDBX3 has the same magic + latest version ID, but major = 3
        let header: [u8; 12] = [
            0x03, 0xd9, 0xa2, 0x9a, // magic
            0x67, 0xfb, 0x4b, 0xb5, // KEEPASS_LATEST_ID (LE)
            0x01, 0x00, // minor = 1
            0x03, 0x00, // major = 3
        ];
        let (path, _f) = write_temp_file(&header);
        let result = validate_header(&path);
        assert!(result.is_ok(), "KDBX3 header should pass: {:?}", result);
    }

    #[test]
    fn test_validate_header_rejects_bad_magic() {
        let header = [0u8; 12];
        let (path, _f) = write_temp_file(&header);
        let result = validate_header(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("KDBX identifier"));
    }

    #[test]
    fn test_validate_header_rejects_unknown_version_id() {
        let mut header = kdbx4_header_bytes();
        header[4..8].copy_from_slice(&0xdeadbeefu32.to_le_bytes());
        let (path, _f) = write_temp_file(&header);
        let result = validate_header(&path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("version identifier"));
    }

    #[test]
    fn test_validate_header_rejects_unsupported_major_version() {
        // Magic OK, version ID OK, but major = 5
        let header: [u8; 12] = [
            0x03, 0xd9, 0xa2, 0x9a, // magic
            0x67, 0xfb, 0x4b, 0xb5, // KEEPASS_LATEST_ID
            0x00, 0x00, // minor = 0
            0x05, 0x00, // major = 5 (unsupported)
        ];
        let (path, _f) = write_temp_file(&header);
        let result = validate_header(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("major version"));
    }

    #[test]
    fn test_validate_header_rejects_truncated() {
        // Only 8 bytes — read_exact will fail with unexpected EOF
        let data = [0x03, 0xd9, 0xa2, 0x9a, 0x67, 0xfb, 0x4b, 0xb5];
        let (path, _f) = write_temp_file(&data);
        let result = validate_header(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_header_file_not_found() {
        let path = PathBuf::from("/tmp/does-not-exist-123456.kdbx");
        let result = validate_header(&path);
        assert!(result.is_err());
    }

    // ── validate_kdf tests ───────────────────────────────────────────────

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

    /// Confirm argon2::Version::Version13 resolves to 0x13.
    /// If rust-argon2 changes its version scheme or the re-export breaks,
    /// this test catches it early.
    #[test]
    fn test_argon2_version_constant() {
        assert_eq!(argon2::Version::Version13 as u32, 0x13);
        assert_eq!(argon2::Version::default(), argon2::Version::Version13);
    }

    // ── validate_master_password tests ────────────────────────────────────

    #[test]
    fn test_validate_master_password_rejects_empty() {
        assert!(validate_master_password("").is_err());
    }

    #[test]
    fn test_validate_master_password_rejects_short() {
        for len in 1..MIN_MASTER_PASSWORD_LEN {
            let pw = "a".repeat(len);
            let result = validate_master_password(&pw);
            assert!(result.is_err(), "expected error for len={}", len);
        }
    }

    #[test]
    fn test_validate_master_password_accepts_minimum() {
        let pw = "a".repeat(MIN_MASTER_PASSWORD_LEN);
        assert!(validate_master_password(&pw).is_ok());
    }

    #[test]
    fn test_validate_master_password_accepts_longer() {
        let pw = "a".repeat(MIN_MASTER_PASSWORD_LEN + 10);
        assert!(validate_master_password(&pw).is_ok());
    }

    #[test]
    fn test_validate_master_password_error_message() {
        let result = validate_master_password("short");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("at least"));
        assert!(msg.contains("8"));
    }
}

#[tauri::command]
pub async fn vault_open(
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> KagiResult<VaultMeta> {
    // Wrap immediately so the buffer is zeroized on any early return
    let mut password = Zeroizing::new(password);

    let path = PathBuf::from(&path);

    // Validate header early before passing to keepass
    validate_header(&path)?;
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

    let kdf_needs_upgrade = needs_kdf_upgrade(&db.config.kdf_config);

    // Build the DatabaseKey; then early-zeroize the source password String
    // since the DatabaseKey holds its own copy (zeroized on drop).
    let db_key = keepass::DatabaseKey::new().with_password(&password);
    password.zeroize();
    // Single-vault app: every read uses vaults.iter().next(), so opening a
    // new vault must replace any previously open one — otherwise stale
    // entries from the old vault leak through and can shadow the new one
    // (entries_list returned the wrong vault's items).
    vaults.clear();
    vaults.insert(
        id,
        OpenVault {
            db,
            path: path.clone(),
            db_key,
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

    // Re-save with the stored DatabaseKey (no raw password in memory)
    let key = vault.db_key.clone();
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
    // Wrap immediately so the buffer is zeroized on any early return
    let mut password = Zeroizing::new(password);

    // Backend-enforced minimum: reject weak passwords before any I/O
    validate_master_password(&password)?;

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

    // Build the DatabaseKey; then early-zeroize the source password String
    // since the DatabaseKey holds its own copy (zeroized on drop).
    let db_key = keepass::DatabaseKey::new().with_password(&password);
    password.zeroize();
    // Single-vault app: replace any previously open vault (see vault_open).
    vaults.clear();
    vaults.insert(
        id,
        OpenVault {
            db,
            path: path.clone(),
            db_key,
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
    let mut old_password = Zeroizing::new(old_password);
    let mut new_password = Zeroizing::new(new_password);

    // Backend-enforced minimum: reject weak new passwords
    validate_master_password(&new_password)?;

    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    // Find the single open vault
    let (_id, vault): (&VaultId, &mut OpenVault) =
        vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

    // Verify old password matches the stored key
    if vault.db_key != keepass::DatabaseKey::new().with_password(&old_password) {
        return Err(KagiError::Custom("Wrong password".to_string()));
    }

    let new_key = keepass::DatabaseKey::new().with_password(&new_password);
    let mut buf = std::io::Cursor::new(Vec::new());
    vault.db.save(&mut buf, new_key)?;
    let bytes = buf.into_inner();
    crate::vault::atomic_write(&vault.path, &bytes)?;

    // Store the new DatabaseKey; early-zeroize both source buffers
    vault.db_key = keepass::DatabaseKey::new().with_password(&new_password);
    old_password.zeroize();
    new_password.zeroize();
    Ok(())
}

#[tauri::command]
pub async fn vault_lock(state: State<'_, AppState>) -> KagiResult<()> {
    // Clear the clipboard of any previously copied secrets (password, CVV, …)
    // so they don't linger after the vault is locked.
    super::clipboard::clear_clipboard_sync();

    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    // Clear all open vaults — this drops each OpenVault, which zeroizes
    // the master-key buffer via Zeroizing's Drop impl.
    vaults.clear();

    Ok(())
}
