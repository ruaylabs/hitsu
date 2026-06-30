use keepass::db::{fields, CustomDataItem, CustomDataValue, EntryId, Value};
use tauri::State;

use crate::error::{KagiError, KagiResult};
use crate::models::{
    CardFields, Entry, EntryDraft, EntryPatch, EntrySummary, IdentityFields, ItemType,
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

    EntrySummary {
        id: entry_ref.id().uuid().to_string(),
        item_type,
        title,
        subtitle: username,
        favorite,
        icon_hint,
    }
}

fn map_entry_to_full(entry_ref: &keepass::db::Entry) -> Entry {
    let id = entry_ref.id().uuid().to_string();
    let title = entry_ref.get_title().unwrap_or("").to_string();
    let username = entry_ref.get_username().unwrap_or("").to_string();
    let password = entry_ref.get_password().unwrap_or("").to_string();
    let url = entry_ref.get_url().map(str::to_string);
    let notes = entry_ref.get(fields::NOTES).map(str::to_string);
    let tags = entry_ref.tags.clone();
    let item_type = read_item_type(entry_ref);
    let icon_hint = read_icon_hint(entry_ref);
    let favorite = read_favorite(entry_ref);
    let totp = read_totp_seed(entry_ref);

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
        Some(CardFields {
            holder: entry_ref.get("card.holder").map(str::to_string),
            number: entry_ref.get("card.number").map(str::to_string),
            card_type: entry_ref.get("card.type").map(str::to_string),
            exp_month: entry_ref
                .get("card.expMonth")
                .and_then(|v: &str| v.parse().ok()),
            exp_year: entry_ref
                .get("card.expYear")
                .and_then(|v: &str| v.parse().ok()),
            cvv: entry_ref.get("card.cvv").map(str::to_string),
            pin: entry_ref.get("card.pin").map(str::to_string),
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
        password: Some(password),
        totp,
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
fn find_entry_ref<'a>(db: &'a keepass::Database, id: &str) -> Option<keepass::db::EntryRef<'a>> {
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

fn save_vault(vault: &OpenVault) -> KagiResult<()> {
    let password = String::from_utf8_lossy(&vault.master_key);
    let key = keepass::DatabaseKey::new().with_password(&password);
    let mut buf = std::io::Cursor::new(Vec::new());
    vault.db.save(&mut buf, key)?;
    let bytes = buf.into_inner();
    crate::vault::atomic_write(&vault.path, &bytes)?;
    Ok(())
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
    let vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    let (_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;

    Ok(vault
        .db
        .iter_all_entries()
        .map(|e| map_entry_to_summary(&e))
        .collect())
}

#[tauri::command]
pub async fn entry_get(state: State<'_, AppState>, id: String) -> KagiResult<Entry> {
    let vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;

    let entry_ref =
        find_entry_ref(&vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
    Ok(map_entry_to_full(&entry_ref))
}

#[tauri::command]
pub async fn entry_create(
    state: State<'_, AppState>,
    item_type: String,
    draft: EntryDraft,
) -> KagiResult<Entry> {
    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

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

    save_vault(vault)?;

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
        password: draft.password,
        totp: draft.totp,
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
    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

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

    save_vault(vault)?;

    let entry_ref = vault
        .db
        .entry(entry_id)
        .ok_or(KagiError::EntryNotFound(id))?;
    Ok(map_entry_to_full(&entry_ref))
}

/// Read the TOTP seed from an entry.
///
/// Tries the KeePassXC standard `otp` field first (an `otpauth://` URI),
/// then falls back to the legacy `TOTP Seed` + `TOTP Settings` fields.
fn read_totp_seed(entry: &keepass::db::Entry) -> Option<String> {
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
        seed, period, digits
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
}

#[tauri::command]
pub async fn entry_delete(state: State<'_, AppState>, id: String) -> KagiResult<()> {
    let mut vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    let (_vault_id, vault) = vaults.iter_mut().next().ok_or(KagiError::NoOpenVault)?;

    remove_entry(&mut vault.db, &id)?;
    save_vault(vault)?;
    Ok(())
}
