use tauri::State;

use crate::error::{KagiError, KagiResult};
use crate::models::{Entry, EntrySummary, ItemType};
use crate::state::AppState;

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
    let totp = entry.get_raw_otp_value().map(str::to_string);

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
        identity: None,
        card: None,
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

fn read_item_type(entry: &keepass::db::Entry) -> ItemType {
    entry
        .custom_data
        .items
        .get("kagi.itemType")
        .and_then(|item| item.value.as_ref())
        .map(|v| match v {
            keepass::db::Value::Unprotected(s) => ItemType::from_db_value(s),
            _ => ItemType::Login,
        })
        .unwrap_or(ItemType::Login)
}

fn read_icon_hint(entry: &keepass::db::Entry) -> Option<String> {
    entry
        .custom_data
        .items
        .get("kagi.iconHint")
        .and_then(|item| {
            item.value.as_ref().map(|v| match v {
                keepass::db::Value::Unprotected(s) => s.clone(),
                keepass::db::Value::Protected(p) => {
                    String::from_utf8_lossy(p.unsecure()).to_string()
                }
                keepass::db::Value::Bytes(b) => String::from_utf8_lossy(b).to_string(),
            })
        })
}

fn read_favorite(entry: &keepass::db::Entry) -> bool {
    entry
        .custom_data
        .items
        .get("kagi.favorite")
        .and_then(|item| item.value.as_ref())
        .is_some_and(|v| matches!(v, keepass::db::Value::Unprotected(s) if s == "true"))
}

fn find_entry<'a>(db: &'a keepass::Database, id: &str) -> Option<&'a keepass::db::Entry> {
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

fn all_entries(db: &keepass::Database) -> Vec<&keepass::db::Entry> {
    let mut entries = Vec::new();
    for node in &db.root {
        if let keepass::db::NodeRef::Entry(e) = node {
            entries.push(e);
        }
    }
    entries
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

    let entry = find_entry(&vault.db, &id).ok_or(KagiError::EntryNotFound(id))?;

    Ok(map_entry_to_full(entry))
}
