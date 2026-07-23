use std::io::Read;

use keepass::config::KdfConfig;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;
use tauri::State;
use zeroize::{Zeroize, Zeroizing};

use super::entries::{
    build_entry_summaries, build_folder_summaries, ensure_recycle_bin, entry_is_trashed,
};
use crate::error::{HitsuError, HitsuResult};
use crate::models::{EmptyRecycleBinResult, VaultMeta, VaultRefreshResult};
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
pub fn validate_header(path: &Path) -> HitsuResult<()> {
    let mut file = File::open(path)?;
    let mut buf = [0u8; 12];
    file.read_exact(&mut buf)?;
    drop(file);

    // Magic bytes: 03 D9 A2 9A
    if buf[0..4] != KDBX_MAGIC {
        return Err(HitsuError::Custom(
            "Invalid KDBX identifier — not a KeePass file".to_string(),
        ));
    }

    // Version ID (bytes 4-7, little-endian u32)
    let version_id = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    if version_id != KEEPASS_2_ID && version_id != KEEPASS_LATEST_ID {
        return Err(HitsuError::Custom(format!(
            "Unsupported KDBX version identifier: {:#010x}",
            version_id
        )));
    }

    // Major version (bytes 10-11, little-endian u16)
    let major = u16::from_le_bytes([buf[10], buf[11]]);
    if major != 3 && major != 4 {
        return Err(HitsuError::Custom(format!(
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
    } else if path
        .components()
        .any(|component| component.as_os_str() == "Dropbox")
    {
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
pub fn validate_master_password(password: &str) -> HitsuResult<()> {
    if password.len() < MIN_MASTER_PASSWORD_LEN {
        return Err(HitsuError::Custom(format!(
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

/// KDF configuration for newly created vaults and KDF upgrades:
/// Argon2id with 64 MiB memory, 2 iterations, 4 lanes. Pinned explicitly so
/// vault security never silently depends on the keepass crate's defaults.
fn default_kdf_config() -> KdfConfig {
    KdfConfig::Argon2id {
        memory: 64 * 1024 * 1024,
        iterations: 2,
        parallelism: 4,
        version: argon2::Version::Version13,
    }
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

fn validate_kdf(kdf: &KdfConfig) -> HitsuResult<()> {
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
                return Err(HitsuError::Custom("KDF memory is below 1 MiB".to_string()));
            }
            if *iterations < 2 {
                return Err(HitsuError::Custom(
                    "KDF iterations must be at least 2".to_string(),
                ));
            }
            if *parallelism < 1 {
                return Err(HitsuError::Custom(
                    "KDF parallelism must be at least 1".to_string(),
                ));
            }
            Ok(())
        }
        KdfConfig::Aes { .. } => Err(HitsuError::Custom(
            "AES-KDF is not supported — use Argon2id".to_string(),
        )),
        _ => Err(HitsuError::Custom("Unsupported KDF".to_string())),
    }
}

fn empty_recycle_bin_database(db: &mut keepass::Database) -> (usize, bool) {
    let Some(recycle_id) = db.meta.recyclebin_uuid.map(keepass::db::GroupId::from_uuid) else {
        return (0, false);
    };
    let Some(recycle_group) = db.group(recycle_id) else {
        return (0, false);
    };
    let deleted_entries = db
        .iter_all_entries()
        .filter(|entry| entry_is_trashed(db, entry))
        .count();
    let entry_ids: Vec<_> = recycle_group.entries().map(|entry| entry.id()).collect();
    let child_group_ids: Vec<_> = recycle_group.groups().map(|group| group.id()).collect();
    let changed = !entry_ids.is_empty() || !child_group_ids.is_empty();

    for entry_id in entry_ids {
        if let Some(mut entry) = db.entry_mut(entry_id) {
            entry.track_changes().remove();
        }
    }
    for group_id in child_group_ids {
        if let Some(mut group) = db.group_mut(group_id) {
            let _ = group.track_changes().remove();
        }
    }
    (deleted_entries, changed)
}

async fn save_and_commit_database(
    state: &State<'_, AppState>,
    db: keepass::Database,
    key: keepass::DatabaseKey,
    path: PathBuf,
    expected_disk_hash: [u8; 32],
    verification_error: &'static str,
) -> HitsuResult<()> {
    let save_path = path.clone();
    let (db, disk_hash) = tauri::async_runtime::spawn_blocking(move || -> HitsuResult<_> {
        crate::vault::ensure_unmodified(&save_path, &expected_disk_hash)?;
        let mut buffer = std::io::Cursor::new(Vec::new());
        db.save(&mut buffer, key.clone())?;
        let bytes = buffer.into_inner();
        crate::vault::backed_up_atomic_write(&save_path, &bytes, |candidate| {
            let mut file = File::open(candidate).map_err(|error| error.to_string())?;
            keepass::Database::open(&mut file, key.clone())
                .map(|_| ())
                .map_err(|error| format!("{verification_error}: {error}"))
        })
        .map_err(HitsuError::Custom)?;
        Ok((db, crate::vault::sha256_bytes(&bytes)))
    })
    .await
    .map_err(HitsuError::from_join)??;

    let mut vaults = state.vaults.lock();
    if let Some((_id, vault)) = vaults
        .iter_mut()
        .find(|(_id, vault)| vault.path == path && vault.disk_hash == expected_disk_hash)
    {
        vault.db = db;
        vault.disk_hash = disk_hash;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_provider_requires_a_complete_dropbox_path_component() {
        assert_eq!(
            detect_sync_provider(Path::new("/home/user/Dropbox/vault.kdbx")),
            "dropbox"
        );
        assert_eq!(
            detect_sync_provider(Path::new("/home/user/Dropbox-backup/vault.kdbx")),
            "local"
        );
    }

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
        let dir = std::env::temp_dir().join(format!("hitsu-hdr-test-{}", uuid::Uuid::new_v4()));
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

    // ── default_kdf_config tests ─────────────────────────────────────────

    #[test]
    fn test_default_kdf_config_passes_validators() {
        let kdf = default_kdf_config();
        assert!(validate_kdf(&kdf).is_ok());
        assert!(!needs_kdf_upgrade(&kdf));
    }

    /// The pinned KDF must survive a save/open round-trip — this is what
    /// vault_create relies on when it re-opens the freshly written vault.
    #[test]
    fn test_default_kdf_config_survives_roundtrip() {
        use std::io::Cursor;
        let mut db = keepass::Database::new();
        db.config.kdf_config = default_kdf_config();
        ensure_kdbx4(&mut db);

        let key = || keepass::DatabaseKey::new().with_password("test-password-123");
        let mut buf = Cursor::new(Vec::new());
        db.save(&mut buf, key()).unwrap();

        let reopened = keepass::Database::open(&mut Cursor::new(buf.into_inner()), key()).unwrap();
        assert!(validate_kdf(&reopened.config.kdf_config).is_ok());
        assert!(
            !needs_kdf_upgrade(&reopened.config.kdf_config),
            "created vaults must not immediately report kdf_needs_upgrade"
        );
    }

    // ── vault maintenance tests ──────────────────────────────────────────

    #[test]
    fn empty_recycle_bin_removes_entries_and_nested_groups() {
        let mut db = keepass::Database::new();
        let recycle_id = ensure_recycle_bin(&mut db);
        let direct_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());
        db.root_mut()
            .add_entry_with_id(direct_id)
            .unwrap()
            .move_to(recycle_id)
            .unwrap();
        let nested_id = {
            let mut recycle = db.group_mut(recycle_id).unwrap();
            let mut nested = recycle.add_group();
            nested.name = "Nested".into();
            nested.id()
        };
        db.group_mut(nested_id)
            .unwrap()
            .add_entry_with_id(keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4()))
            .unwrap();

        assert_eq!(empty_recycle_bin_database(&mut db), (2, true));
        let recycle = db.group(recycle_id).unwrap();
        assert_eq!(recycle.entries().count(), 0);
        assert_eq!(recycle.groups().count(), 0);
        assert_eq!(empty_recycle_bin_database(&mut db), (0, false));
    }

    // ── validate_master_password tests ────────────────────────────────────

    #[test]
    fn empty_recycle_bin_reports_changes_for_empty_nested_folders() {
        let mut db = keepass::Database::new();
        let recycle_id = ensure_recycle_bin(&mut db);
        db.group_mut(recycle_id).unwrap().add_group();

        assert_eq!(empty_recycle_bin_database(&mut db), (0, true));
        assert_eq!(db.group(recycle_id).unwrap().groups().count(), 0);
    }

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
) -> HitsuResult<VaultMeta> {
    // Wrap immediately so the buffer is zeroized on any early return
    let password = Zeroizing::new(password);

    // Take the writer lock so we never read the file while a queued save is
    // mid-write — the bytes we hash below must be the bytes we parsed.
    let _save_guard = state.save_lock.lock().await;

    let path = PathBuf::from(&path);
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unnamed")
        .to_string();

    // The Argon2 KDF inside Database::open takes hundreds of ms at 64 MiB —
    // run it on a blocking thread so other commands stay responsive.
    let open_path = path.clone();
    type OpenResult = (keepass::Database, keepass::DatabaseKey, [u8; 32], [u8; 32]);
    let (db, db_key, password_hash, disk_hash) =
        tauri::async_runtime::spawn_blocking(move || -> HitsuResult<OpenResult> {
            // Wrap in Zeroizing again inside the closure so the buffer is
            // scrubbed when the task finishes, success or not.
            let mut password = password;

            // Validate header early before passing to keepass
            validate_header(&open_path)?;

            // Read the whole file once: the same bytes are hashed (for
            // external-modification detection on later saves) and parsed.
            let bytes = std::fs::read(&open_path)?;
            let disk_hash = crate::vault::sha256_bytes(&bytes);
            let key = keepass::DatabaseKey::new().with_password(&password);
            let mut db = keepass::Database::parse(&bytes, key)?;

            validate_kdf(&db.config.kdf_config)?;

            // keepass 0.13 only supports saving KDBX4 — upgrade if needed
            ensure_kdbx4(&mut db);

            // Build the DatabaseKey; then early-zeroize the source password
            // String since the DatabaseKey holds its own copy (zeroized on drop).
            let db_key = keepass::DatabaseKey::new().with_password(&password);
            let password_hash = Sha256::digest(password.as_bytes()).into();
            password.zeroize();
            Ok((db, db_key, password_hash, disk_hash))
        })
        .await
        .map_err(HitsuError::from_join)??;

    let entry_count = db.num_entries();
    let id = uuid::Uuid::new_v4();

    // Build entry summaries while we have a reference to the db,
    // so the frontend doesn't need a second entries_list round-trip.
    let entries = build_entry_summaries(&db);
    let folders = build_folder_summaries(&db);

    let kdf_needs_upgrade = needs_kdf_upgrade(&db.config.kdf_config);

    let mut vaults = state.vaults.lock();
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
            password_hash,
            disk_hash,
        },
    );
    drop(vaults);
    state.arm_idle_lock();

    Ok(VaultMeta {
        path: path.to_string_lossy().to_string(),
        name,
        item_count: entry_count,
        sync_provider: detect_sync_provider(&path),
        kdf_needs_upgrade,
        entries,
        folders,
    })
}

type RefreshLoadResult = (bool, Option<(keepass::Database, [u8; 32])>);

/// Check whether another application replaced the open vault file and,
/// when allowed, reload the decrypted in-memory database from that file.
/// The save lock prevents this from racing any Hitsu writer.
#[tauri::command]
pub async fn vault_refresh_if_changed(
    state: State<'_, AppState>,
    allow_reload: bool,
) -> HitsuResult<VaultRefreshResult> {
    let _save_guard = state.save_lock.lock().await;

    let (path, key, expected_disk_hash, expected_root_id) = {
        let vaults = state.vaults.lock();
        let (_, vault) = vaults.iter().next().ok_or(HitsuError::NoOpenVault)?;
        (
            vault.path.clone(),
            vault.db_key.clone(),
            vault.disk_hash,
            vault.db.root().id(),
        )
    };

    let refresh_path = path.clone();
    let (changed, loaded) =
        tauri::async_runtime::spawn_blocking(move || -> HitsuResult<RefreshLoadResult> {
            let bytes = std::fs::read(&refresh_path)?;
            let disk_hash = crate::vault::sha256_bytes(&bytes);
            if disk_hash == expected_disk_hash {
                return Ok((false, None));
            }
            if !allow_reload {
                return Ok((true, None));
            }

            let mut db = keepass::Database::parse(&bytes, key)?;
            validate_kdf(&db.config.kdf_config)?;
            ensure_kdbx4(&mut db);
            if db.root().id() != expected_root_id {
                return Err(HitsuError::Custom(
                    "The vault path now contains a different database. Lock and reopen it.".into(),
                ));
            }

            // Do not install a snapshot if the external writer changed the
            // file again while Argon2 and parsing were running.
            let latest_hash = crate::vault::disk::sha256_file(&refresh_path)?;
            if latest_hash != disk_hash {
                return Err(HitsuError::Custom(
                    "The vault is still changing on disk. Try again shortly.".into(),
                ));
            }

            Ok((true, Some((db, disk_hash))))
        })
        .await
        .map_err(HitsuError::from_join)??;

    let Some((db, disk_hash)) = loaded else {
        return Ok(VaultRefreshResult {
            changed,
            reloaded: false,
            vault: None,
        });
    };

    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Unnamed")
        .to_string();
    let meta = VaultMeta {
        path: path.to_string_lossy().to_string(),
        name,
        item_count: db.num_entries(),
        sync_provider: detect_sync_provider(&path),
        kdf_needs_upgrade: needs_kdf_upgrade(&db.config.kdf_config),
        entries: build_entry_summaries(&db),
        folders: build_folder_summaries(&db),
    };

    let mut vaults = state.vaults.lock();
    let Some((_, vault)) = vaults.iter_mut().next() else {
        return Ok(VaultRefreshResult {
            changed: true,
            reloaded: false,
            vault: None,
        });
    };
    if vault.path != path || vault.disk_hash != expected_disk_hash {
        return Ok(VaultRefreshResult {
            changed: true,
            reloaded: false,
            vault: None,
        });
    }
    vault.db = db;
    vault.disk_hash = disk_hash;

    Ok(VaultRefreshResult {
        changed: true,
        reloaded: true,
        vault: Some(meta),
    })
}

#[tauri::command]
pub async fn vault_empty_recycle_bin(
    state: State<'_, AppState>,
) -> HitsuResult<EmptyRecycleBinResult> {
    let _save_guard = state.save_lock.lock().await;
    let (mut db, key, path, expected_disk_hash) = {
        let vaults = state.vaults.lock();
        let (_id, vault) = vaults.iter().next().ok_or(HitsuError::NoOpenVault)?;
        (
            vault.db.clone(),
            vault.db_key.clone(),
            vault.path.clone(),
            vault.disk_hash,
        )
    };
    let (deleted_entries, changed) = empty_recycle_bin_database(&mut db);
    if changed {
        save_and_commit_database(
            &state,
            db,
            key,
            path,
            expected_disk_hash,
            "Cannot re-open after emptying the Recycle Bin",
        )
        .await?;
    }
    Ok(EmptyRecycleBinResult { deleted_entries })
}

#[tauri::command]
pub async fn vault_upgrade_kdf(state: State<'_, AppState>) -> HitsuResult<()> {
    // Writer lock first (see AppState::save_lock ordering rules).
    let _save_guard = state.save_lock.lock().await;

    // Mutate + snapshot under a brief vaults lock.
    let (db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();

        let (_id, vault): (&VaultId, &mut OpenVault) =
            vaults.iter_mut().next().ok_or(HitsuError::NoOpenVault)?;

        // Upgrade KDF to Argon2id with 64 MiB
        vault.db.config.kdf_config = default_kdf_config();

        (
            vault.db.clone(),
            vault.db_key.clone(),
            vault.path.clone(),
            vault.disk_hash,
        )
    };

    // KDF + write + verification re-open (a second KDF) off the runtime.
    let save_path = path.clone();
    let new_disk_hash = tauri::async_runtime::spawn_blocking(move || -> HitsuResult<[u8; 32]> {
        // Abort before touching the file if another program changed it.
        crate::vault::ensure_unmodified(&save_path, &expected_disk_hash)?;

        // Re-save with the stored DatabaseKey (no raw password in memory)
        let mut buf = std::io::Cursor::new(Vec::new());
        db.save(&mut buf, key.clone())?;
        let bytes = buf.into_inner();

        crate::vault::backed_up_atomic_write(&save_path, &bytes, |path| {
            let mut file = File::open(path).map_err(|e| e.to_string())?;
            keepass::Database::open(&mut file, key.clone())
                .map(|_| ())
                .map_err(|e| format!("Cannot re-open after KDF upgrade: {}", e))
        })
        .map_err(HitsuError::Custom)?;

        Ok(crate::vault::sha256_bytes(&bytes))
    })
    .await
    .map_err(HitsuError::from_join)??;

    state.commit_disk_hash(&path, new_disk_hash);
    Ok(())
}

#[tauri::command]
pub async fn vault_create(
    state: State<'_, AppState>,
    path: String,
    password: String,
    name: String,
) -> HitsuResult<VaultMeta> {
    // Wrap immediately so the buffer is zeroized on any early return
    let password = Zeroizing::new(password);

    // Backend-enforced minimum: reject weak passwords before any I/O
    validate_master_password(&password)?;

    // Writer lock: creating writes a file that could be the currently open
    // vault's path — don't interleave with a queued save.
    let _save_guard = state.save_lock.lock().await;

    let path = PathBuf::from(&path);
    let vault_name = if name.is_empty() {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unnamed")
            .to_string()
    } else {
        name
    };

    // Save + verification re-open each run the Argon2 KDF — keep both off
    // the async runtime.
    let create_path = path.clone();
    let create_name = vault_name.clone();
    type CreateResult = (keepass::Database, keepass::DatabaseKey, [u8; 32], [u8; 32]);
    let (db, db_key, password_hash, disk_hash) =
        tauri::async_runtime::spawn_blocking(move || -> HitsuResult<CreateResult> {
            let mut password = password;

            let mut db = keepass::Database::new();
            db.meta.database_name = Some(create_name);
            ensure_recycle_bin(&mut db);

            // Pin the KDF and format explicitly instead of trusting the
            // library defaults.
            db.config.kdf_config = default_kdf_config();
            ensure_kdbx4(&mut db);

            // Serialise to buffer first, then atomic-write — never truncate the
            // target directly; a crash mid-save leaves the original file intact.
            let key = keepass::DatabaseKey::new().with_password(&password);
            let mut buf = std::io::Cursor::new(Vec::new());
            db.save(&mut buf, key)?;
            let bytes = buf.into_inner();
            crate::vault::atomic_write(&create_path, &bytes)?;
            let disk_hash = crate::vault::sha256_bytes(&bytes);

            // Re-open from buffer to verify and obtain the in-memory DB
            let key = keepass::DatabaseKey::new().with_password(&password);
            let mut db = keepass::Database::open(&mut std::io::Cursor::new(bytes), key)?;

            // Check the pinned KDF actually survived the save/open round-trip
            // rather than assuming it did.
            validate_kdf(&db.config.kdf_config)?;

            // keepass 0.13 only supports saving KDBX4 — upgrade if needed
            ensure_kdbx4(&mut db);

            // Build the DatabaseKey; then early-zeroize the source password
            // String since the DatabaseKey holds its own copy (zeroized on drop).
            let db_key = keepass::DatabaseKey::new().with_password(&password);
            let password_hash = Sha256::digest(password.as_bytes()).into();
            password.zeroize();
            Ok((db, db_key, password_hash, disk_hash))
        })
        .await
        .map_err(HitsuError::from_join)??;

    let entry_count = 0;
    let id = uuid::Uuid::new_v4();
    let kdf_needs_upgrade = needs_kdf_upgrade(&db.config.kdf_config);

    let mut vaults = state.vaults.lock();
    // Single-vault app: replace any previously open vault (see vault_open).
    vaults.clear();
    vaults.insert(
        id,
        OpenVault {
            db,
            path: path.clone(),
            db_key,
            password_hash,
            disk_hash,
        },
    );
    drop(vaults);
    state.arm_idle_lock();

    Ok(VaultMeta {
        path: path.to_string_lossy().to_string(),
        name: vault_name,
        item_count: entry_count,
        sync_provider: detect_sync_provider(&path),
        kdf_needs_upgrade,
        entries: Vec::new(),
        folders: Vec::new(),
    })
}

#[tauri::command]
pub async fn vault_change_password(
    state: State<'_, AppState>,
    old_password: String,
    new_password: String,
) -> HitsuResult<()> {
    // Wrap both immediately so they're zeroized on any early return
    let mut old_password = Zeroizing::new(old_password);
    let new_password = Zeroizing::new(new_password);

    // Backend-enforced minimum: reject weak new passwords
    validate_master_password(&new_password)?;

    // Writer lock first (see AppState::save_lock ordering rules).
    let _save_guard = state.save_lock.lock().await;

    // Verify the old password + snapshot under a brief vaults lock.
    let (db, path, expected_disk_hash) = {
        let vaults = state.vaults.lock();

        // Find the single open vault
        let (_id, vault): (&VaultId, &OpenVault) =
            vaults.iter().next().ok_or(HitsuError::NoOpenVault)?;

        // Verify old password matches the stored hash (constant-time).
        // Avoids timing side-channels from PartialEq on DatabaseKey.
        let old_hash = Sha256::digest(old_password.as_bytes());
        if vault.password_hash[..].ct_ne(&*old_hash).into() {
            return Err(HitsuError::Custom("Wrong password".to_string()));
        }

        (vault.db.clone(), vault.path.clone(), vault.disk_hash)
    };
    old_password.zeroize();

    // Derive the new key material before the password moves into the
    // blocking task (the verification re-open needs the raw password).
    let new_key = keepass::DatabaseKey::new().with_password(&new_password);
    let new_hash: [u8; 32] = Sha256::digest(new_password.as_bytes()).into();

    // Save + verification re-open each run the Argon2 KDF — off the runtime.
    let save_key = new_key.clone();
    let save_path = path.clone();
    let new_disk_hash = tauri::async_runtime::spawn_blocking(move || -> HitsuResult<[u8; 32]> {
        // Abort before touching the file if another program changed it.
        crate::vault::ensure_unmodified(&save_path, &expected_disk_hash)?;

        // new_password (Zeroizing) is scrubbed when this closure drops it.
        let mut buf = std::io::Cursor::new(Vec::new());
        db.save(&mut buf, save_key)?;
        let bytes = buf.into_inner();

        crate::vault::backed_up_atomic_write(&save_path, &bytes, |path| {
            let mut file = File::open(path).map_err(|e| e.to_string())?;
            let key = keepass::DatabaseKey::new().with_password(&new_password);
            keepass::Database::open(&mut file, key)
                .map(|_| ())
                .map_err(|e| format!("Cannot re-open with new password: {}", e))
        })
        .map_err(HitsuError::Custom)?;

        Ok(crate::vault::sha256_bytes(&bytes))
    })
    .await
    .map_err(HitsuError::from_join)??;

    // Commit the new key material. The vault may have been locked (or swapped
    // for another file) while the KDF ran; in that case there is nothing to
    // update in memory — the file on disk already uses the new password.
    let mut vaults = state.vaults.lock();
    if let Some((_id, vault)) = vaults.iter_mut().next() {
        if vault.path == path {
            vault.db_key = new_key;
            vault.password_hash = new_hash;
            vault.disk_hash = new_disk_hash;
        }
    }
    Ok(())
}

/// Immediately clear all backend secret state. Shared by the IPC command and
/// the OS session-lock monitor so locking does not depend on the webview being
/// awake enough to make an IPC call.
pub(crate) fn lock_open_vaults(state: &AppState) {
    // Clear the clipboard of any previously copied secrets (password, CVV, …)
    // so they don't linger after the vault is locked.
    super::clipboard::clear_clipboard_sync();

    // Clear all open vaults — this drops each OpenVault, which zeroizes
    // the master-key buffer via Zeroizing's Drop impl — and disarm the
    // watchdog until another vault opens.
    state.lock_open_vaults();
}

#[tauri::command]
pub async fn vault_lock(state: State<'_, AppState>) -> HitsuResult<()> {
    lock_open_vaults(&state);
    Ok(())
}
