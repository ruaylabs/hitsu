use base64::Engine;
use keepass::db::{fields, CustomDataItem, CustomDataValue, EntryId, Value};
use tauri::State;

use crate::error::{KagiError, KagiResult};
use crate::models::{
    AttachmentMeta, CardFields, Entry, EntryDraft, EntryPatch, EntrySummary, HistoryEntrySummary,
    IdentityFields, ItemType, SecretField,
};
use crate::state::{AppState, OpenVault};

/// Which KDBX standard field names should be stored as Protected values.
fn is_protected_key(key: &str) -> bool {
    matches!(
        key,
        fields::PASSWORD | "card.cvv" | "card.pin" | "card.number"
    )
}

fn map_entry_to_summary(entry_ref: &keepass::db::Entry) -> EntrySummary {
    let title = entry_ref.get_title().unwrap_or("").to_string();
    let username = entry_ref.get_username().unwrap_or("").to_string();
    let item_type = read_item_type(entry_ref);
    let icon_hint = read_icon_hint(entry_ref);
    let favorite = read_favorite(entry_ref);

    let url = entry_ref.get_url().map(str::to_string);

    // For card entries, show a masked card number as subtitle instead of username.
    let subtitle = if item_type == ItemType::Card {
        entry_ref
            .get("card.number")
            .and_then(mask_card_number)
            .unwrap_or(username.clone())
    } else {
        username.clone()
    };

    EntrySummary {
        id: entry_ref.id().uuid().to_string(),
        item_type,
        title,
        subtitle,
        url,
        username: Some(username),
        tags: entry_ref.tags.clone(),
        favorite,
        icon_hint,
    }
}

/// Build entry summaries for every entry in the database.
/// Exported for vault_open so the frontend doesn't need a second
/// round-trip (entries_list) after unlock.
pub(crate) fn build_entry_summaries(db: &keepass::Database) -> Vec<EntrySummary> {
    db.iter_all_entries()
        .map(|e| map_entry_to_summary(&e))
        .collect()
}

/// Mask a card number for display. First/last 4 digits are shown only when
/// the value is long enough (>= 12 chars — real PANs are 13–19) that they
/// don't overlap and at least four digits stay hidden; anything shorter is
/// masked entirely rather than leaked verbatim into the list subtitle.
/// Returns `None` for empty values so callers can fall back to the username.
fn mask_card_number(num: &str) -> Option<String> {
    if num.is_empty() {
        return None;
    }
    if num.len() >= 12 && num.is_ascii() {
        Some(format!("{} •••• {}", &num[..4], &num[num.len() - 4..]))
    } else {
        Some("••••".to_string())
    }
}

/// Read attachment metadata from an entry ref.
pub(crate) fn read_attachments(entry_ref: &keepass::db::EntryRef<'_>) -> Vec<AttachmentMeta> {
    entry_ref
        .attachments_named()
        .map(|(name, att)| AttachmentMeta {
            id: name.to_string(),
            name: name.to_string(),
            size_bytes: att.data.len() as u64,
        })
        .collect()
}

/// Map a KDBX entry to the webview detail model. Secrets are reduced to
/// presence flags / masked values here — see the `Entry` doc comment.
fn map_entry_to_full(entry_ref: &keepass::db::Entry) -> Entry {
    let id = entry_ref.id().uuid().to_string();
    let title = entry_ref.get_title().unwrap_or("").to_string();
    let username = entry_ref.get_username().unwrap_or("").to_string();
    let has_password = entry_ref.get_password().is_some_and(|p| !p.is_empty());
    let url = entry_ref.get_url().map(str::to_string);
    let notes = entry_ref.get(fields::NOTES).map(str::to_string);
    let tags = entry_ref.tags.clone();
    let item_type = read_item_type(entry_ref);
    let icon_hint = read_icon_hint(entry_ref);
    let favorite = read_favorite(entry_ref);
    let has_totp = read_totp_seed(entry_ref).is_some();

    let now = chrono::Utc::now().to_rfc3339();

    let modified_at = entry_ref
        .times
        .last_modification
        .map(|d: chrono::NaiveDateTime| d.and_utc().to_rfc3339())
        .unwrap_or_else(|| now.clone());

    let created_at = entry_ref
        .times
        .creation
        .map(|d: chrono::NaiveDateTime| d.and_utc().to_rfc3339())
        .unwrap_or_else(|| now.clone());

    let identity = if item_type == ItemType::Identity {
        Some(IdentityFields {
            first_name: entry_ref.get("identity.firstName").map(str::to_string),
            last_name: entry_ref.get("identity.lastName").map(str::to_string),
            email: entry_ref.get("identity.email").map(str::to_string),
            phone: entry_ref.get("identity.phone").map(str::to_string),
            address: entry_ref.get("identity.address").map(str::to_string),
            dob: entry_ref.get("identity.dob").map(str::to_string),
        })
    } else {
        None
    };

    let card = if item_type == ItemType::Card {
        let number = entry_ref.get("card.number");
        Some(CardFields {
            holder: entry_ref.get("card.holder").map(str::to_string),
            number_masked: number.and_then(mask_card_number),
            card_type: entry_ref.get("card.type").map(str::to_string),
            exp_month: entry_ref
                .get("card.expMonth")
                .and_then(|v: &str| v.parse().ok()),
            exp_year: entry_ref
                .get("card.expYear")
                .and_then(|v: &str| v.parse().ok()),
            has_number: number.is_some_and(|v| !v.is_empty()),
            has_cvv: entry_ref.get("card.cvv").is_some_and(|v| !v.is_empty()),
            has_pin: entry_ref.get("card.pin").is_some_and(|v| !v.is_empty()),
        })
    } else {
        None
    };

    Entry {
        id,
        item_type,
        title,
        subtitle: username.clone(),
        url,
        username: Some(username),
        has_password,
        has_totp,
        notes,
        tags,
        favorite,
        icon_hint,
        identity,
        card,
        attachments: Vec::new(),
        custom_fields: Vec::new(),
        modified_at,
        created_at,
        history_count: entry_ref
            .history
            .as_ref()
            .map_or(0, |h: &keepass::db::History| h.get_entries().len() as u32),
    }
}

fn read_custom_data_string(entry: &keepass::db::Entry, key: &str) -> Option<String> {
    entry.custom_data.get(key).and_then(|item| {
        item.value.as_ref().map(|cv| match cv {
            CustomDataValue::String(s) => s.clone(),
            CustomDataValue::Binary(b) => {
                // New-style storage (intentional Binary): value is valid UTF-8.
                if let Ok(s) = String::from_utf8(b.clone()) {
                    s
                } else {
                    // Old-style storage: the value was originally written as a plain string
                    // but keepass 0.13's XML deserialiser accidentally base64-decoded it
                    // because the string happened to be valid base64 (e.g. "note", "card",
                    // "true"). Recovery: base64-encode the binary back to the original string.
                    use base64::Engine;
                    base64::engine::general_purpose::STANDARD.encode(b)
                }
            }
        })
    })
}

fn read_item_type(entry: &keepass::db::Entry) -> ItemType {
    read_custom_data_string(entry, "kagi.itemType")
        .map(|v| ItemType::from_db_value(&v))
        .unwrap_or(ItemType::Login)
}

fn read_icon_hint(entry: &keepass::db::Entry) -> Option<String> {
    read_custom_data_string(entry, "kagi.iconHint")
}

fn read_favorite(entry: &keepass::db::Entry) -> bool {
    read_custom_data_string(entry, "kagi.favorite").is_some_and(|v| v == "true")
}

/// Look up an entry by UUID string. Works for all entries (flat map — nested groups included).
pub(crate) fn find_entry_ref<'a>(
    db: &'a keepass::Database,
    id: &str,
) -> Option<keepass::db::EntryRef<'a>> {
    let uuid = uuid::Uuid::parse_str(id).ok()?;
    db.entry(EntryId::from_uuid(uuid))
}

/// Remove an entry by UUID string.
fn remove_entry(db: &mut keepass::Database, id: &str) -> KagiResult<()> {
    let uuid = uuid::Uuid::parse_str(id).map_err(|_| KagiError::EntryNotFound(id.to_string()))?;
    let entry_id = EntryId::from_uuid(uuid);
    let em = db
        .entry_mut(entry_id)
        .ok_or_else(|| KagiError::EntryNotFound(id.to_string()))?;
    em.remove();
    Ok(())
}

/// Cheap in-memory snapshot of everything a save needs, taken under the
/// `vaults` lock so the KDF + write can run outside it.
fn snapshot_for_save(
    vault: &OpenVault,
) -> (
    keepass::Database,
    keepass::DatabaseKey,
    std::path::PathBuf,
    [u8; 32],
) {
    (
        vault.db.clone(),
        vault.db_key.clone(),
        vault.path.clone(),
        vault.disk_hash,
    )
}

/// Run KDF + serialize + atomic write on a blocking thread. Returns the
/// hash of the written bytes; the caller must commit it via
/// `AppState::commit_disk_hash` so the next save's conflict check passes.
///
/// Aborts with `ExternalModification` (writing nothing) if the file on disk
/// no longer hashes to `expected_disk_hash` — another program (sync client,
/// other KeePass app) changed it and we must not clobber those changes.
///
/// The caller must hold `AppState::save_lock` from before the in-memory
/// mutation until this completes (so a later writer can't hit the disk
/// first with a snapshot that supersedes ours), and must NOT hold the
/// `vaults` mutex while awaiting.
async fn save_snapshot(
    db: keepass::Database,
    key: keepass::DatabaseKey,
    path: std::path::PathBuf,
    expected_disk_hash: [u8; 32],
) -> KagiResult<[u8; 32]> {
    tauri::async_runtime::spawn_blocking(move || -> KagiResult<[u8; 32]> {
        crate::vault::ensure_unmodified(&path, &expected_disk_hash)?;
        let mut buf = std::io::Cursor::new(Vec::new());
        db.save(&mut buf, key)?;
        let bytes = buf.into_inner();
        crate::vault::atomic_write(&path, &bytes)?;
        Ok(crate::vault::sha256_bytes(&bytes))
    })
    .await
    .map_err(KagiError::from_join)?
}

fn set_kdbx_field(entry: &mut keepass::db::Entry, key: &str, value: Option<&str>) {
    match value {
        Some(v) => {
            if is_protected_key(key) {
                entry.set_protected(key, v);
            } else {
                entry.set_unprotected(key, v);
            }
        }
        None => {
            entry.fields.remove(key);
        }
    }
}

fn set_custom_data(entry: &mut keepass::db::Entry, key: &str, value: Option<&str>) {
    match value {
        Some(v) => {
            // Use Binary variant so the library base64-encodes on serialisation.
            // CustomDataValue::String is unreliable: the library's XML deserialiser
            // tries base64 decode first, so plain strings like "note", "login",
            // "true" are accidentally decoded as binary data.
            let item = CustomDataItem {
                value: Some(CustomDataValue::Binary(v.as_bytes().to_vec())),
                last_modification_time: None,
            };
            entry.custom_data.insert(key.to_string(), item);
        }
        None => {
            entry.custom_data.remove(key);
        }
    }
}

#[tauri::command]
pub async fn entries_list(state: State<'_, AppState>) -> KagiResult<Vec<EntrySummary>> {
    let vaults = state.vaults.lock();

    let (_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;

    Ok(vault
        .db
        .iter_all_entries()
        .map(|e| map_entry_to_summary(&e))
        .collect())
}

#[tauri::command]
pub async fn entry_get(state: State<'_, AppState>, id: String) -> KagiResult<Entry> {
    let vaults = state.vaults.lock();

    let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;

    let entry_ref =
        find_entry_ref(&vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
    let mut entry = map_entry_to_full(&entry_ref);
    entry.attachments = read_attachments(&entry_ref);
    Ok(entry)
}

#[tauri::command]
pub async fn entry_create(
    state: State<'_, AppState>,
    item_type: String,
    draft: EntryDraft,
) -> KagiResult<Entry> {
    let mut vaults = state.vaults.lock();

    let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

    let entry_id = EntryId::from_uuid(uuid::Uuid::new_v4());
    let id = entry_id.uuid().to_string();

    {
        let mut root = vault.db.root_mut();
        let mut em = root
            .add_entry_with_id(entry_id)
            .map_err(|_| KagiError::Custom("Duplicate entry ID (should not happen)".into()))?;

        em.set_unprotected(fields::TITLE, &draft.title);
        if let Some(ref u) = draft.username {
            em.set_unprotected(fields::USERNAME, u);
        }
        if let Some(ref p) = draft.password {
            em.set_protected(fields::PASSWORD, p);
        }
        if let Some(ref u) = draft.url {
            em.set_unprotected(fields::URL, u);
        }
        if let Some(ref n) = draft.notes {
            em.set_unprotected(fields::NOTES, n);
        }

        if let Some(ref t) = draft.totp {
            write_totp_seed(&mut em, t);
        }

        set_custom_data(&mut em, "kagi.itemType", Some(&item_type));
        set_custom_data(&mut em, "kagi.favorite", Some("false"));

        let now = chrono::Utc::now().naive_utc();
        em.times.creation = Some(now);
        em.times.last_modification = Some(now);
    }

    // NOTE: do NOT save here. The new entry lives only in memory until the
    // first `entry_update` (Save) persists it. If the user cancels, the stub
    // is dropped from memory via `entry_discard` without ever touching disk.

    let item_type_enum = ItemType::from_db_value(&item_type);
    let subtitle = draft.username.clone().unwrap_or_default();
    let now_rfc = chrono::Utc::now().to_rfc3339();

    Ok(Entry {
        id,
        item_type: item_type_enum,
        title: draft.title,
        subtitle,
        url: draft.url,
        username: draft.username,
        has_password: draft.password.as_deref().is_some_and(|p| !p.is_empty()),
        has_totp: draft.totp.as_deref().is_some_and(|t| !t.is_empty()),
        notes: draft.notes,
        tags: Vec::new(),
        favorite: false,
        icon_hint: None,
        identity: None,
        card: None,
        attachments: Vec::new(),
        custom_fields: Vec::new(),
        modified_at: now_rfc.clone(),
        created_at: now_rfc,
        history_count: 0,
    })
}

#[tauri::command]
pub async fn entry_update(
    state: State<'_, AppState>,
    id: String,
    patch: EntryPatch,
) -> KagiResult<Entry> {
    // Take the writer lock before mutating so no other save can interleave
    // between our in-memory commit and our disk write.
    let _save_guard = state.save_lock.lock().await;

    let (updated, db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();

        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

        let entry_id = EntryId::from_uuid(
            uuid::Uuid::parse_str(&id).map_err(|_| KagiError::EntryNotFound(id.clone()))?,
        );

        {
            let mut em = vault
                .db
                .entry_mut(entry_id)
                .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;

            // Use edit_tracking to automatically push the prior state into history
            em.edit_tracking(|tracked| {
                apply_patch(&mut *tracked, &patch);
            });
            // edit_tracking doesn't touch last_modification (apply_patch goes through
            // DerefMut → Entry::set_unprotected, not EntryTrack's tracked setters)
            em.times.last_modification = Some(chrono::Utc::now().naive_utc());
        }

        let entry_ref = vault
            .db
            .entry(entry_id)
            .ok_or(KagiError::EntryNotFound(id))?;
        let mut updated = map_entry_to_full(&entry_ref);
        updated.attachments = read_attachments(&entry_ref);
        let (db, key, path, expected_disk_hash) = snapshot_for_save(vault);
        (updated, db, key, path, expected_disk_hash)
    }; // vaults lock released — KDF + fsync run outside it

    let new_disk_hash = save_snapshot(db, key, path.clone(), expected_disk_hash).await?;
    state.commit_disk_hash(&path, new_disk_hash);
    Ok(updated)
}

/// Read the TOTP seed from an entry.
///
/// Tries the KeePassXC standard `otp` field first (an `otpauth://` URI),
/// then falls back to the legacy `TOTP Seed` + `TOTP Settings` fields.
pub(crate) fn read_totp_seed(entry: &keepass::db::Entry) -> Option<String> {
    // 1. Try the modern `otp` field (full otpauth:// URI — KeePassXC convention)
    if let Some(otp_uri) = entry.get_raw_otp_value() {
        // Validate that it parses as a valid TOTP URI
        if otp_uri.parse::<keepass::db::TOTP>().is_ok() {
            return Some(otp_uri.to_string());
        }
    }

    // 2. Fall back to legacy TOTP Seed + Settings fields
    let seed = entry.get("TOTP Seed")?;
    let settings = entry.get("TOTP Settings").unwrap_or("30;6");
    let mut parts = settings.split(';');
    let period = parts
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(30);
    let digits = parts
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(6);
    Some(format!(
        "otpauth://totp/entry?secret={}&period={}&digits={}",
        percent_encoding::utf8_percent_encode(seed, percent_encoding::NON_ALPHANUMERIC),
        period,
        digits
    ))
}

/// Write the TOTP seed to an entry (KeePassXC-compatible `otp` field).
fn write_totp_seed(entry: &mut keepass::db::Entry, uri: &str) {
    entry
        .fields
        .insert(fields::OTP.to_string(), Value::unprotected(uri.to_string()));
}

/// Apply an optional string field: `Some("")` clears the field (removes from KDBX),
/// `Some("value")` sets it, `None` leaves it unchanged.
fn apply_opt(entry: &mut keepass::db::Entry, key: &str, value: &Option<String>) {
    match value {
        Some(v) if v.is_empty() => set_kdbx_field(entry, key, None),
        Some(v) => set_kdbx_field(entry, key, Some(v)),
        None => {}
    }
}

fn apply_patch(entry: &mut keepass::db::Entry, patch: &EntryPatch) {
    apply_opt(entry, fields::TITLE, &patch.title);
    apply_opt(entry, fields::USERNAME, &patch.username);
    apply_opt(entry, fields::PASSWORD, &patch.password);
    apply_opt(entry, fields::URL, &patch.url);
    apply_opt(entry, fields::NOTES, &patch.notes);

    if let Some(ref v) = patch.totp {
        if v.is_empty() {
            entry.fields.remove(fields::OTP);
            entry.fields.remove("TOTP Seed");
            entry.fields.remove("TOTP Settings");
        } else {
            write_totp_seed(entry, v);
        }
    }
    if let Some(ref v) = patch.tags {
        entry.tags = v.clone();
    }
    if let Some(v) = patch.favorite {
        set_custom_data(
            entry,
            "kagi.favorite",
            Some(if v { "true" } else { "false" }),
        );
    }

    apply_opt(entry, "identity.firstName", &patch.first_name);
    apply_opt(entry, "identity.lastName", &patch.last_name);
    apply_opt(entry, "identity.email", &patch.email);
    apply_opt(entry, "identity.phone", &patch.phone);
    apply_opt(entry, "identity.address", &patch.address);
    apply_opt(entry, "card.holder", &patch.card_holder);
    apply_opt(entry, "card.number", &patch.card_number);
    apply_opt(entry, "card.type", &patch.card_type);
    apply_opt(entry, "card.expMonth", &patch.card_exp_month);
    apply_opt(entry, "card.expYear", &patch.card_exp_year);
    apply_opt(entry, "card.cvv", &patch.card_cvv);
    apply_opt(entry, "card.pin", &patch.card_pin);
}

#[tauri::command]
pub async fn entry_delete(state: State<'_, AppState>, id: String) -> KagiResult<()> {
    let _save_guard = state.save_lock.lock().await;

    let (db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();

        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

        remove_entry(&mut vault.db, &id)?;
        snapshot_for_save(vault)
    };

    let new_disk_hash = save_snapshot(db, key, path.clone(), expected_disk_hash).await?;
    state.commit_disk_hash(&path, new_disk_hash);
    Ok(())
}

/// Drop a brand-new, never-persisted entry from the in-memory database
/// without writing to disk. Used when the user cancels creation of an entry
/// that `entry_create` added to memory but never saved.
#[tauri::command]
pub async fn entry_discard(state: State<'_, AppState>, id: String) -> KagiResult<()> {
    let mut vaults = state.vaults.lock();

    let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

    remove_entry(&mut vault.db, &id)?;
    // Intentionally no save_vault(): the entry was never on disk.
    Ok(())
}

/// Read a secret field's plaintext from an entry, or from one of its history
/// versions when `version` is given.
fn read_secret_value(
    vault: &OpenVault,
    id: &str,
    field: SecretField,
    version: Option<u32>,
) -> KagiResult<String> {
    let entry_ref =
        find_entry_ref(&vault.db, id).ok_or_else(|| KagiError::EntryNotFound(id.to_string()))?;

    let read = |e: &keepass::db::Entry| -> Option<String> {
        match field {
            SecretField::Password => e.get_password().map(str::to_string),
            SecretField::Totp => read_totp_seed(e),
            SecretField::CardNumber => e.get("card.number").map(str::to_string),
            SecretField::CardCvv => e.get("card.cvv").map(str::to_string),
            SecretField::CardPin => e.get("card.pin").map(str::to_string),
        }
    };

    let value = match version {
        None => read(&entry_ref),
        Some(v) => {
            let history = entry_ref
                .history
                .as_ref()
                .ok_or_else(|| KagiError::Custom("No history for this entry".into()))?;
            let history_entry = history
                .get_entries()
                .get(v as usize)
                .ok_or_else(|| KagiError::Custom(format!("Version {} not found in history", v)))?;
            read(history_entry)
        }
    };

    Ok(value.unwrap_or_default())
}

/// Return a secret field's plaintext to the webview.
///
/// This is the only command that sends secrets over IPC, and it runs solely
/// on explicit user action (reveal button, populating the edit form) — never
/// on selection. For copy, use `entry_copy_field`, which keeps the secret
/// backend-side.
#[tauri::command]
pub async fn entry_reveal_field(
    state: State<'_, AppState>,
    id: String,
    field: SecretField,
    version: Option<u32>,
) -> KagiResult<String> {
    let vaults = state.vaults.lock();
    let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
    read_secret_value(vault, &id, field, version)
}

/// Copy a secret field to the clipboard entirely inside Rust — the plaintext
/// never crosses IPC to the webview. `timeout_secs = 0` disables auto-clear.
#[tauri::command]
pub async fn entry_copy_field(
    state: State<'_, AppState>,
    id: String,
    field: SecretField,
    timeout_secs: u64,
    version: Option<u32>,
) -> KagiResult<()> {
    let value = {
        let vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
        zeroize::Zeroizing::new(read_secret_value(vault, &id, field, version)?)
    }; // lock released before touching the clipboard
    super::clipboard::copy_secret(value, timeout_secs)
}

#[tauri::command]
pub async fn entry_history_list(
    state: State<'_, AppState>,
    id: String,
) -> KagiResult<Vec<HistoryEntrySummary>> {
    let vaults = state.vaults.lock();

    let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;

    let entry_ref =
        find_entry_ref(&vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;

    let history = entry_ref
        .history
        .as_ref()
        .ok_or_else(|| KagiError::Custom("No history for this entry".into()))?;

    let now = chrono::Utc::now().to_rfc3339();

    Ok(history
        .get_entries()
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let title = e.get_title().unwrap_or("").to_string();
            let modified_at = e
                .times
                .last_modification
                .map(|d: chrono::NaiveDateTime| d.and_utc().to_rfc3339())
                .unwrap_or_else(|| now.clone());
            HistoryEntrySummary {
                version: i as u32,
                modified_at,
                title,
            }
        })
        .collect())
}

#[tauri::command]
pub async fn entry_history_get(
    state: State<'_, AppState>,
    id: String,
    version: u32,
) -> KagiResult<Entry> {
    let vaults = state.vaults.lock();

    let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;

    let entry_ref =
        find_entry_ref(&vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;

    let history = entry_ref
        .history
        .as_ref()
        .ok_or_else(|| KagiError::Custom("No history for this entry".into()))?;

    let history_entry = history
        .get_entries()
        .get(version as usize)
        .ok_or_else(|| KagiError::Custom(format!("Version {} not found in history", version)))?;

    let mut result = map_entry_to_full(history_entry);
    result.id = id;
    Ok(result)
}

/// Save an attachment to a file on disk.
#[tauri::command]
pub async fn entry_attachment_save(
    state: State<'_, AppState>,
    id: String,
    name: String,
    dest: String,
) -> KagiResult<u64> {
    let data = {
        let vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
        let entry_ref =
            find_entry_ref(&vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        let att = entry_ref
            .attachment_by_name(&name)
            .ok_or_else(|| KagiError::Custom(format!("Attachment not found: {name}")))?;
        att.data.clone()
    };

    let path = std::path::PathBuf::from(&dest);
    std::fs::write(&path, &*data)
        .map_err(|e| KagiError::Custom(format!("Failed to write file: {e}")))?;
    Ok(data.len() as u64)
}

/// Add an attachment to an entry. `data_b64` is base64-encoded binary data.
#[tauri::command]
pub async fn entry_attachment_add(
    state: State<'_, AppState>,
    id: String,
    name: String,
    data_b64: String,
) -> KagiResult<AttachmentMeta> {
    let _save_guard = state.save_lock.lock().await;

    let (meta, db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();

        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

        let entry_id = EntryId::from_uuid(
            uuid::Uuid::parse_str(&id).map_err(|_| KagiError::EntryNotFound(id.clone()))?,
        );

        let data = base64::engine::general_purpose::STANDARD
            .decode(&data_b64)
            .map_err(|e| KagiError::Custom(format!("Invalid base64 data: {e}")))?;
        let size_bytes = data.len() as u64;

        let meta = {
            let mut em = vault
                .db
                .entry_mut(entry_id)
                .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;

            em.add_attachment(name.clone(), Value::unprotected(data));
            em.times.last_modification = Some(chrono::Utc::now().naive_utc());

            AttachmentMeta {
                id: name.clone(),
                name,
                size_bytes,
            }
        };

        let (db, key, path, expected_disk_hash) = snapshot_for_save(vault);
        (meta, db, key, path, expected_disk_hash)
    };

    let new_disk_hash = save_snapshot(db, key, path.clone(), expected_disk_hash).await?;
    state.commit_disk_hash(&path, new_disk_hash);
    Ok(meta)
}

/// Remove an attachment from an entry.
#[tauri::command]
pub async fn entry_attachment_remove(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> KagiResult<()> {
    let _save_guard = state.save_lock.lock().await;

    let (db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();

        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

        let entry_id = EntryId::from_uuid(
            uuid::Uuid::parse_str(&id).map_err(|_| KagiError::EntryNotFound(id.clone()))?,
        );

        {
            let mut em = vault
                .db
                .entry_mut(entry_id)
                .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;

            em.remove_attachment_by_name(&name);
            em.times.last_modification = Some(chrono::Utc::now().naive_utc());
        }

        snapshot_for_save(vault)
    };

    let new_disk_hash = save_snapshot(db, key, path.clone(), expected_disk_hash).await?;
    state.commit_disk_hash(&path, new_disk_hash);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{mask_card_number, read_attachments, read_totp_seed};

    #[test]
    fn test_mask_full_length_pan_shows_ends() {
        assert_eq!(
            mask_card_number("4111111111111111").as_deref(),
            Some("4111 •••• 1111")
        );
        // 13 digits — shortest real PAN
        assert_eq!(
            mask_card_number("4222222222222").as_deref(),
            Some("4222 •••• 2222")
        );
    }

    #[test]
    fn test_mask_short_values_are_fully_hidden() {
        // Below 12 chars first/last-4 would overlap and leak digits.
        for v in ["1234", "12345678", "12345678901"] {
            assert_eq!(mask_card_number(v).as_deref(), Some("••••"), "leaked {v}");
        }
    }

    #[test]
    fn test_mask_non_ascii_is_fully_hidden() {
        // Byte-slicing a non-ASCII string could split a code point — mask it.
        assert_eq!(
            mask_card_number("４１１１１１１１１１１１１１１１").as_deref(),
            Some("••••")
        );
    }

    #[test]
    fn test_mask_empty_is_none() {
        assert_eq!(mask_card_number(""), None);
    }

    #[test]
    fn test_read_totp_seed_url_encodes_padded_base32() {
        // A Base32 secret with `=` padding — without URL-encoding the `=` would
        // be interpreted as a query-param separator or KV separator, mangling the URI.
        let padded_secret = "JBSWY3DPEHPK3PXP====";

        let mut db = keepass::Database::new();
        let entry_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());
        {
            let mut root = db.root_mut();
            let mut em = root
                .add_entry_with_id(entry_id)
                .expect("duplicate entry id");
            em.fields.insert(
                "TOTP Seed".to_string(),
                keepass::db::Value::unprotected(padded_secret),
            );
            em.fields.insert(
                "TOTP Settings".to_string(),
                keepass::db::Value::unprotected("30;6"),
            );
        } // drop EntryMut + RootMut, releasing the mutable borrow on db

        let entry = db.iter_all_entries().next().expect("entry should exist");
        let uri = read_totp_seed(&entry).expect("should produce a URI");

        // The seed must be URL-encoded so `=` comes through as `%3D`.
        let expected =
            "otpauth://totp/entry?secret=JBSWY3DPEHPK3PXP%3D%3D%3D%3D&period=30&digits=6";
        assert_eq!(uri, expected);

        // The produced URI must be parseable (the KeePass library URL-decodes the
        // %3D back to `=` internally). get_secret() strips Base32 padding, so we
        // compare without the padding chars.
        let totp: keepass::db::TOTP = uri.parse().expect("should be a valid TOTP URI");
        assert_eq!(
            totp.get_secret(),
            "JBSWY3DPEHPK3PXP",
            "secret should match (padding stripped by library)"
        );
        assert_eq!(totp.period, 30);
        assert_eq!(totp.digits, 6);
    }

    #[test]
    fn test_read_attachments_returns_name_and_size() {
        let mut db = keepass::Database::new();
        let entry_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());

        {
            let mut root = db.root_mut();
            let mut em = root
                .add_entry_with_id(entry_id)
                .expect("duplicate entry id");
            em.set_unprotected(keepass::db::fields::TITLE, "Test");

            em.add_attachment(
                "notes.txt",
                keepass::db::Value::unprotected(b"hello world".to_vec()),
            );
            em.add_attachment(
                "photo.jpg",
                keepass::db::Value::unprotected(vec![0u8; 4096]),
            );
        }

        let entry = db.iter_all_entries().next().expect("entry should exist");
        let metas = read_attachments(&entry);

        assert_eq!(metas.len(), 2);

        // Sort so order is deterministic
        let mut metas = metas;
        metas.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(metas[0].id, "notes.txt");
        assert_eq!(metas[0].name, "notes.txt");
        assert_eq!(metas[0].size_bytes, 11);

        assert_eq!(metas[1].id, "photo.jpg");
        assert_eq!(metas[1].name, "photo.jpg");
        assert_eq!(metas[1].size_bytes, 4096);
    }
}
