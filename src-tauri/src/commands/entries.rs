use keepass::db::{fields, CustomDataItem, CustomDataValue, EntryId, GroupId, Value};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::{KagiError, KagiResult};
use crate::models::{
    AttachmentMeta, CardFields, CustomField, Entry, EntryDraft, EntryEditPayload, EntryPatch,
    EntrySummary, FolderSummary, HistoryEntrySummary, IdentityFields, ItemType, PassportFields,
    SecretField, SoftwareLicenseFields,
};
use crate::state::{AppState, OpenVault};

/// Which KDBX standard field names should be stored as Protected values.
fn is_protected_key(key: &str) -> bool {
    matches!(
        key,
        fields::PASSWORD
            | "card.cvv"
            | "card.pin"
            | "card.number"
            | "license.key"
            | "passport.number"
    )
}

fn map_entry_to_summary(entry_ref: &keepass::db::EntryRef<'_>, trashed: bool) -> EntrySummary {
    let title = entry_ref.get_title().unwrap_or("").to_string();
    let username = entry_ref.get_username().unwrap_or("").to_string();
    let item_type = read_item_type(entry_ref);
    let icon_hint = read_icon_hint(entry_ref);
    let favorite = read_favorite(entry_ref);

    let url = entry_ref.get_url().map(str::to_string);

    let subtitle = match item_type {
        ItemType::Card => entry_ref
            .get("card.number")
            .and_then(mask_card_number)
            .unwrap_or(username.clone()),
        ItemType::SoftwareLicense => entry_ref
            .get("license.version")
            .unwrap_or(&username)
            .to_string(),
        ItemType::Passport => entry_ref
            .get("passport.fullName")
            .or_else(|| entry_ref.get("passport.issuingCountry"))
            .unwrap_or(&username)
            .to_string(),
        _ => username.clone(),
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
        trashed,
        folder_id: entry_folder_id(entry_ref, trashed),
        icon_hint,
    }
}

fn group_is_in_recycle_bin(db: &keepass::Database, group_id: GroupId) -> bool {
    let Some(recycle_id) = db.meta.recyclebin_uuid.map(GroupId::from_uuid) else {
        return false;
    };
    let mut current = Some(group_id);
    while let Some(id) = current {
        if id == recycle_id {
            return true;
        }
        current = db
            .group(id)
            .and_then(|group| group.parent().map(|parent| parent.id()));
    }
    false
}

pub(crate) fn entry_is_trashed(db: &keepass::Database, entry: &keepass::db::EntryRef<'_>) -> bool {
    group_is_in_recycle_bin(db, entry.parent().id())
}

/// Build entry summaries for every entry in the database, including recycle-bin
/// entries (marked with `trashed` so the frontend can keep them out of normal views).
/// `pub` (not `pub(crate)`) so integration tests can assert on the summary list.
pub fn build_entry_summaries(db: &keepass::Database) -> Vec<EntrySummary> {
    db.iter_all_entries()
        .map(|e| map_entry_to_summary(&e, entry_is_trashed(db, &e)))
        .collect()
}

pub fn build_folder_summaries(db: &keepass::Database) -> Vec<FolderSummary> {
    let root_id = db.root().id();
    let mut folders = db
        .iter_all_groups()
        .filter(|group| group.id() != root_id && !group_is_in_recycle_bin(db, group.id()))
        .map(|group| FolderSummary {
            id: group.id().uuid().to_string(),
            name: group.name.clone(),
            parent_id: group
                .parent()
                .filter(|parent| parent.id() != root_id)
                .map(|parent| parent.id().uuid().to_string()),
        })
        .collect::<Vec<_>>();
    folders.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
    folders
}

fn entry_folder_id(entry: &keepass::db::EntryRef<'_>, trashed: bool) -> Option<String> {
    let parent = entry.parent();
    (!trashed && parent.id() != entry.database().root().id())
        .then(|| parent.id().uuid().to_string())
}

fn entry_matches_search(entry: &keepass::db::Entry, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    entry
        .tags
        .iter()
        .any(|tag| tag.to_lowercase().contains(&query))
        || entry.fields.iter().any(|(name, value)| {
            let custom_name_matches = name
                .strip_prefix(CUSTOM_FIELD_PREFIX)
                .is_some_and(|name| name.to_lowercase().contains(&query));
            custom_name_matches
                || (!value.is_protected() && value.get().to_lowercase().contains(&query))
        })
}

/// Mask a card number for display. First/last 4 digits are shown only when
/// the value is long enough (>= 12 chars — real PANs are 13–19) that they
/// don't overlap and at least four digits stay hidden; anything shorter is
/// masked entirely rather than leaked verbatim into the list subtitle.
/// Returns `None` for empty values so callers can fall back to the username.
fn entry_expiration_date(entry: &keepass::db::Entry) -> Option<String> {
    if !entry.times.expires.unwrap_or(false) {
        return None;
    }
    entry.times.expiry.map(|expiry| expiry.date().to_string())
}

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
fn map_entry_to_full(
    entry_ref: &keepass::db::Entry,
    trashed: bool,
    folder_id: Option<String>,
) -> Entry {
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
    let subtitle = match item_type {
        ItemType::Card => entry_ref
            .get("card.number")
            .and_then(mask_card_number)
            .unwrap_or(username.clone()),
        ItemType::SoftwareLicense => entry_ref
            .get("license.version")
            .unwrap_or(&username)
            .to_string(),
        ItemType::Passport => entry_ref
            .get("passport.fullName")
            .or_else(|| entry_ref.get("passport.issuingCountry"))
            .unwrap_or(&username)
            .to_string(),
        _ => username.clone(),
    };

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

    let software_license = if item_type == ItemType::SoftwareLicense {
        Some(SoftwareLicenseFields {
            version: entry_ref.get("license.version").map(str::to_string),
            has_license_key: entry_ref
                .get("license.key")
                .is_some_and(|value| !value.is_empty()),
            licensed_to: entry_ref.get("license.licensedTo").map(str::to_string),
            registered_email: entry_ref.get("license.registeredEmail").map(str::to_string),
            company: entry_ref.get("license.company").map(str::to_string),
            download_page: entry_ref.get("license.downloadPage").map(str::to_string),
            publisher: entry_ref.get("license.publisher").map(str::to_string),
            website: entry_ref.get("license.website").map(str::to_string),
            retail_price: entry_ref.get("license.retailPrice").map(str::to_string),
            support_email: entry_ref.get("license.supportEmail").map(str::to_string),
            purchase_date: entry_ref.get("license.purchaseDate").map(str::to_string),
            order_number: entry_ref.get("license.orderNumber").map(str::to_string),
            order_total: entry_ref.get("license.orderTotal").map(str::to_string),
        })
    } else {
        None
    };

    let passport = if item_type == ItemType::Passport {
        Some(PassportFields {
            passport_type: entry_ref.get("passport.type").map(str::to_string),
            issuing_country: entry_ref.get("passport.issuingCountry").map(str::to_string),
            has_number: entry_ref
                .get("passport.number")
                .is_some_and(|value| !value.is_empty()),
            full_name: entry_ref.get("passport.fullName").map(str::to_string),
            sex: entry_ref.get("passport.sex").map(str::to_string),
            nationality: entry_ref.get("passport.nationality").map(str::to_string),
            issuing_authority: entry_ref
                .get("passport.issuingAuthority")
                .map(str::to_string),
            birth_date: entry_ref.get("passport.birthDate").map(str::to_string),
            birth_place: entry_ref.get("passport.birthPlace").map(str::to_string),
            issue_date: entry_ref.get("passport.issueDate").map(str::to_string),
            expiry_date: entry_ref.get("passport.expiryDate").map(str::to_string),
        })
    } else {
        None
    };

    Entry {
        id,
        item_type,
        title,
        subtitle,
        url,
        username: Some(username),
        has_password,
        has_totp,
        notes,
        tags,
        favorite,
        trashed,
        folder_id,
        icon_hint,
        identity,
        card,
        software_license,
        passport,
        attachments: Vec::new(),
        custom_fields: read_custom_fields(entry_ref),
        expires_at: entry_expiration_date(entry_ref),
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

const CUSTOM_FIELD_PREFIX: &str = "custom.";

fn custom_field_storage_name(name: &str) -> String {
    format!("{CUSTOM_FIELD_PREFIX}{}", name.trim())
}

fn read_custom_fields(entry: &keepass::db::Entry) -> Vec<CustomField> {
    read_custom_fields_with_secrets(entry, false)
}

fn read_custom_fields_with_secrets(
    entry: &keepass::db::Entry,
    include_protected_values: bool,
) -> Vec<CustomField> {
    let mut custom_fields = entry
        .fields
        .iter()
        .filter_map(|(name, value)| {
            let display_name = name.strip_prefix(CUSTOM_FIELD_PREFIX)?;
            Some(CustomField {
                name: display_name.to_string(),
                value: if value.is_protected() && !include_protected_values {
                    String::new()
                } else {
                    value.get().clone()
                },
                protected: value.is_protected(),
            })
        })
        .collect::<Vec<_>>();
    custom_fields.sort_by_key(|field| field.name.to_lowercase());
    custom_fields
}

fn build_entry_edit_payload(entry: &keepass::db::Entry) -> EntryEditPayload {
    let value = |key: &str| entry.get(key).unwrap_or_default().to_string();
    EntryEditPayload {
        password: value(fields::PASSWORD),
        totp: read_totp_seed(entry).unwrap_or_default(),
        card_number: value("card.number"),
        card_cvv: value("card.cvv"),
        card_pin: value("card.pin"),
        license_key: value("license.key"),
        passport_number: value("passport.number"),
        custom_fields: read_custom_fields_with_secrets(entry, true),
    }
}

fn parse_entry_id(id: &str) -> KagiResult<EntryId> {
    uuid::Uuid::parse_str(id)
        .map(EntryId::from_uuid)
        .map_err(|_| KagiError::EntryNotFound(id.to_string()))
}

fn folder_destination(db: &keepass::Database, folder_id: &str) -> KagiResult<GroupId> {
    if folder_id.is_empty() {
        return Ok(db.root().id());
    }
    let id = uuid::Uuid::parse_str(folder_id)
        .map(GroupId::from_uuid)
        .map_err(|_| KagiError::Custom("Folder not found".into()))?;
    if db.group(id).is_none() || group_is_in_recycle_bin(db, id) {
        return Err(KagiError::Custom("Folder not found".into()));
    }
    Ok(id)
}

fn validate_folder_name(name: &str) -> KagiResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(KagiError::Custom("Folder name cannot be empty".into()));
    }
    if name.len() > 255 {
        return Err(KagiError::Custom("Folder name is too long".into()));
    }
    Ok(name.to_string())
}

/// Look up an entry by UUID string. Works for all entries (flat map — nested groups included).
pub(crate) fn find_entry_ref<'a>(
    db: &'a keepass::Database,
    id: &str,
) -> Option<keepass::db::EntryRef<'a>> {
    db.entry(parse_entry_id(id).ok()?)
}

/// Ensure the database has a KeePass-compatible recycle-bin group.
/// New Kagi vaults call this eagerly; imported vaults are upgraded lazily on
/// their first deletion if their recycle-bin metadata is absent or stale.
pub(crate) fn ensure_recycle_bin(db: &mut keepass::Database) -> GroupId {
    if let Some(id) = db.meta.recyclebin_uuid.map(GroupId::from_uuid) {
        if db.group(id).is_some() {
            db.meta.recyclebin_enabled = Some(true);
            return id;
        }
    }

    let id = {
        let mut root = db.root_mut();
        let mut group = root.add_group();
        group.name = "Recycle Bin".to_string();
        group.id()
    };
    let now = chrono::Utc::now().naive_utc();
    db.meta.recyclebin_enabled = Some(true);
    db.meta.recyclebin_uuid = Some(id.uuid());
    db.meta.recyclebin_changed = Some(now);
    id
}

/// Remove an entry by UUID string permanently.
fn remove_entry(db: &mut keepass::Database, id: &str) -> KagiResult<()> {
    let em = db
        .entry_mut(parse_entry_id(id)?)
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
pub async fn entry_get(state: State<'_, AppState>, id: String) -> KagiResult<Entry> {
    let vaults = state.vaults.lock();

    let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;

    let entry_ref =
        find_entry_ref(&vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
    let trashed = entry_is_trashed(&vault.db, &entry_ref);
    let folder_id = entry_folder_id(&entry_ref, trashed);
    let mut entry = map_entry_to_full(&entry_ref, trashed, folder_id);
    entry.attachments = read_attachments(&entry_ref);
    Ok(entry)
}

#[tauri::command]
pub async fn entries_search(state: State<'_, AppState>, query: String) -> KagiResult<Vec<String>> {
    let vaults = state.vaults.lock();
    let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
    Ok(vault
        .db
        .iter_all_entries()
        .filter(|entry| entry_matches_search(entry, &query))
        .map(|entry| entry.id().uuid().to_string())
        .collect())
}

#[tauri::command]
pub async fn entry_edit_payload(
    state: State<'_, AppState>,
    id: String,
) -> KagiResult<EntryEditPayload> {
    let vaults = state.vaults.lock();
    let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
    let entry_ref =
        find_entry_ref(&vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
    Ok(build_entry_edit_payload(&entry_ref))
}

#[tauri::command]
pub async fn entry_create(
    state: State<'_, AppState>,
    item_type: String,
    mut draft: EntryDraft,
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
        title: std::mem::take(&mut draft.title),
        subtitle,
        url: draft.url.take(),
        username: draft.username.take(),
        has_password: draft.password.as_deref().is_some_and(|p| !p.is_empty()),
        has_totp: draft.totp.as_deref().is_some_and(|t| !t.is_empty()),
        notes: draft.notes.take(),
        tags: Vec::new(),
        favorite: false,
        trashed: false,
        folder_id: None,
        icon_hint: None,
        identity: None,
        card: None,
        software_license: None,
        passport: None,
        attachments: Vec::new(),
        custom_fields: Vec::new(),
        expires_at: None,
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
    validate_custom_fields(&patch)?;
    validate_expiration(&patch)?;
    // Take the writer lock before mutating so no other save can interleave
    // between our in-memory commit and our disk write.
    let _save_guard = state.save_lock.lock().await;

    let (updated, db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();

        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

        let entry_id = parse_entry_id(&id)?;

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
        let trashed = entry_is_trashed(&vault.db, &entry_ref);
        let folder_id = entry_folder_id(&entry_ref, trashed);
        let mut updated = map_entry_to_full(&entry_ref, trashed, folder_id);
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
    apply_opt(entry, "identity.dob", &patch.dob);
    apply_opt(entry, "card.holder", &patch.card_holder);
    apply_opt(entry, "card.number", &patch.card_number);
    apply_opt(entry, "card.type", &patch.card_type);
    apply_opt(entry, "card.expMonth", &patch.card_exp_month);
    apply_opt(entry, "card.expYear", &patch.card_exp_year);
    apply_opt(entry, "card.cvv", &patch.card_cvv);
    apply_opt(entry, "card.pin", &patch.card_pin);
    apply_opt(entry, "license.version", &patch.license_version);
    apply_opt(entry, "license.key", &patch.license_key);
    apply_opt(entry, "license.licensedTo", &patch.license_licensed_to);
    apply_opt(
        entry,
        "license.registeredEmail",
        &patch.license_registered_email,
    );
    apply_opt(entry, "license.company", &patch.license_company);
    apply_opt(entry, "license.downloadPage", &patch.license_download_page);
    apply_opt(entry, "license.publisher", &patch.license_publisher);
    apply_opt(entry, "license.website", &patch.license_website);
    apply_opt(entry, "license.retailPrice", &patch.license_retail_price);
    apply_opt(entry, "license.supportEmail", &patch.license_support_email);
    apply_opt(entry, "license.purchaseDate", &patch.license_purchase_date);
    apply_opt(entry, "license.orderNumber", &patch.license_order_number);
    apply_opt(entry, "license.orderTotal", &patch.license_order_total);
    apply_opt(entry, "passport.type", &patch.passport_type);
    apply_opt(
        entry,
        "passport.issuingCountry",
        &patch.passport_issuing_country,
    );
    apply_opt(entry, "passport.number", &patch.passport_number);
    apply_opt(entry, "passport.fullName", &patch.passport_full_name);
    apply_opt(entry, "passport.sex", &patch.passport_sex);
    apply_opt(entry, "passport.nationality", &patch.passport_nationality);
    apply_opt(
        entry,
        "passport.issuingAuthority",
        &patch.passport_issuing_authority,
    );
    apply_opt(entry, "passport.birthDate", &patch.passport_birth_date);
    apply_opt(entry, "passport.birthPlace", &patch.passport_birth_place);
    apply_opt(entry, "passport.issueDate", &patch.passport_issue_date);
    apply_opt(entry, "passport.expiryDate", &patch.passport_expiry_date);

    if let Some(expires_at) = &patch.expires_at {
        if expires_at.is_empty() {
            entry.times.expires = Some(false);
            entry.times.expiry = None;
        } else if let Ok(date) = chrono::NaiveDate::parse_from_str(expires_at, "%Y-%m-%d") {
            entry.times.expires = Some(true);
            entry.times.expiry = date.and_hms_opt(23, 59, 59);
        }
    }

    if let Some(custom_fields) = &patch.custom_fields {
        let existing = entry
            .fields
            .keys()
            .filter(|name| name.starts_with(CUSTOM_FIELD_PREFIX))
            .cloned()
            .collect::<Vec<_>>();
        for name in existing {
            entry.fields.remove(&name);
        }
        for field in custom_fields {
            let value = if field.protected {
                Value::protected(field.value.clone())
            } else {
                Value::unprotected(field.value.clone())
            };
            entry
                .fields
                .insert(custom_field_storage_name(&field.name), value);
        }
    }
}

fn validate_expiration(patch: &EntryPatch) -> KagiResult<()> {
    let Some(expires_at) = patch
        .expires_at
        .as_deref()
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    chrono::NaiveDate::parse_from_str(expires_at, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| KagiError::Custom("Expiration date must use YYYY-MM-DD".into()))
}

fn validate_custom_fields(patch: &EntryPatch) -> KagiResult<()> {
    let Some(custom_fields) = &patch.custom_fields else {
        return Ok(());
    };
    if custom_fields.len() > 64 {
        return Err(KagiError::Custom(
            "An entry cannot have more than 64 custom fields".into(),
        ));
    }
    let mut names = std::collections::HashSet::new();
    for field in custom_fields {
        let name = field.name.trim();
        if name.is_empty() {
            return Err(KagiError::Custom(
                "Custom field names cannot be empty".into(),
            ));
        }
        if name.len() > 255 {
            return Err(KagiError::Custom("Custom field name is too long".into()));
        }
        if !names.insert(name.to_lowercase()) {
            return Err(KagiError::Custom(format!(
                "Custom field names must be unique: {name}"
            )));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn folder_create(
    state: State<'_, AppState>,
    parent_id: Option<String>,
    name: String,
) -> KagiResult<FolderSummary> {
    let _save_guard = state.save_lock.lock().await;
    let name = validate_folder_name(&name)?;

    let (folder, db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;
        let destination = folder_destination(&vault.db, parent_id.as_deref().unwrap_or(""))?;
        let root_id = vault.db.root().id();
        let folder = {
            let mut parent = vault
                .db
                .group_mut(destination)
                .ok_or_else(|| KagiError::Custom("Folder not found".into()))?;
            let mut group = parent.add_group();
            group.name = name.clone();
            FolderSummary {
                id: group.id().uuid().to_string(),
                name,
                parent_id: (destination != root_id).then(|| destination.uuid().to_string()),
            }
        };
        let (db, key, path, expected_disk_hash) = snapshot_for_save(vault);
        (folder, db, key, path, expected_disk_hash)
    };

    let new_disk_hash = save_snapshot(db, key, path.clone(), expected_disk_hash).await?;
    state.commit_disk_hash(&path, new_disk_hash);
    Ok(folder)
}

#[tauri::command]
pub async fn folder_rename(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> KagiResult<FolderSummary> {
    let _save_guard = state.save_lock.lock().await;
    let name = validate_folder_name(&name)?;

    let (folder, db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;
        let folder_id = uuid::Uuid::parse_str(&id)
            .map(GroupId::from_uuid)
            .map_err(|_| KagiError::Custom("Folder not found".into()))?;
        if folder_id == vault.db.root().id()
            || vault.db.group(folder_id).is_none()
            || group_is_in_recycle_bin(&vault.db, folder_id)
        {
            return Err(KagiError::Custom("Folder not found".into()));
        }
        let root_id = vault.db.root().id();
        let parent_id = vault.db.group(folder_id).and_then(|group| {
            group
                .parent()
                .filter(|parent| parent.id() != root_id)
                .map(|parent| parent.id().uuid().to_string())
        });
        {
            let mut group = vault
                .db
                .group_mut(folder_id)
                .ok_or_else(|| KagiError::Custom("Folder not found".into()))?;
            group.name = name.clone();
            group.times.last_modification = Some(chrono::Utc::now().naive_utc());
        }
        let folder = FolderSummary {
            id,
            name,
            parent_id,
        };
        let (db, key, path, expected_disk_hash) = snapshot_for_save(vault);
        (folder, db, key, path, expected_disk_hash)
    };

    let new_disk_hash = save_snapshot(db, key, path.clone(), expected_disk_hash).await?;
    state.commit_disk_hash(&path, new_disk_hash);
    Ok(folder)
}

#[tauri::command]
pub async fn entry_move(
    state: State<'_, AppState>,
    id: String,
    folder_id: Option<String>,
) -> KagiResult<Entry> {
    let _save_guard = state.save_lock.lock().await;

    let (updated, db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;
        let entry_id = parse_entry_id(&id)?;
        let destination = folder_destination(&vault.db, folder_id.as_deref().unwrap_or(""))?;
        let entry_ref = vault
            .db
            .entry(entry_id)
            .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        if entry_is_trashed(&vault.db, &entry_ref) {
            return Err(KagiError::Custom(
                "Entries in the Recycle Bin cannot be moved".into(),
            ));
        }

        let mut entry = vault
            .db
            .entry_mut(entry_id)
            .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        entry
            .move_to(destination)
            .map_err(|_| KagiError::Custom("Folder not found".into()))?;
        let now = chrono::Utc::now().naive_utc();
        entry.times.location_changed = Some(now);
        entry.times.last_modification = Some(now);

        let entry_ref = vault
            .db
            .entry(entry_id)
            .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        let folder_id = entry_folder_id(&entry_ref, false);
        let mut updated = map_entry_to_full(&entry_ref, false, folder_id);
        updated.attachments = read_attachments(&entry_ref);
        let (db, key, path, expected_disk_hash) = snapshot_for_save(vault);
        (updated, db, key, path, expected_disk_hash)
    };

    let new_disk_hash = save_snapshot(db, key, path.clone(), expected_disk_hash).await?;
    state.commit_disk_hash(&path, new_disk_hash);
    Ok(updated)
}

#[tauri::command]
pub async fn entry_delete(state: State<'_, AppState>, id: String) -> KagiResult<()> {
    let _save_guard = state.save_lock.lock().await;

    let (db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

        let recycle_id = ensure_recycle_bin(&mut vault.db);
        let entry_id = parse_entry_id(&id)?;
        let entry_ref = vault
            .db
            .entry(entry_id)
            .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        if entry_is_trashed(&vault.db, &entry_ref) {
            return Err(KagiError::Custom(
                "Entry is already in the Recycle Bin".into(),
            ));
        }
        let mut entry = vault
            .db
            .entry_mut(entry_id)
            .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        entry
            .move_to(recycle_id)
            .map_err(|_| KagiError::Custom("Recycle Bin is unavailable".into()))?;
        entry.times.location_changed = Some(chrono::Utc::now().naive_utc());
        snapshot_for_save(vault)
    };

    let new_disk_hash = save_snapshot(db, key, path.clone(), expected_disk_hash).await?;
    state.commit_disk_hash(&path, new_disk_hash);
    Ok(())
}

#[tauri::command]
pub async fn entry_restore(state: State<'_, AppState>, id: String) -> KagiResult<()> {
    let _save_guard = state.save_lock.lock().await;

    let (db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;
        let entry_id = parse_entry_id(&id)?;
        let entry_ref = vault
            .db
            .entry(entry_id)
            .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        if !entry_is_trashed(&vault.db, &entry_ref) {
            return Err(KagiError::Custom("Entry is not in the Recycle Bin".into()));
        }
        let root_id = vault.db.root().id();
        let destination = entry_ref
            .previous_parent()
            .map(|group| group.id())
            .filter(|group_id| !group_is_in_recycle_bin(&vault.db, *group_id))
            .unwrap_or(root_id);
        let mut entry = vault
            .db
            .entry_mut(entry_id)
            .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        entry
            .move_to(destination)
            .map_err(|_| KagiError::Custom("Original group is unavailable".into()))?;
        entry.times.location_changed = Some(chrono::Utc::now().naive_utc());
        snapshot_for_save(vault)
    };

    let new_disk_hash = save_snapshot(db, key, path.clone(), expected_disk_hash).await?;
    state.commit_disk_hash(&path, new_disk_hash);
    Ok(())
}

#[tauri::command]
pub async fn entry_delete_permanent(state: State<'_, AppState>, id: String) -> KagiResult<()> {
    let _save_guard = state.save_lock.lock().await;

    let (db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;
        let entry_ref =
            find_entry_ref(&vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        if !entry_is_trashed(&vault.db, &entry_ref) {
            return Err(KagiError::Custom(
                "Only entries in the Recycle Bin can be permanently deleted".into(),
            ));
        }
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
            SecretField::LicenseKey => e.get("license.key").map(str::to_string),
            SecretField::PassportNumber => e.get("passport.number").map(str::to_string),
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

fn read_custom_field_value(vault: &OpenVault, id: &str, name: &str) -> KagiResult<String> {
    let entry =
        find_entry_ref(&vault.db, id).ok_or_else(|| KagiError::EntryNotFound(id.to_string()))?;
    entry
        .fields
        .get(&custom_field_storage_name(name))
        .map(|value| value.get().clone())
        .ok_or_else(|| KagiError::Custom("Custom field not found".into()))
}

#[tauri::command]
pub async fn entry_reveal_custom_field(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> KagiResult<String> {
    let vaults = state.vaults.lock();
    let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
    read_custom_field_value(vault, &id, &name)
}

#[tauri::command]
pub async fn entry_copy_custom_field(
    state: State<'_, AppState>,
    id: String,
    name: String,
    timeout_secs: u64,
) -> KagiResult<()> {
    let value = {
        let vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
        zeroize::Zeroizing::new(read_custom_field_value(vault, &id, &name)?)
    };
    super::clipboard::copy_secret(value, timeout_secs)
}

fn sort_history_revisions_newest_first(revisions: &mut [HistoryEntrySummary]) {
    revisions.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
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

    let mut revisions = history
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
        .collect::<Vec<_>>();

    // KeePass files do not consistently preserve history insertion order.
    // Keep `version` as the underlying history index, but display newest first.
    sort_history_revisions_newest_first(&mut revisions);
    Ok(revisions)
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

    let mut result =
        map_entry_to_full(history_entry, entry_is_trashed(&vault.db, &entry_ref), None);
    result.id = id;
    Ok(result)
}

fn safe_attachment_file_name(name: &str) -> String {
    let file_name = name
        .rsplit(['/', '\\'])
        .find(|part| !part.is_empty() && *part != "." && *part != "..")
        .unwrap_or("attachment")
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>();

    if file_name.is_empty() {
        "attachment".to_string()
    } else {
        file_name
    }
}

fn write_attachment_file(path: &std::path::Path, data: &[u8]) -> KagiResult<()> {
    // Refuse an existing symlink rather than following it to an unrelated
    // user file. The native dialog already handles normal overwrite
    // confirmation for regular files.
    if std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Err(KagiError::Custom(
            "Refusing to save an attachment through a symbolic link".into(),
        ));
    }
    std::fs::write(path, data)
        .map_err(|e| KagiError::Custom(format!("Failed to write file: {e}")))?;
    Ok(())
}

/// Save an attachment using a native save dialog owned by the Rust backend.
///
/// The destination path never crosses IPC and cannot be supplied by webview
/// code. `None` means the user cancelled the native dialog.
#[tauri::command]
pub async fn entry_attachment_save(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> KagiResult<Option<u64>> {
    // Validate before opening a dialog, but do not clone the attachment data
    // yet: it may be large or sensitive, and the dialog can remain open for
    // an arbitrary amount of time.
    {
        let vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
        let entry_ref =
            find_entry_ref(&vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        entry_ref
            .attachment_by_name(&name)
            .ok_or_else(|| KagiError::Custom(format!("Attachment not found: {name}")))?;
    }

    // Treat attachment names from imported vaults as untrusted. Supplying only
    // the final component prevents a crafted name from steering the dialog to
    // another directory.
    let suggested_name = safe_attachment_file_name(&name);

    let Some(destination) = app
        .dialog()
        .file()
        .set_file_name(suggested_name)
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = destination
        .into_path()
        .map_err(|_| KagiError::Custom("The selected destination is not a local file".into()))?;

    // Re-read only after the user approves the destination. If the vault was
    // locked while the dialog was open, fail instead of retaining stale data.
    let data = {
        let vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
        let entry_ref =
            find_entry_ref(&vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        let attachment = entry_ref
            .attachment_by_name(&name)
            .ok_or_else(|| KagiError::Custom(format!("Attachment not found: {name}")))?;
        zeroize::Zeroizing::new(attachment.data.get().clone())
    };
    let bytes_written = data.len() as u64;

    tauri::async_runtime::spawn_blocking(move || write_attachment_file(&path, &data))
        .await
        .map_err(KagiError::from_join)??;

    Ok(Some(bytes_written))
}

fn read_attachment_file(
    path: &std::path::Path,
) -> KagiResult<(String, zeroize::Zeroizing<Vec<u8>>)> {
    let name = path
        .file_name()
        .map(|name| safe_attachment_file_name(&name.to_string_lossy()))
        .ok_or_else(|| KagiError::Custom("The selected attachment has no file name".into()))?;
    let data = zeroize::Zeroizing::new(std::fs::read(path)?);
    Ok((name, data))
}

/// Select and add an attachment using a Rust-owned native open dialog.
///
/// The path and attachment bytes stay in the backend. `None` means the user
/// cancelled the native dialog.
#[tauri::command]
pub async fn entry_attachment_add(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> KagiResult<Option<AttachmentMeta>> {
    let entry_id = parse_entry_id(&id)?;

    // Validate before opening a dialog. Revalidate after reading because the
    // vault may be locked or replaced while the dialog is open.
    {
        let vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
        vault
            .db
            .entry(entry_id)
            .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
    }

    let Some(selected) = app.dialog().file().blocking_pick_file() else {
        return Ok(None);
    };
    let selected_path = selected
        .into_path()
        .map_err(|_| KagiError::Custom("The selected attachment is not a local file".into()))?;

    let (name, mut data) =
        tauri::async_runtime::spawn_blocking(move || read_attachment_file(&selected_path))
            .await
            .map_err(KagiError::from_join)??;
    let size_bytes = data.len() as u64;

    let _save_guard = state.save_lock.lock().await;
    let (meta, db, key, path, expected_disk_hash) = {
        let mut vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

        let meta = {
            let mut em = vault
                .db
                .entry_mut(entry_id)
                .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;

            // Move the selected bytes into the database without leaving an
            // additional unsanitized buffer behind.
            em.add_attachment(name.clone(), Value::unprotected(std::mem::take(&mut *data)));
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
    Ok(Some(meta))
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

        let entry_id = parse_entry_id(&id)?;

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
    use super::{
        apply_patch, build_entry_edit_payload, build_entry_summaries, ensure_recycle_bin,
        entry_matches_search, map_entry_to_full, mask_card_number, parse_entry_id,
        read_attachment_file, read_attachments, read_totp_seed, safe_attachment_file_name,
        sort_history_revisions_newest_first, validate_custom_fields, validate_expiration,
        write_attachment_file,
    };
    use crate::models::{CustomField, EntryPatch, HistoryEntrySummary, ItemType};
    use keepass::db::fields;

    #[test]
    fn history_revisions_are_sorted_newest_first() {
        let mut revisions = vec![
            HistoryEntrySummary {
                version: 0,
                modified_at: "2025-01-01T00:00:00+00:00".into(),
                title: "Oldest".into(),
            },
            HistoryEntrySummary {
                version: 1,
                modified_at: "2025-03-01T00:00:00+00:00".into(),
                title: "Newest".into(),
            },
            HistoryEntrySummary {
                version: 2,
                modified_at: "2025-02-01T00:00:00+00:00".into(),
                title: "Middle".into(),
            },
        ];

        sort_history_revisions_newest_first(&mut revisions);

        assert_eq!(
            revisions
                .iter()
                .map(|revision| revision.version)
                .collect::<Vec<_>>(),
            vec![1, 2, 0]
        );
    }

    #[test]
    fn entry_id_parser_preserves_valid_ids_and_normalizes_errors() {
        let uuid = uuid::Uuid::new_v4();
        assert_eq!(parse_entry_id(&uuid.to_string()).unwrap().uuid(), uuid);

        let error = parse_entry_id("not-an-entry-id").unwrap_err();
        assert!(matches!(
            error,
            crate::error::KagiError::EntryNotFound(id) if id == "not-an-entry-id"
        ));
    }

    #[test]
    fn recycle_bin_is_created_once_and_marks_moved_entries() {
        let mut db = keepass::Database::new();
        let entry_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());
        db.root_mut()
            .add_entry_with_id(entry_id)
            .expect("duplicate entry id")
            .set_unprotected(keepass::db::fields::TITLE, "Deleted entry");

        let recycle_id = ensure_recycle_bin(&mut db);
        assert_eq!(ensure_recycle_bin(&mut db), recycle_id);
        assert_eq!(db.meta.recyclebin_enabled, Some(true));
        assert_eq!(db.meta.recyclebin_uuid, Some(recycle_id.uuid()));

        db.entry_mut(entry_id)
            .expect("entry should exist")
            .move_to(recycle_id)
            .expect("recycle bin should exist");
        let summaries = build_entry_summaries(&db);
        assert_eq!(summaries.len(), 1);
        assert!(summaries[0].trashed);
    }

    #[test]
    fn search_matches_all_non_secret_fields_without_exposing_protected_values() {
        let mut db = keepass::Database::new();
        let entry_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());
        {
            let mut root = db.root_mut();
            let mut entry = root.add_entry_with_id(entry_id).unwrap();
            entry.set_unprotected(fields::TITLE, "Example account");
            entry.set_unprotected(fields::NOTES, "Buried recovery instructions");
            entry.set_unprotected("identity.address", "42 Galaxy Way");
            entry.set_unprotected("card.holder", "Ada Lovelace");
            entry.set_unprotected("custom.Environment", "Production West");
            entry.set_protected(fields::PASSWORD, "password-secret");
            entry.set_protected("custom.API key", "custom-secret");
            entry.tags = vec!["finance".into()];
        }
        let entry = db.entry(entry_id).unwrap();

        for query in [
            "example account",
            "RECOVERY INSTRUCTIONS",
            "galaxy way",
            "ada lovelace",
            "production west",
            "environment",
            "api key",
            "finance",
        ] {
            assert!(entry_matches_search(&entry, query), "query: {query}");
        }
        assert!(!entry_matches_search(&entry, "password-secret"));
        assert!(!entry_matches_search(&entry, "custom-secret"));
    }

    #[test]
    fn custom_fields_roundtrip_and_protected_values_are_sanitized() {
        let mut db = keepass::Database::new();
        let entry_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());
        db.root_mut()
            .add_entry_with_id(entry_id)
            .expect("duplicate entry id")
            .set_unprotected("PluginData", "preserved");
        let mut patch = EntryPatch::default();
        patch.custom_fields = Some(vec![
            CustomField {
                name: "Environment".into(),
                value: "Production".into(),
                protected: false,
            },
            CustomField {
                name: "API key".into(),
                value: "secret".into(),
                protected: true,
            },
        ]);
        validate_custom_fields(&patch).unwrap();
        apply_patch(&mut db.entry_mut(entry_id).unwrap(), &patch);

        let entry = db.entry(entry_id).unwrap();
        let mapped = map_entry_to_full(&entry, false, None);
        assert_eq!(mapped.custom_fields[0].name, "API key");
        assert_eq!(mapped.custom_fields[0].value, "");
        assert!(mapped.custom_fields[0].protected);
        assert_eq!(mapped.custom_fields[1].value, "Production");
        assert!(!mapped.custom_fields[1].protected);
        assert_eq!(entry.fields["custom.API key"].get(), "secret");
        assert!(entry.fields["custom.API key"].is_protected());
        assert_eq!(entry.fields["PluginData"].get(), "preserved");
    }

    #[test]
    fn edit_payload_reads_all_secrets_in_one_projection() {
        let mut db = keepass::Database::new();
        let entry_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());
        {
            let mut root = db.root_mut();
            let mut entry = root
                .add_entry_with_id(entry_id)
                .expect("duplicate entry id");
            entry.set_protected(keepass::db::fields::PASSWORD, "password-secret");
            entry.set_protected("card.number", "4111111111111111");
            entry.set_protected("card.cvv", "123");
            entry.set_protected("card.pin", "4567");
            entry.set_protected("license.key", "license-secret");
            entry.set_protected("passport.number", "passport-secret");
            entry.set_protected("custom.API key", "custom-secret");
            entry.set_unprotected("custom.Environment", "Production");
            super::write_totp_seed(&mut entry, "otpauth://totp/Test?secret=JBSWY3DPEHPK3PXP");
        }

        let payload = build_entry_edit_payload(&db.entry(entry_id).unwrap());
        assert_eq!(payload.password, "password-secret");
        assert_eq!(payload.card_number, "4111111111111111");
        assert_eq!(payload.card_cvv, "123");
        assert_eq!(payload.card_pin, "4567");
        assert_eq!(payload.license_key, "license-secret");
        assert_eq!(payload.passport_number, "passport-secret");
        assert_eq!(payload.totp, "otpauth://totp/Test?secret=JBSWY3DPEHPK3PXP");
        assert_eq!(payload.custom_fields[0].name, "API key");
        assert_eq!(payload.custom_fields[0].value, "custom-secret");
        assert!(payload.custom_fields[0].protected);
        assert_eq!(payload.custom_fields[1].value, "Production");
    }

    #[test]
    fn entry_expiration_roundtrips_clears_and_validates() {
        let mut db = keepass::Database::new();
        let entry_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());
        db.root_mut()
            .add_entry_with_id(entry_id)
            .expect("duplicate entry id");

        let mut patch = EntryPatch::default();
        patch.expires_at = Some("2030-05-20".into());
        validate_expiration(&patch).unwrap();
        apply_patch(&mut db.entry_mut(entry_id).unwrap(), &patch);

        let entry = db.entry(entry_id).unwrap();
        assert_eq!(entry.times.expires, Some(true));
        assert_eq!(
            entry.times.expiry.unwrap().to_string(),
            "2030-05-20 23:59:59"
        );
        assert_eq!(
            map_entry_to_full(&entry, false, None).expires_at.as_deref(),
            Some("2030-05-20")
        );

        patch.expires_at = Some(String::new());
        apply_patch(&mut db.entry_mut(entry_id).unwrap(), &patch);
        let entry = db.entry(entry_id).unwrap();
        assert_eq!(entry.times.expires, Some(false));
        assert!(entry.times.expiry.is_none());
        assert!(map_entry_to_full(&entry, false, None).expires_at.is_none());

        patch.expires_at = Some("05/20/2030".into());
        assert!(validate_expiration(&patch).is_err());
    }

    #[test]
    fn full_card_entry_keeps_masked_number_subtitle() {
        let mut db = keepass::Database::new();
        let entry_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());
        {
            let mut root = db.root_mut();
            let mut entry = root
                .add_entry_with_id(entry_id)
                .expect("duplicate entry id");
            entry.set_unprotected(keepass::db::fields::TITLE, "Visa");
            entry.set_protected("card.number", "4111111111111111");
            super::set_custom_data(&mut entry, "kagi.itemType", Some("card"));
        }

        let mapped = map_entry_to_full(&db.entry(entry_id).unwrap(), false, None);
        assert_eq!(mapped.item_type, ItemType::Card);
        assert_eq!(mapped.subtitle, "4111 •••• 1111");
    }

    #[test]
    fn software_license_fields_roundtrip_and_key_is_sanitized() {
        let mut db = keepass::Database::new();
        let entry_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());
        {
            let mut root = db.root_mut();
            let mut entry = root
                .add_entry_with_id(entry_id)
                .expect("duplicate entry id");
            entry.set_unprotected(keepass::db::fields::TITLE, "Editor Pro");
            super::set_custom_data(&mut entry, "kagi.itemType", Some("software_license"));
        }

        let mut patch = EntryPatch::default();
        patch.license_version = Some("4.2".into());
        patch.license_key = Some("AAAA-BBBB".into());
        patch.license_licensed_to = Some("Ada".into());
        patch.license_registered_email = Some("ada@example.com".into());
        patch.license_purchase_date = Some("2024-01-01".into());
        apply_patch(&mut db.entry_mut(entry_id).unwrap(), &patch);

        let entry = db.entry(entry_id).unwrap();
        assert!(entry.fields["license.key"].is_protected());
        let mapped = map_entry_to_full(&entry, false, None);
        assert_eq!(mapped.item_type, ItemType::SoftwareLicense);
        let license = mapped.software_license.as_ref().unwrap();
        assert_eq!(license.version.as_deref(), Some("4.2"));
        assert!(license.has_license_key);
        assert_eq!(license.licensed_to.as_deref(), Some("Ada"));
        assert_eq!(license.purchase_date.as_deref(), Some("2024-01-01"));
    }

    #[test]
    fn passport_fields_roundtrip_and_number_is_sanitized() {
        let mut db = keepass::Database::new();
        let entry_id = keepass::db::EntryId::from_uuid(uuid::Uuid::new_v4());
        {
            let mut root = db.root_mut();
            let mut entry = root
                .add_entry_with_id(entry_id)
                .expect("duplicate entry id");
            entry.set_unprotected(keepass::db::fields::TITLE, "US Passport");
            super::set_custom_data(&mut entry, "kagi.itemType", Some("passport"));
        }

        let mut patch = EntryPatch::default();
        patch.passport_issuing_country = Some("United States".into());
        patch.passport_number = Some("123456789".into());
        patch.passport_full_name = Some("Ada Lovelace".into());
        patch.passport_birth_date = Some("1815-12-10".into());
        patch.passport_expiry_date = Some("2030-01-01".into());
        apply_patch(&mut db.entry_mut(entry_id).unwrap(), &patch);

        let entry = db.entry(entry_id).unwrap();
        assert!(entry.fields["passport.number"].is_protected());
        let mapped = map_entry_to_full(&entry, false, None);
        assert_eq!(mapped.item_type, ItemType::Passport);
        let passport = mapped.passport.as_ref().unwrap();
        assert!(passport.has_number);
        assert_eq!(passport.issuing_country.as_deref(), Some("United States"));
        assert_eq!(passport.full_name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(passport.birth_date.as_deref(), Some("1815-12-10"));
        assert_eq!(passport.expiry_date.as_deref(), Some("2030-01-01"));
    }

    #[test]
    fn custom_field_validation_rejects_duplicate_names() {
        let mut patch = EntryPatch::default();
        patch.custom_fields = Some(vec![
            CustomField {
                name: "Title".into(),
                value: "bad".into(),
                protected: false,
            },
            CustomField {
                name: "title".into(),
                value: "duplicate".into(),
                protected: false,
            },
        ]);
        assert!(validate_custom_fields(&patch).is_err());
    }

    #[test]
    fn attachment_file_name_strips_directory_components() {
        assert_eq!(
            safe_attachment_file_name("../../secrets.txt"),
            "secrets.txt"
        );
        assert_eq!(
            safe_attachment_file_name(r"..\\..\\secrets.txt"),
            "secrets.txt"
        );
        assert_eq!(safe_attachment_file_name("notes.txt"), "notes.txt");
    }

    #[test]
    fn attachment_file_name_rejects_empty_and_control_only_names() {
        for name in ["", ".", "..", "../..", "\n\r\t"] {
            assert_eq!(safe_attachment_file_name(name), "attachment");
        }
    }

    #[test]
    fn attachment_file_read_returns_name_and_bytes() {
        let dir = std::env::temp_dir().join(format!("kagi-attachment-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("upload.txt");
        std::fs::write(&path, b"sensitive attachment").unwrap();

        let (name, data) = read_attachment_file(&path).unwrap();
        assert_eq!(name, "upload.txt");
        assert_eq!(data.as_slice(), b"sensitive attachment");

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn attachment_file_write_creates_and_overwrites_regular_file() {
        let dir = std::env::temp_dir().join(format!("kagi-attachment-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("export.txt");

        write_attachment_file(&path, b"first").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"first");

        write_attachment_file(&path, b"replacement").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"replacement");

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn attachment_file_write_refuses_symlink_without_touching_target() {
        use std::os::unix::fs::symlink;

        let dir = std::env::temp_dir().join(format!("kagi-attachment-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target.txt");
        let link = dir.join("export.txt");
        std::fs::write(&target, b"original").unwrap();
        symlink(&target, &link).unwrap();

        let error = write_attachment_file(&link, b"malicious").unwrap_err();
        assert!(error.to_string().contains("symbolic link"));
        assert_eq!(std::fs::read(&target).unwrap(), b"original");

        std::fs::remove_dir_all(dir).unwrap();
    }

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
