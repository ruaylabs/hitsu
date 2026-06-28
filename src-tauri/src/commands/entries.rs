use keepass::db::{Entry as KdbxEntry, Value};
use std::fs::File;
use tauri::State;

use crate::error::{KagiError, KagiResult};
use crate::models::{
    CardFields, Entry, EntryDraft, EntryPatch, EntrySummary, IdentityFields, ItemType,
};
use crate::state::{AppState, OpenVault};

fn map_entry_to_summary(entry: &keepass::db::Entry) -> EntrySummary {
    let title = entry.get_title().unwrap_or("").to_string();
    let username = entry.get_username().unwrap_or("").to_string();
    let item_type = read_item_type(entry);
    let icon_hint = read_icon_hint(entry);
    let favorite = read_favorite(entry);

    EntrySummary {
        id: entry.uuid.to_string(),
        item_type,
        title,
        subtitle: username,
        favorite,
        icon_hint,
    }
}

fn map_entry_to_full(entry: &keepass::db::Entry) -> Entry {
    let id = entry.uuid.to_string();
    let title = entry.get_title().unwrap_or("").to_string();
    let username = entry.get_username().unwrap_or("").to_string();
    let password = entry.get_password().unwrap_or("").to_string();
    let url = entry.get_url().map(str::to_string);
    let notes = entry.get("Notes").map(str::to_string);
    let tags = entry.tags.clone();
    let item_type = read_item_type(entry);
    let icon_hint = read_icon_hint(entry);
    let favorite = read_favorite(entry);
    let totp = read_custom_data_string(entry, "otp")
        .or_else(|| entry.get_raw_otp_value().map(str::to_string));

    let now = chrono::Utc::now().to_rfc3339();

    let modified_at = entry
        .times
        .get_last_modification()
        .map(|d: &chrono::NaiveDateTime| d.and_utc().to_rfc3339())
        .unwrap_or_else(|| now.clone());

    let created_at = entry
        .times
        .get_creation()
        .map(|d: &chrono::NaiveDateTime| d.and_utc().to_rfc3339())
        .unwrap_or_else(|| now.clone());

    let identity = if item_type == ItemType::Identity {
        Some(IdentityFields {
            first_name: entry.get("identity.firstName").map(str::to_string),
            last_name: entry.get("identity.lastName").map(str::to_string),
            email: entry.get("identity.email").map(str::to_string),
            phone: entry.get("identity.phone").map(str::to_string),
            address: entry.get("identity.address").map(str::to_string),
            dob: entry.get("identity.dob").map(str::to_string),
        })
    } else {
        None
    };

    let card = if item_type == ItemType::Card {
        Some(CardFields {
            holder: entry.get("card.holder").map(str::to_string),
            number: entry.get("card.number").map(str::to_string),
            card_type: entry.get("card.type").map(str::to_string),
            exp_month: entry
                .get("card.expMonth")
                .and_then(|v: &str| v.parse().ok()),
            exp_year: entry.get("card.expYear").and_then(|v: &str| v.parse().ok()),
            cvv: entry.get("card.cvv").map(str::to_string),
            pin: entry.get("card.pin").map(str::to_string),
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
        history_count: entry
            .history
            .as_ref()
            .map_or(0, |h: &keepass::db::History| h.get_entries().len() as u32),
    }
}

fn value_to_string(v: &keepass::db::Value) -> String {
    match v {
        keepass::db::Value::Unprotected(s) => s.clone(),
        keepass::db::Value::Protected(p) => String::from_utf8_lossy(p.unsecure()).to_string(),
        keepass::db::Value::Bytes(b) => String::from_utf8_lossy(b).to_string(),
    }
}

fn read_custom_data_string(entry: &keepass::db::Entry, key: &str) -> Option<String> {
    entry
        .custom_data
        .items
        .get(key)
        .and_then(|item| item.value.as_ref())
        .map(value_to_string)
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

fn find_entry_ref<'a>(db: &'a keepass::Database, id: &str) -> Option<&'a keepass::db::Entry> {
    let uuid = uuid::Uuid::parse_str(id).ok()?;
    for node in &db.root {
        if let keepass::db::NodeRef::Entry(e) = node {
            if e.uuid == uuid {
                return Some(e);
            }
        }
    }
    None
}

fn find_entry_mut<'a>(
    db: &'a mut keepass::Database,
    id: &str,
) -> Option<&'a mut keepass::db::Entry> {
    let uuid = uuid::Uuid::parse_str(id).ok()?;
    for node in &mut db.root.children {
        if let keepass::db::Node::Entry(ref mut e) = node {
            if e.uuid == uuid {
                return Some(e);
            }
        }
    }
    None
}

fn all_entries(db: &keepass::Database) -> Vec<&keepass::db::Entry> {
    let mut entries = Vec::new();
    for node in &db.root {
        if let keepass::db::NodeRef::Entry(e) = node {
            entries.push(e);
        }
    }
    entries
}

fn remove_entry(db: &mut keepass::Database, id: &str) -> KagiResult<()> {
    let uuid = uuid::Uuid::parse_str(id).map_err(|_| KagiError::EntryNotFound(id.to_string()))?;
    let len_before = db.root.children.len();
    db.root.children.retain(|node| match node {
        keepass::db::Node::Entry(e) => e.uuid != uuid,
        _ => true,
    });
    if db.root.children.len() == len_before {
        return Err(KagiError::EntryNotFound(id.to_string()));
    }
    Ok(())
}

fn save_vault(vault: &OpenVault) -> KagiResult<()> {
    let password = String::from_utf8_lossy(&vault._master_key);
    let key = keepass::DatabaseKey::new().with_password(&password);
    let mut file = File::create(&vault._path)?;
    vault.db.save(&mut file, key)?;
    Ok(())
}

fn set_kdbx_field(entry: &mut KdbxEntry, key: &str, value: Option<&str>) {
    match value {
        Some(v) => {
            if key == "Password" || key == "card.cvv" || key == "card.pin" {
                entry
                    .fields
                    .insert(key.to_string(), Value::Protected(v.as_bytes().into()));
            } else {
                entry
                    .fields
                    .insert(key.to_string(), Value::Unprotected(v.to_string()));
            }
        }
        None => {
            entry.fields.remove(key);
        }
    }
}

fn set_custom_data(entry: &mut KdbxEntry, key: &str, value: Option<&str>) {
    match value {
        Some(v) => {
            let item = keepass::db::CustomDataItem {
                value: Some(Value::Unprotected(v.to_string())),
                last_modification_time: None,
            };
            entry.custom_data.items.insert(key.to_string(), item);
        }
        None => {
            entry.custom_data.items.remove(key);
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

    Ok(all_entries(&vault.db)
        .into_iter()
        .map(map_entry_to_summary)
        .collect())
}

#[tauri::command]
pub async fn entry_get(state: State<'_, AppState>, id: String) -> KagiResult<Entry> {
    let vaults = state
        .vaults
        .lock()
        .map_err(|e| KagiError::Custom(format!("Lock error: {}", e)))?;

    let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;

    let entry = find_entry_ref(&vault.db, &id).ok_or(KagiError::EntryNotFound(id))?;
    Ok(map_entry_to_full(entry))
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

    let id = uuid::Uuid::new_v4().to_string();
    let mut entry = KdbxEntry::new();
    entry.uuid = uuid::Uuid::parse_str(&id).unwrap();

    set_kdbx_field(&mut entry, "Title", Some(&draft.title));
    set_kdbx_field(&mut entry, "UserName", draft.username.as_deref());
    set_kdbx_field(&mut entry, "Password", draft.password.as_deref());
    set_kdbx_field(&mut entry, "URL", draft.url.as_deref());
    set_kdbx_field(&mut entry, "Notes", draft.notes.as_deref());

    if let Some(t) = &draft.totp {
        set_custom_data(&mut entry, "otp", Some(t));
    }

    set_custom_data(&mut entry, "kagi.itemType", Some(&item_type));
    set_custom_data(&mut entry, "kagi.favorite", Some("false"));

    let now = chrono::Utc::now();
    let naive = now.naive_utc();
    entry.times.set_creation(naive);
    entry.times.set_last_modification(naive);

    vault.db.root.add_child(entry);

    save_vault(vault)?;

    let item_type_enum = ItemType::from_db_value(&item_type);
    let subtitle = draft.username.clone().unwrap_or_default();
    let now_rfc = now.to_rfc3339();

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

    let entry =
        find_entry_mut(&mut vault.db, &id).ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;

    entry.update_history();
    apply_patch(entry, &patch);

    let now = chrono::Utc::now().naive_utc();
    entry.times.set_last_modification(now);

    save_vault(vault)?;

    Ok(map_entry_to_full(
        find_entry_ref(&vault.db, &id).ok_or(KagiError::EntryNotFound(id))?,
    ))
}

fn apply_patch(entry: &mut KdbxEntry, patch: &EntryPatch) {
    if let Some(ref v) = patch.title {
        set_kdbx_field(entry, "Title", Some(v));
    }
    if let Some(ref v) = patch.username {
        set_kdbx_field(entry, "UserName", Some(v));
    }
    if let Some(ref v) = patch.password {
        set_kdbx_field(entry, "Password", Some(v));
    }
    if let Some(ref v) = patch.url {
        set_kdbx_field(entry, "URL", Some(v));
    }
    if let Some(ref v) = patch.notes {
        set_kdbx_field(entry, "Notes", Some(v));
    }
    if let Some(ref v) = patch.totp {
        set_custom_data(entry, "otp", Some(v));
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
    if let Some(ref v) = patch.first_name {
        set_kdbx_field(entry, "identity.firstName", Some(v));
    }
    if let Some(ref v) = patch.last_name {
        set_kdbx_field(entry, "identity.lastName", Some(v));
    }
    if let Some(ref v) = patch.email {
        set_kdbx_field(entry, "identity.email", Some(v));
    }
    if let Some(ref v) = patch.phone {
        set_kdbx_field(entry, "identity.phone", Some(v));
    }
    if let Some(ref v) = patch.address {
        set_kdbx_field(entry, "identity.address", Some(v));
    }
    if let Some(ref v) = patch.card_holder {
        set_kdbx_field(entry, "card.holder", Some(v));
    }
    if let Some(ref v) = patch.card_number {
        set_kdbx_field(entry, "card.number", Some(v));
    }
    if let Some(ref v) = patch.card_type {
        set_kdbx_field(entry, "card.type", Some(v));
    }
    if let Some(ref v) = patch.card_exp_month {
        set_kdbx_field(entry, "card.expMonth", Some(v));
    }
    if let Some(ref v) = patch.card_exp_year {
        set_kdbx_field(entry, "card.expYear", Some(v));
    }
    if let Some(ref v) = patch.card_cvv {
        set_kdbx_field(entry, "card.cvv", Some(v));
    }
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
