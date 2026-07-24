use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use base64::Engine;
use keepass::db::{fields, EntryId, Value};
use serde::Serialize;
use serde_json::Value as JsonValue;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use super::entries::build_entry_summaries;
use crate::error::{HitsuError, HitsuResult};
use crate::kdbx_fields::{set_custom_data, set_field};
use crate::models::{EntrySummary, ItemType};
use crate::state::AppState;

const MAX_1PIF_BYTES: u64 = 256 * 1024 * 1024;
const MAX_ATTACHMENT_BYTES: u64 = 100 * 1024 * 1024;
const MAX_INDEXED_FILES: usize = 20_000;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub imported_items: usize,
    pub imported_attachments: usize,
    pub skipped_items: usize,
    pub skipped_entries: Vec<SkippedImportEntry>,
    pub entries: Vec<EntrySummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedImportEntry {
    pub title: String,
    pub reason: String,
}

struct ImportedAttachment {
    name: String,
    data: Vec<u8>,
}

struct ImportedField {
    name: String,
    value: String,
    protected: bool,
}

struct ImportedItem {
    source_id: Option<String>,
    parent_id: Option<String>,
    is_file: bool,
    item_type: ItemType,
    title: String,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    notes: Option<String>,
    totp: Option<String>,
    tags: Vec<String>,
    favorite: bool,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
    dob: Option<String>,
    card_holder: Option<String>,
    card_number: Option<String>,
    card_type: Option<String>,
    card_exp_month: Option<String>,
    card_exp_year: Option<String>,
    card_cvv: Option<String>,
    card_pin: Option<String>,
    license_version: Option<String>,
    license_key: Option<String>,
    license_licensed_to: Option<String>,
    license_registered_email: Option<String>,
    license_company: Option<String>,
    license_download_page: Option<String>,
    license_publisher: Option<String>,
    license_website: Option<String>,
    license_retail_price: Option<String>,
    license_support_email: Option<String>,
    license_purchase_date: Option<String>,
    license_order_number: Option<String>,
    license_order_total: Option<String>,
    passport_type: Option<String>,
    passport_issuing_country: Option<String>,
    passport_number: Option<String>,
    passport_full_name: Option<String>,
    passport_sex: Option<String>,
    passport_nationality: Option<String>,
    passport_issuing_authority: Option<String>,
    passport_birth_date: Option<String>,
    passport_birth_place: Option<String>,
    passport_issue_date: Option<String>,
    passport_expiry_date: Option<String>,
    custom_fields: Vec<ImportedField>,
    attachments: Vec<ImportedAttachment>,
    created_at: Option<chrono::NaiveDateTime>,
    updated_at: Option<chrono::NaiveDateTime>,
    expires_at: Option<chrono::NaiveDateTime>,
}

struct ParsedImport {
    items: Vec<ImportedItem>,
    skipped_entries: Vec<SkippedImportEntry>,
}

#[derive(Clone)]
struct AttachmentSpec {
    name: Option<String>,
    locator: Option<String>,
    embedded: Option<Vec<u8>>,
}

struct ImportSource {
    data_file: PathBuf,
    attachment_roots: Vec<PathBuf>,
    recurse_first_root: bool,
}

fn import_source(path: &Path) -> HitsuResult<ImportSource> {
    if path.is_dir() {
        for name in ["data.1pif", "contents.js"] {
            let candidate = path.join(name);
            if candidate.is_file() {
                return Ok(ImportSource {
                    data_file: candidate,
                    attachment_roots: vec![path.to_path_buf()],
                    recurse_first_root: true,
                });
            }
        }
        return Err(HitsuError::Custom(
            "The selected 1PIF package has no data.1pif file".into(),
        ));
    }

    if !path.is_file() {
        return Err(HitsuError::Custom(
            "The selected 1PIF file was not found".into(),
        ));
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let mut roots = vec![parent.to_path_buf()];
    for name in [
        format!("{stem}.attachments"),
        stem.to_string(),
        "attachments".into(),
        "files".into(),
    ] {
        let candidate = parent.join(name);
        if candidate.is_dir() && !roots.contains(&candidate) {
            roots.push(candidate);
        }
    }
    Ok(ImportSource {
        data_file: path.to_path_buf(),
        attachment_roots: roots,
        recurse_first_root: false,
    })
}

fn parse_1pif(path: &Path) -> HitsuResult<ParsedImport> {
    let source = import_source(path)?;
    let size = std::fs::metadata(&source.data_file)?.len();
    if size > MAX_1PIF_BYTES {
        return Err(HitsuError::Custom(
            "The 1PIF export is too large to import".into(),
        ));
    }
    let text = std::fs::read_to_string(&source.data_file)
        .map_err(|_| HitsuError::Custom("The 1PIF export is not valid UTF-8".into()))?;

    let mut records = Vec::new();
    for line in text.lines() {
        let line = line.trim().trim_start_matches('\u{feff}');
        if !line.starts_with('{') {
            continue;
        }
        let value: JsonValue = serde_json::from_str(line)
            .map_err(|_| HitsuError::Custom("The 1PIF export contains invalid JSON".into()))?;
        records.push(value);
    }
    if records.is_empty() {
        if let Ok(value) = serde_json::from_str::<JsonValue>(text.trim()) {
            match value {
                JsonValue::Array(values) => records = values,
                JsonValue::Object(_) => records.push(value),
                _ => {}
            }
        }
    }
    if records.is_empty() {
        return Err(HitsuError::Custom(
            "No 1Password items were found in the export".into(),
        ));
    }

    let mut folders = HashMap::new();
    for record in &records {
        let type_name = string_at(record, &["typeName"]).unwrap_or_default();
        if type_name.to_ascii_lowercase().contains("folder") {
            if let (Some(id), Some(title)) =
                (string_at(record, &["uuid"]), string_at(record, &["title"]))
            {
                folders.insert(normalize_id(&id), title);
            }
        }
    }

    let files = index_attachment_files(&source)?;
    let mut items = Vec::new();
    let mut skipped_entries = Vec::new();
    for record in records {
        match parse_record(&record, &folders, &files) {
            Some(item) => items.push(item),
            None => {
                let type_name = string_at(&record, &["typeName"]).unwrap_or_default();
                let reason = if bool_at(&record, &["trashed"]).unwrap_or(false)
                    || string_at(&record, &["category"]).as_deref() == Some("099")
                {
                    "Item is in the 1Password trash"
                } else if type_name.to_ascii_lowercase().contains("folder") {
                    "Folders aren't imported"
                } else {
                    "The item couldn't be converted"
                };
                skipped_entries.push(SkippedImportEntry {
                    title: string_at(&record, &["title"])
                        .or_else(|| string_at(&record, &["overview", "title"]))
                        .unwrap_or_else(|| "Untitled item".into()),
                    reason: reason.into(),
                });
            }
        }
    }

    // 1Password can represent a legacy file as a separate record linked to
    // its owning item. Merge it into that item instead of creating a duplicate note.
    let positions: HashMap<String, usize> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| item.source_id.as_ref().map(|id| (normalize_id(id), index)))
        .collect();
    let mut remove = HashSet::new();
    for index in 0..items.len() {
        if !items[index].is_file || items[index].attachments.is_empty() {
            continue;
        }
        let Some(parent) = items[index].parent_id.as_ref().map(|id| normalize_id(id)) else {
            continue;
        };
        let Some(&parent_index) = positions.get(&parent) else {
            continue;
        };
        if parent_index == index {
            continue;
        }
        let attachments = std::mem::take(&mut items[index].attachments);
        items[parent_index].attachments.extend(attachments);
        remove.insert(index);
    }
    if !remove.is_empty() {
        skipped_entries.extend(
            items
                .iter()
                .enumerate()
                .filter(|(index, _)| remove.contains(index))
                .map(|(_, item)| SkippedImportEntry {
                    title: item.title.clone(),
                    reason: "Attachment was merged into its parent entry".into(),
                }),
        );
        items = items
            .into_iter()
            .enumerate()
            .filter_map(|(index, item)| (!remove.contains(&index)).then_some(item))
            .collect();
    }

    Ok(ParsedImport {
        items,
        skipped_entries,
    })
}

fn index_attachment_files(source: &ImportSource) -> HitsuResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut seen = HashSet::new();
    for (root_index, root) in source.attachment_roots.iter().enumerate() {
        index_directory(
            root,
            source.recurse_first_root || root_index > 0,
            &source.data_file,
            &mut files,
            &mut seen,
        )?;
    }
    Ok(files)
}

fn index_directory(
    root: &Path,
    recurse: bool,
    data_file: &Path,
    files: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
) -> HitsuResult<()> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        if files.len() >= MAX_INDEXED_FILES {
            break;
        }
        let path = entry.path();
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() && recurse {
            index_directory(&path, true, data_file, files, seen)?;
        } else if metadata.is_file()
            && path != data_file
            && metadata.len() <= MAX_ATTACHMENT_BYTES
            && seen.insert(path.clone())
        {
            files.push(path);
        }
    }
    Ok(())
}

fn parse_record(
    record: &JsonValue,
    folders: &HashMap<String, String>,
    files: &[PathBuf],
) -> Option<ImportedItem> {
    if bool_at(record, &["trashed"]).unwrap_or(false)
        || string_at(record, &["category"]).as_deref() == Some("099")
    {
        return None;
    }
    let type_name = string_at(record, &["typeName"]).unwrap_or_default();
    if type_name.to_ascii_lowercase().contains("folder") {
        return None;
    }

    let content = record
        .get("secureContents")
        .or_else(|| record.get("details"))
        .unwrap_or(&JsonValue::Null);
    let (item_type, is_file) = classify(record, content, &type_name);
    let title = string_at(record, &["title"])
        .or_else(|| string_at(record, &["overview", "title"]))
        .unwrap_or_else(|| "Untitled".into());
    let source_id = string_at(record, &["uuid"]);
    let parent_id = first_string(
        content,
        &[
            "itemUUID",
            "itemUuid",
            "parentUUID",
            "parentUuid",
            "linkedItemUUID",
        ],
    );

    let mut item = ImportedItem {
        source_id,
        parent_id,
        is_file,
        item_type,
        title: title.clone(),
        username: string_at(content, &["username"]),
        password: string_at(content, &["password"]),
        url: first_url(record, content),
        notes: string_at(content, &["notesPlain"]),
        totp: None,
        tags: tags_from(record),
        favorite: favorite_from(record),
        first_name: None,
        last_name: None,
        email: None,
        phone: None,
        address: None,
        dob: None,
        card_holder: None,
        card_number: None,
        card_type: None,
        card_exp_month: None,
        card_exp_year: None,
        card_cvv: None,
        card_pin: None,
        license_version: None,
        license_key: None,
        license_licensed_to: None,
        license_registered_email: None,
        license_company: None,
        license_download_page: None,
        license_publisher: None,
        license_website: None,
        license_retail_price: None,
        license_support_email: None,
        license_purchase_date: None,
        license_order_number: None,
        license_order_total: None,
        passport_type: None,
        passport_issuing_country: None,
        passport_number: None,
        passport_full_name: None,
        passport_sex: None,
        passport_nationality: None,
        passport_issuing_authority: None,
        passport_birth_date: None,
        passport_birth_place: None,
        passport_issue_date: None,
        passport_expiry_date: None,
        custom_fields: Vec::new(),
        attachments: Vec::new(),
        created_at: timestamp_at(record, &["createdAt"])
            .or_else(|| timestamp_at(record, &["created"])),
        updated_at: timestamp_at(record, &["updatedAt"])
            .or_else(|| timestamp_at(record, &["updated"]))
            .or_else(|| timestamp_at(record, &["txTimestamp"])),
        expires_at: timestamp_at(record, &["expiresAt"])
            .or_else(|| timestamp_at(content, &["expiresAt"])),
    };

    apply_direct_fields(&mut item, content);
    for field in collect_fields(content) {
        apply_1password_field(&mut item, field);
    }
    add_remaining_direct_fields(&mut item, content);
    remove_custom_fields_duplicating_regular_fields(&mut item);

    // A 1Password 7 package stores attachments outside data.1pif under
    // attachments/<item UUID>/ (some exports add a files/ level). These files are not referenced by the
    // item's JSON, so associate them by the item's UUID before processing
    // the less common inline/reference attachment representations.
    if let Some(source_id) = item.source_id.as_deref() {
        item.attachments
            .extend(load_packaged_attachments(source_id, files));
    }

    if let Some(folder_id) = first_string(record, &["folderUuid", "folderUUID"]).or_else(|| {
        first_string(
            record.get("openContents").unwrap_or(&JsonValue::Null),
            &["folderUuid", "folderUUID"],
        )
    }) {
        if let Some(folder) = folders.get(&normalize_id(&folder_id)) {
            if !item.tags.iter().any(|tag| tag == folder) {
                item.tags.push(folder.clone());
            }
        }
    }

    for spec in attachment_specs(record, content, &item) {
        if let Some(attachment) = load_attachment(spec, files) {
            if !item
                .attachments
                .iter()
                .any(|existing| existing.name == attachment.name)
            {
                item.attachments.push(attachment);
            }
        }
    }
    Some(item)
}

fn classify(record: &JsonValue, content: &JsonValue, type_name: &str) -> (ItemType, bool) {
    let lower = type_name.to_ascii_lowercase();
    let category = string_at(record, &["category"]).unwrap_or_default();
    if lower.contains("secure-note") || lower.contains("securenote") || category == "003" {
        (ItemType::Note, false)
    } else if lower.contains("computer.license") || category == "100" {
        (ItemType::SoftwareLicense, false)
    } else if lower.contains("government.passport") || category == "106" {
        (ItemType::Passport, false)
    } else if lower.contains("creditcard") || category == "002" {
        (ItemType::Card, false)
    } else if lower.contains("identit") || category == "004" {
        (ItemType::Identity, false)
    } else if lower.contains("password") || category == "005" {
        (ItemType::Password, false)
    } else if lower.ends_with(".file") || lower.contains("document") {
        (ItemType::Note, true)
    } else if content.get("ccnum").is_some() || content.get("cvv").is_some() {
        (ItemType::Card, false)
    } else if content.get("firstname").is_some() || content.get("address1").is_some() {
        (ItemType::Identity, false)
    } else {
        (ItemType::Login, false)
    }
}

fn apply_direct_fields(item: &mut ImportedItem, content: &JsonValue) {
    match item.item_type {
        ItemType::Card => {
            item.card_holder = first_string(content, &["cardholder", "cardholderName", "name"]);
            item.card_number = first_string(content, &["ccnum", "number"]);
            item.card_type = first_string(content, &["type", "brand"]);
            item.card_cvv = first_string(content, &["cvv", "verificationNumber"]);
            item.card_pin = first_string(content, &["pin"]);
            if let Some(expiry) = first_string(content, &["expiry", "expiration"]) {
                set_card_expiry(item, &expiry);
            }
        }
        ItemType::Identity => {
            item.first_name = first_string(content, &["firstname", "firstName"]);
            item.last_name = first_string(content, &["lastname", "lastName"]);
            item.email = first_string(content, &["email"]);
            item.phone = first_string(content, &["defphone", "phone", "homephone"]);
            item.address = direct_address(content);
            let year = first_string(content, &["birthdate_yy"]);
            let month = first_string(content, &["birthdate_mm"]);
            let day = first_string(content, &["birthdate_dd"]);
            if let (Some(year), Some(month), Some(day)) = (year, month, day) {
                item.dob = Some(format!("{year}-{:0>2}-{:0>2}", month, day));
            }
        }
        ItemType::SoftwareLicense => {
            item.license_version = first_string(content, &["product_version", "version"]);
            item.license_key = first_string(content, &["reg_code", "licenseKey"]);
            item.license_licensed_to = first_string(content, &["reg_name", "licensedTo"]);
            item.license_registered_email =
                first_string(content, &["reg_email", "registeredEmail"]);
            item.license_company = first_string(content, &["company"]);
            item.license_download_page = first_string(content, &["download_link", "downloadPage"]);
            item.license_publisher = first_string(content, &["publisher_name", "publisher"]);
            item.license_website = first_string(content, &["publisher_website", "website"]);
            item.license_retail_price = first_string(content, &["retail_price", "retailPrice"]);
            item.license_support_email = first_string(content, &["support_email", "supportEmail"]);
            item.license_purchase_date = first_string(content, &["order_date", "purchaseDate"])
                .map(|value| normalize_import_date(&value));
            item.license_order_number = first_string(content, &["order_number", "orderNumber"]);
            item.license_order_total = first_string(content, &["order_total", "orderTotal"]);
        }
        ItemType::Passport => {
            item.passport_type = first_string(content, &["type"]);
            item.passport_issuing_country =
                first_string(content, &["issuing_country", "issuingCountry"]);
            item.passport_number = first_string(content, &["number"]);
            item.passport_full_name = first_string(content, &["fullname", "fullName"]);
            item.passport_sex = first_string(content, &["sex"]);
            item.passport_nationality = first_string(content, &["nationality"]);
            item.passport_issuing_authority =
                first_string(content, &["issuing_authority", "issuingAuthority"]);
            item.passport_birth_date = first_string(content, &["birthdate", "birthDate"])
                .map(|value| normalize_import_date(&value));
            item.passport_birth_place = first_string(content, &["birthplace", "birthPlace"]);
            item.passport_issue_date = first_string(content, &["issue_date", "issueDate"])
                .map(|value| normalize_import_date(&value));
            item.passport_expiry_date = first_string(content, &["expiry_date", "expiryDate"])
                .map(|value| normalize_import_date(&value));
        }
        _ => {}
    }
}

fn collect_fields(content: &JsonValue) -> Vec<&JsonValue> {
    let mut result = Vec::new();
    if let Some(fields) = content.get("fields").and_then(JsonValue::as_array) {
        result.extend(fields);
    }
    if let Some(sections) = content.get("sections").and_then(JsonValue::as_array) {
        for section in sections {
            if let Some(fields) = section.get("fields").and_then(JsonValue::as_array) {
                result.extend(fields);
            }
        }
    }
    result
}

fn apply_1password_field(item: &mut ImportedItem, field: &JsonValue) {
    let Some(raw_value) = field.get("value").or_else(|| field.get("v")) else {
        return;
    };
    let Some(value) = field_value_to_string(raw_value, string_at(field, &["k"]).as_deref()) else {
        return;
    };
    let designation = first_string(field, &["designation", "n"]).unwrap_or_default();
    let designation_lower = designation.to_ascii_lowercase();
    let label = first_string(field, &["name", "t", "id", "n"])
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "1Password field".into());
    let protected = string_at(field, &["k"]).is_some_and(|value| value == "concealed")
        || string_at(field, &["type"]).is_some_and(|value| value == "P");

    let consumed = match item.item_type {
        ItemType::Login | ItemType::Password => {
            if designation_lower == "username" && item.username.is_none() {
                item.username = Some(value.clone());
                true
            } else if designation_lower == "password" && item.password.is_none() {
                item.password = Some(value.clone());
                true
            } else if designation.to_ascii_uppercase().starts_with("TOTP_") && item.totp.is_none() {
                item.totp = Some(totp_uri(&item.title, &value));
                true
            } else {
                false
            }
        }
        ItemType::Card => match designation_lower.as_str() {
            "ccnum" if item.card_number.is_none() => {
                item.card_number = Some(value.clone());
                true
            }
            "cvv" if item.card_cvv.is_none() => {
                item.card_cvv = Some(value.clone());
                true
            }
            "cardholder" if item.card_holder.is_none() => {
                item.card_holder = Some(value.clone());
                true
            }
            "expiry" => {
                set_card_expiry(item, &value);
                true
            }
            "type" if item.card_type.is_none() => {
                item.card_type = Some(value.clone());
                true
            }
            "pin" if item.card_pin.is_none() => {
                item.card_pin = Some(value.clone());
                true
            }
            _ => false,
        },
        ItemType::Identity => match designation_lower.as_str() {
            "firstname" if item.first_name.is_none() => {
                item.first_name = Some(value.clone());
                true
            }
            "lastname" if item.last_name.is_none() => {
                item.last_name = Some(value.clone());
                true
            }
            "email" if item.email.is_none() => {
                item.email = Some(value.clone());
                true
            }
            "defphone" if item.phone.is_none() => {
                item.phone = Some(value.clone());
                true
            }
            "address" if item.address.is_none() => {
                item.address = Some(value.clone());
                true
            }
            "birthdate" if item.dob.is_none() => {
                item.dob = Some(value.clone());
                true
            }
            "username" if item.username.is_none() => {
                item.username = Some(value.clone());
                true
            }
            _ => false,
        },
        ItemType::SoftwareLicense => match designation_lower.as_str() {
            "product_version" | "version" if item.license_version.is_none() => {
                item.license_version = Some(value.clone());
                true
            }
            "reg_code" | "license_key" if item.license_key.is_none() => {
                item.license_key = Some(value.clone());
                true
            }
            "reg_name" | "licensed_to" if item.license_licensed_to.is_none() => {
                item.license_licensed_to = Some(value.clone());
                true
            }
            "reg_email" | "registered_email" if item.license_registered_email.is_none() => {
                item.license_registered_email = Some(value.clone());
                true
            }
            "company" if item.license_company.is_none() => {
                item.license_company = Some(value.clone());
                true
            }
            "download_link" | "download_page" if item.license_download_page.is_none() => {
                item.license_download_page = Some(value.clone());
                true
            }
            "publisher_name" | "publisher" if item.license_publisher.is_none() => {
                item.license_publisher = Some(value.clone());
                true
            }
            "publisher_website" | "website" if item.license_website.is_none() => {
                item.license_website = Some(value.clone());
                true
            }
            "retail_price" if item.license_retail_price.is_none() => {
                item.license_retail_price = Some(value.clone());
                true
            }
            "support_email" if item.license_support_email.is_none() => {
                item.license_support_email = Some(value.clone());
                true
            }
            "order_date" | "purchase_date" if item.license_purchase_date.is_none() => {
                item.license_purchase_date = Some(normalize_import_date(&value));
                true
            }
            "order_number" if item.license_order_number.is_none() => {
                item.license_order_number = Some(value.clone());
                true
            }
            "order_total" if item.license_order_total.is_none() => {
                item.license_order_total = Some(value.clone());
                true
            }
            _ => false,
        },
        ItemType::Passport => match designation_lower.as_str() {
            "type" if item.passport_type.is_none() => {
                item.passport_type = Some(value.clone());
                true
            }
            "issuing_country" if item.passport_issuing_country.is_none() => {
                item.passport_issuing_country = Some(value.clone());
                true
            }
            "number" if item.passport_number.is_none() => {
                item.passport_number = Some(value.clone());
                true
            }
            "fullname" | "full_name" if item.passport_full_name.is_none() => {
                item.passport_full_name = Some(value.clone());
                true
            }
            "sex" if item.passport_sex.is_none() => {
                item.passport_sex = Some(value.clone());
                true
            }
            "nationality" if item.passport_nationality.is_none() => {
                item.passport_nationality = Some(value.clone());
                true
            }
            "issuing_authority" if item.passport_issuing_authority.is_none() => {
                item.passport_issuing_authority = Some(value.clone());
                true
            }
            "birthdate" | "birth_date" if item.passport_birth_date.is_none() => {
                item.passport_birth_date = Some(normalize_import_date(&value));
                true
            }
            "birthplace" | "birth_place" if item.passport_birth_place.is_none() => {
                item.passport_birth_place = Some(value.clone());
                true
            }
            "issue_date" if item.passport_issue_date.is_none() => {
                item.passport_issue_date = Some(normalize_import_date(&value));
                true
            }
            "expiry_date" if item.passport_expiry_date.is_none() => {
                item.passport_expiry_date = Some(normalize_import_date(&value));
                true
            }
            _ => false,
        },
        ItemType::Note => false,
    };
    // IDs containing `;opid=__` are 1Password's captured HTML form-control
    // metadata, not user-created custom fields. Keep designated standard
    // fields above, but don't turn the remaining autofill noise into Hitsu fields.
    let is_internal_form_field =
        string_at(field, &["id"]).is_some_and(|id| id.to_ascii_lowercase().contains(";opid=__"));
    if !consumed && !is_internal_form_field {
        push_custom(item, label, value, protected);
    }
}

fn add_remaining_direct_fields(item: &mut ImportedItem, content: &JsonValue) {
    let Some(object) = content.as_object() else {
        return;
    };
    const OMIT: &[&str] = &[
        "fields",
        "sections",
        "URLs",
        "notesPlain",
        "password",
        "passwordHistory",
        "attachments",
        "documentAttributes",
        "itemUUID",
        "itemUuid",
        "parentUUID",
        "parentUuid",
        "linkedItemUUID",
        "username",
        "ccnum",
        "number",
        "cvv",
        "pin",
        "cardholder",
        "cardholderName",
        "type",
        "brand",
        "expiry",
        "expiration",
        "firstname",
        "firstName",
        "lastname",
        "lastName",
        "email",
        "defphone",
        "phone",
        "address",
        "address1",
        "address2",
        "city",
        "state",
        "zip",
        "country",
        "birthdate_yy",
        "birthdate_mm",
        "birthdate_dd",
        "product_version",
        "version",
        "reg_code",
        "licenseKey",
        "reg_name",
        "licensedTo",
        "reg_email",
        "registeredEmail",
        "download_link",
        "downloadPage",
        "publisher_name",
        "publisher",
        "publisher_website",
        "website",
        "retail_price",
        "retailPrice",
        "support_email",
        "supportEmail",
        "order_date",
        "purchaseDate",
        "order_number",
        "orderNumber",
        "order_total",
        "orderTotal",
        "issuing_country",
        "issuingCountry",
        "number",
        "fullname",
        "fullName",
        "sex",
        "nationality",
        "issuing_authority",
        "issuingAuthority",
        "birthdate",
        "birthDate",
        "birthplace",
        "birthPlace",
        "issue_date",
        "issueDate",
        "expiry_date",
        "expiryDate",
    ];
    for (name, raw) in object {
        if OMIT.contains(&name.as_str()) || raw.is_null() || raw.is_array() || raw.is_object() {
            continue;
        }
        if let Some(value) = json_scalar(raw) {
            push_custom(item, humanize(name), value, false);
        }
    }
}

fn push_custom(item: &mut ImportedItem, name: String, value: String, protected: bool) {
    let normalized_name = name
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    if normalized_name == "htmlmethod"
        || value.trim().is_empty()
        || item
            .custom_fields
            .iter()
            .any(|field| field.name.eq_ignore_ascii_case(name.trim()) && field.value == value)
    {
        return;
    }
    item.custom_fields.push(ImportedField {
        name: name.trim().to_string(),
        value,
        protected,
    });
}

fn remove_custom_fields_duplicating_regular_fields(item: &mut ImportedItem) {
    let regular_values = [
        Some(item.title.as_str()),
        item.username.as_deref(),
        item.password.as_deref(),
        item.url.as_deref(),
        item.notes.as_deref(),
        item.totp.as_deref(),
        item.first_name.as_deref(),
        item.last_name.as_deref(),
        item.email.as_deref(),
        item.phone.as_deref(),
        item.address.as_deref(),
        item.dob.as_deref(),
        item.card_holder.as_deref(),
        item.card_number.as_deref(),
        item.card_type.as_deref(),
        item.card_exp_month.as_deref(),
        item.card_exp_year.as_deref(),
        item.card_cvv.as_deref(),
        item.card_pin.as_deref(),
        item.license_version.as_deref(),
        item.license_key.as_deref(),
        item.license_licensed_to.as_deref(),
        item.license_registered_email.as_deref(),
        item.license_company.as_deref(),
        item.license_download_page.as_deref(),
        item.license_publisher.as_deref(),
        item.license_website.as_deref(),
        item.license_retail_price.as_deref(),
        item.license_support_email.as_deref(),
        item.license_purchase_date.as_deref(),
        item.license_order_number.as_deref(),
        item.license_order_total.as_deref(),
        item.passport_type.as_deref(),
        item.passport_issuing_country.as_deref(),
        item.passport_number.as_deref(),
        item.passport_full_name.as_deref(),
        item.passport_sex.as_deref(),
        item.passport_nationality.as_deref(),
        item.passport_issuing_authority.as_deref(),
        item.passport_birth_date.as_deref(),
        item.passport_birth_place.as_deref(),
        item.passport_issue_date.as_deref(),
        item.passport_expiry_date.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(str::to_string)
    .collect::<HashSet<_>>();

    item.custom_fields
        .retain(|field| !regular_values.contains(field.value.trim()));
}

fn load_packaged_attachments(source_id: &str, files: &[PathBuf]) -> Vec<ImportedAttachment> {
    let source_id = normalize_id(source_id);
    files
        .iter()
        .filter(|path| attachment_belongs_to_item(path, &source_id))
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?.to_string();
            let data = std::fs::read(path).ok()?;
            Some(ImportedAttachment { name, data })
        })
        .collect()
}

fn attachment_belongs_to_item(path: &Path, source_id: &str) -> bool {
    let components = path
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    components.windows(2).any(|parts| {
        parts[0].eq_ignore_ascii_case("attachments") && normalize_id(parts[1]) == source_id
    })
}

fn attachment_specs(
    record: &JsonValue,
    content: &JsonValue,
    item: &ImportedItem,
) -> Vec<AttachmentSpec> {
    let mut specs = Vec::new();
    for owner in [record, content] {
        if let Some(values) = owner.get("attachments").and_then(JsonValue::as_array) {
            for value in values {
                if let Some(spec) = attachment_spec(value) {
                    specs.push(spec);
                }
            }
        }
    }
    if let Some(attributes) = content.get("documentAttributes") {
        if let Some(spec) = attachment_spec(attributes) {
            specs.push(spec);
        }
    }
    if item.is_file && specs.is_empty() {
        specs.push(AttachmentSpec {
            name: first_string(content, &["fileName", "filename", "name"])
                .or_else(|| Some(item.title.clone())),
            locator: first_string(content, &["documentId", "fileId", "uuid"])
                .or_else(|| item.source_id.clone()),
            embedded: None,
        });
    }
    specs
}

fn attachment_spec(value: &JsonValue) -> Option<AttachmentSpec> {
    let name = first_string(value, &["fileName", "filename", "name", "title"]);
    let locator = first_string(value, &["documentId", "fileId", "uuid", "id", "path"]);
    let embedded = first_string(value, &["dataBase64", "base64", "data"]).and_then(|data| {
        let encoded = data
            .split_once(",")
            .map_or(data.as_str(), |(_, value)| value);
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .filter(|bytes| bytes.len() as u64 <= MAX_ATTACHMENT_BYTES)
    });
    (name.is_some() || locator.is_some() || embedded.is_some()).then_some(AttachmentSpec {
        name,
        locator,
        embedded,
    })
}

fn load_attachment(spec: AttachmentSpec, files: &[PathBuf]) -> Option<ImportedAttachment> {
    let data = if let Some(data) = spec.embedded {
        data
    } else {
        let candidates = [spec.locator.as_deref(), spec.name.as_deref()];
        let path = files.iter().find(|path| {
            let file_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            candidates.into_iter().flatten().any(|candidate| {
                let candidate = Path::new(candidate)
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or(candidate);
                file_name.eq_ignore_ascii_case(candidate)
            })
        })?;
        std::fs::read(path).ok()?
    };
    let fallback = spec.locator.as_deref().unwrap_or("attachment");
    let name = spec
        .name
        .as_deref()
        .and_then(|name| Path::new(name).file_name()?.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(fallback)
        .to_string();
    Some(ImportedAttachment { name, data })
}

fn apply_import(
    db: &mut keepass::Database,
    parsed: ParsedImport,
) -> (usize, usize, Vec<SkippedImportEntry>) {
    let mut imported_items = 0;
    let mut imported_attachments = 0;
    let mut skipped_entries = parsed.skipped_entries;
    for item in parsed.items {
        let preferred_id = item
            .source_id
            .as_deref()
            .and_then(parse_1password_uuid)
            .map(EntryId::from_uuid)
            .filter(|id| db.entry(*id).is_none());
        let entry_id = preferred_id.unwrap_or_else(|| EntryId::from_uuid(uuid::Uuid::new_v4()));
        let mut root = db.root_mut();
        let Ok(mut entry) = root.add_entry_with_id(entry_id) else {
            skipped_entries.push(SkippedImportEntry {
                title: item.title,
                reason: "Hitsu couldn't create this entry".into(),
            });
            continue;
        };
        entry.set_unprotected(fields::TITLE, &item.title);
        set_field(
            &mut entry,
            fields::USERNAME,
            item.username.as_deref(),
            false,
        );
        set_field(&mut entry, fields::PASSWORD, item.password.as_deref(), true);
        set_field(&mut entry, fields::URL, item.url.as_deref(), false);
        set_field(&mut entry, fields::NOTES, item.notes.as_deref(), false);
        set_field(&mut entry, fields::OTP, item.totp.as_deref(), false);
        set_field(
            &mut entry,
            "identity.firstName",
            item.first_name.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "identity.lastName",
            item.last_name.as_deref(),
            false,
        );
        set_field(&mut entry, "identity.email", item.email.as_deref(), false);
        set_field(&mut entry, "identity.phone", item.phone.as_deref(), false);
        set_field(
            &mut entry,
            "identity.address",
            item.address.as_deref(),
            false,
        );
        set_field(&mut entry, "identity.dob", item.dob.as_deref(), false);
        set_field(
            &mut entry,
            "card.holder",
            item.card_holder.as_deref(),
            false,
        );
        set_field(&mut entry, "card.number", item.card_number.as_deref(), true);
        set_field(&mut entry, "card.type", item.card_type.as_deref(), false);
        set_field(
            &mut entry,
            "card.expMonth",
            item.card_exp_month.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "card.expYear",
            item.card_exp_year.as_deref(),
            false,
        );
        set_field(&mut entry, "card.cvv", item.card_cvv.as_deref(), true);
        set_field(&mut entry, "card.pin", item.card_pin.as_deref(), true);
        set_field(
            &mut entry,
            "license.version",
            item.license_version.as_deref(),
            false,
        );
        set_field(&mut entry, "license.key", item.license_key.as_deref(), true);
        set_field(
            &mut entry,
            "license.licensedTo",
            item.license_licensed_to.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "license.registeredEmail",
            item.license_registered_email.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "license.company",
            item.license_company.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "license.downloadPage",
            item.license_download_page.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "license.publisher",
            item.license_publisher.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "license.website",
            item.license_website.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "license.retailPrice",
            item.license_retail_price.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "license.supportEmail",
            item.license_support_email.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "license.purchaseDate",
            item.license_purchase_date.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "license.orderNumber",
            item.license_order_number.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "license.orderTotal",
            item.license_order_total.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "passport.type",
            item.passport_type.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "passport.issuingCountry",
            item.passport_issuing_country.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "passport.number",
            item.passport_number.as_deref(),
            true,
        );
        set_field(
            &mut entry,
            "passport.fullName",
            item.passport_full_name.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "passport.sex",
            item.passport_sex.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "passport.nationality",
            item.passport_nationality.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "passport.issuingAuthority",
            item.passport_issuing_authority.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "passport.birthDate",
            item.passport_birth_date.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "passport.birthPlace",
            item.passport_birth_place.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "passport.issueDate",
            item.passport_issue_date.as_deref(),
            false,
        );
        set_field(
            &mut entry,
            "passport.expiryDate",
            item.passport_expiry_date.as_deref(),
            false,
        );
        entry.tags = item.tags;
        set_custom_data(
            &mut entry,
            "hitsu.itemType",
            Some(item_type_name(&item.item_type)),
        );
        set_custom_data(
            &mut entry,
            "hitsu.favorite",
            Some(if item.favorite { "true" } else { "false" }),
        );
        let mut names = HashSet::new();
        for field in item.custom_fields {
            let base = if field.name.trim().is_empty() {
                "1Password field"
            } else {
                field.name.trim()
            };
            let mut name = base.to_string();
            let mut suffix = 2;
            while !names.insert(name.to_ascii_lowercase()) {
                name = format!("{base} {suffix}");
                suffix += 1;
            }
            set_field(
                &mut entry,
                &format!("custom.{name}"),
                Some(&field.value),
                field.protected,
            );
        }
        for attachment in item.attachments {
            entry.add_attachment(attachment.name, Value::unprotected(attachment.data));
            imported_attachments += 1;
        }
        let now = chrono::Utc::now().naive_utc();
        entry.times.creation = Some(item.created_at.unwrap_or(now));
        entry.times.last_modification = Some(item.updated_at.unwrap_or(now));
        if let Some(expires_at) = item.expires_at {
            entry.times.expires = Some(true);
            entry.times.expiry = Some(expires_at);
        }
        imported_items += 1;
    }
    (imported_items, imported_attachments, skipped_entries)
}

fn item_type_name(item_type: &ItemType) -> &'static str {
    match item_type {
        ItemType::Login => "login",
        ItemType::Password => "password",
        ItemType::Note => "note",
        ItemType::Identity => "identity",
        ItemType::Card => "card",
        ItemType::SoftwareLicense => "software_license",
        ItemType::Passport => "passport",
    }
}

#[tauri::command]
pub async fn vault_import_1pif(
    app: AppHandle,
    state: State<'_, AppState>,
) -> HitsuResult<Option<ImportReport>> {
    // The source path stays in native Rust; the webview only starts the operation.
    let Some(selected) = app
        .dialog()
        .file()
        .add_filter("1Password Interchange Format", &["1pif"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = selected
        .into_path()
        .map_err(|_| HitsuError::Custom("The selected source is not a local file".into()))?;
    let parsed = tauri::async_runtime::spawn_blocking(move || parse_1pif(&path))
        .await
        .map_err(HitsuError::from_join)??;

    let _save_guard = state.save_lock.lock().await;
    let (mut db, key, vault_path, expected_disk_hash) = {
        let vaults = state.vaults.lock();
        let (_, vault) = vaults.iter().next().ok_or(HitsuError::NoOpenVault)?;
        (
            vault.db.clone(),
            vault.db_key.clone(),
            vault.path.clone(),
            vault.disk_hash,
        )
    };
    let (imported_items, imported_attachments, skipped_entries) = apply_import(&mut db, parsed);
    let skipped_items = skipped_entries.len();
    let entries = build_entry_summaries(&db);

    let save_db = db.clone();
    let save_key = key.clone();
    let save_path = vault_path.clone();
    let new_disk_hash = tauri::async_runtime::spawn_blocking(move || -> HitsuResult<[u8; 32]> {
        crate::vault::ensure_unmodified(&save_path, &expected_disk_hash)?;
        let mut buffer = std::io::Cursor::new(Vec::new());
        save_db.save(&mut buffer, save_key)?;
        let bytes = buffer.into_inner();
        crate::vault::atomic_write(&save_path, &bytes)?;
        Ok(crate::vault::sha256_bytes(&bytes))
    })
    .await
    .map_err(HitsuError::from_join)??;

    let mut vaults = state.vaults.lock();
    let (_, vault) = vaults.iter_mut().next().ok_or(HitsuError::NoOpenVault)?;
    if vault.path != vault_path {
        return Err(HitsuError::Custom(
            "The open vault changed during import".into(),
        ));
    }
    vault.db = db;
    vault.disk_hash = new_disk_hash;

    Ok(Some(ImportReport {
        imported_items,
        imported_attachments,
        skipped_items,
        skipped_entries,
        entries,
    }))
}

fn string_at(value: &JsonValue, path: &[&str]) -> Option<String> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    json_scalar(current)
}

fn first_string(value: &JsonValue, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| string_at(value, &[*key]))
}

fn json_scalar(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(value) if !value.trim().is_empty() => Some(value.clone()),
        JsonValue::Number(value) => Some(value.to_string()),
        JsonValue::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn bool_at(value: &JsonValue, path: &[&str]) -> Option<bool> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_bool()
}

fn timestamp_at(value: &JsonValue, path: &[&str]) -> Option<chrono::NaiveDateTime> {
    let raw = string_at(value, path)?;
    if let Ok(timestamp) = raw.parse::<i64>() {
        let seconds = if timestamp > 10_000_000_000 {
            timestamp / 1000
        } else {
            timestamp
        };
        return chrono::DateTime::from_timestamp(seconds, 0).map(|date| date.naive_utc());
    }
    chrono::DateTime::parse_from_rfc3339(&raw)
        .map(|date| date.naive_utc())
        .ok()
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(&raw, "%Y-%m-%d")
                .ok()?
                .and_hms_opt(23, 59, 59)
        })
}

fn normalize_import_date(value: &str) -> String {
    value
        .parse::<i64>()
        .ok()
        .and_then(|timestamp| {
            let seconds = if timestamp > 10_000_000_000 {
                timestamp / 1000
            } else {
                timestamp
            };
            chrono::DateTime::from_timestamp(seconds, 0)
        })
        .map_or_else(|| value.to_string(), |date| date.date_naive().to_string())
}

fn first_url(record: &JsonValue, content: &JsonValue) -> Option<String> {
    for owner in [
        content,
        record.get("overview").unwrap_or(&JsonValue::Null),
        record,
    ] {
        if let Some(urls) = owner.get("URLs").and_then(JsonValue::as_array) {
            if let Some(url) = urls.iter().find_map(|url| first_string(url, &["url", "u"])) {
                return Some(url);
            }
        }
    }
    string_at(record, &["location"]).or_else(|| string_at(record, &["overview", "url"]))
}

fn tags_from(record: &JsonValue) -> Vec<String> {
    let mut tags = Vec::new();
    for owner in [
        record.get("openContents"),
        record.get("overview"),
        Some(record),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(values) = owner.get("tags") {
            match values {
                JsonValue::Array(values) => tags.extend(values.iter().filter_map(json_scalar)),
                JsonValue::String(value) => tags.extend(
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string),
                ),
                _ => {}
            }
        }
    }
    tags.sort();
    tags.dedup();
    tags
}

fn favorite_from(record: &JsonValue) -> bool {
    bool_at(record, &["favorite"]).unwrap_or(false)
        || bool_at(record, &["openContents", "favorite"]).unwrap_or(false)
        || string_at(record, &["openContents", "faveIndex"])
            .and_then(|value| value.parse::<i64>().ok())
            .is_some_and(|value| value > 0)
}

fn field_value_to_string(value: &JsonValue, kind: Option<&str>) -> Option<String> {
    if kind == Some("date") {
        if let Some(seconds) = value.as_i64() {
            return chrono::DateTime::from_timestamp(seconds, 0)
                .map(|date| date.date_naive().to_string());
        }
    }
    if let Some(object) = value.as_object() {
        let parts = ["street", "city", "state", "zip", "country"]
            .iter()
            .filter_map(|key| object.get(*key).and_then(json_scalar))
            .collect::<Vec<_>>();
        return (!parts.is_empty()).then(|| parts.join(", "));
    }
    json_scalar(value)
}

fn direct_address(content: &JsonValue) -> Option<String> {
    let parts = ["address1", "address2", "city", "state", "zip", "country"]
        .iter()
        .filter_map(|key| string_at(content, &[*key]))
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join(", "))
}

fn set_card_expiry(item: &mut ImportedItem, expiry: &str) {
    let digits: String = expiry.chars().filter(char::is_ascii_digit).collect();
    let (year, month) = if digits.len() == 6 && digits.starts_with("20") {
        (&digits[..4], &digits[4..])
    } else if matches!(digits.len(), 4 | 6) {
        (&digits[2..], &digits[..2])
    } else {
        return;
    };
    item.card_exp_month = Some(month.trim_start_matches('0').to_string());
    item.card_exp_year = Some(if year.len() == 2 {
        format!("20{year}")
    } else {
        year.to_string()
    });
}

fn totp_uri(title: &str, value: &str) -> String {
    if value.starts_with("otpauth://") {
        return value.to_string();
    }
    format!(
        "otpauth://totp/{}?secret={}",
        percent_encoding::utf8_percent_encode(title, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(value, percent_encoding::NON_ALPHANUMERIC)
    )
}

fn parse_1password_uuid(value: &str) -> Option<uuid::Uuid> {
    uuid::Uuid::parse_str(value).ok().or_else(|| {
        (value.len() == 32)
            .then(|| {
                format!(
                    "{}-{}-{}-{}-{}",
                    &value[..8],
                    &value[8..12],
                    &value[12..16],
                    &value[16..20],
                    &value[20..]
                )
            })?
            .parse()
            .ok()
    })
}

fn normalize_id(value: &str) -> String {
    value
        .chars()
        .filter(|character| *character != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn humanize(value: &str) -> String {
    let mut result = String::new();
    for character in value.chars() {
        if character.is_ascii_uppercase() && !result.is_empty() {
            result.push(' ');
        }
        result.push(if character == '_' { ' ' } else { character });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_export(lines: &[JsonValue], attachment: Option<(&str, &[u8])>) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hitsu-1pif-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("attachments")).unwrap();
        let text = lines
            .iter()
            .map(JsonValue::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let path = dir.join("export.1pif");
        std::fs::write(&path, text).unwrap();
        if let Some((name, bytes)) = attachment {
            std::fs::write(dir.join("attachments").join(name), bytes).unwrap();
        }
        path
    }

    #[test]
    fn parses_all_supported_types_and_attachments() {
        let values = vec![
            serde_json::json!({"uuid":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA","title":"Login","typeName":"webforms.WebForm","location":"https://example.com","expiresAt":1893456000,"secureContents":{"fields":[{"designation":"username","name":"email","value":"me@example.com"},{"designation":"password","name":"password","type":"P","value":"secret"},{"designation":"TOTP_1","name":"one-time password","value":"ABC123"}],"attachments":[{"fileName":"proof.txt","documentId":"DOC1"}]}}),
            serde_json::json!({"title":"Password","typeName":"passwords.Password","secureContents":{"password":"generated"}}),
            serde_json::json!({"title":"Note","typeName":"securenotes.SecureNote","secureContents":{"notesPlain":"hello"}}),
            serde_json::json!({"title":"Person","typeName":"identities.Identity","secureContents":{"firstname":"Ada","lastname":"Lovelace","email":"ada@example.com","address1":"1 Main St","city":"London"}}),
            serde_json::json!({"title":"Card","typeName":"wallet.financial.CreditCard","secureContents":{"ccnum":"4111111111111111","cvv":"123","cardholder":"Ada","expiry":"203012"}}),
            serde_json::json!({"title":"Editor Pro","typeName":"wallet.computer.License","secureContents":{"order_date":1704067200,"sections":[{"fields":[{"n":"product_version","t":"version","v":"4.2"},{"n":"reg_code","t":"license key","k":"concealed","v":"AAAA-BBBB"}]},{"name":"customer","fields":[{"n":"reg_name","t":"licensed to","v":"Ada"},{"n":"reg_email","t":"registered email","v":"ada@example.com"}]},{"name":"order","fields":[{"n":"order_date","t":"purchase date","k":"date","v":1704067200},{"n":"order_number","t":"order number","v":"ORDER-1"}]}]}}),
            serde_json::json!({"title":"US Passport","typeName":"wallet.government.Passport","secureContents":{"sections":[{"fields":[{"n":"type","t":"type","v":"Passport"},{"n":"issuing_country","t":"issuing country","v":"United States"},{"n":"number","t":"number","v":"123456789"},{"n":"fullname","t":"full name","v":"Ada Lovelace"},{"n":"sex","t":"sex","v":"female"},{"n":"nationality","t":"nationality","v":"American"},{"n":"issuing_authority","t":"issuing authority","v":"Department of State"},{"n":"birthdate","t":"date of birth","k":"date","v":-4865616000_i64},{"n":"birthplace","t":"place of birth","v":"London"},{"n":"issue_date","t":"issued on","k":"date","v":1704067200},{"n":"expiry_date","t":"expiry date","k":"date","v":2019686400}]}]}}),
        ];
        let path = write_export(&values, Some(("DOC1", b"attachment")));
        let parsed = parse_1pif(&path).unwrap();
        assert_eq!(parsed.items.len(), 7);
        assert_eq!(parsed.items[0].item_type, ItemType::Login);
        assert_eq!(parsed.items[0].username.as_deref(), Some("me@example.com"));
        assert!(parsed.items[0]
            .totp
            .as_deref()
            .unwrap()
            .starts_with("otpauth://"));
        assert_eq!(parsed.items[0].attachments[0].data, b"attachment");
        assert_eq!(
            parsed.items[0].expires_at.unwrap().date().to_string(),
            "2030-01-01"
        );
        assert_eq!(parsed.items[1].item_type, ItemType::Password);
        assert_eq!(parsed.items[2].item_type, ItemType::Note);
        assert_eq!(parsed.items[3].item_type, ItemType::Identity);
        assert_eq!(parsed.items[3].first_name.as_deref(), Some("Ada"));
        assert_eq!(parsed.items[4].item_type, ItemType::Card);
        assert_eq!(parsed.items[4].card_exp_year.as_deref(), Some("2030"));
        assert_eq!(parsed.items[4].card_exp_month.as_deref(), Some("12"));
        assert_eq!(parsed.items[5].item_type, ItemType::SoftwareLicense);
        assert_eq!(parsed.items[5].license_version.as_deref(), Some("4.2"));
        assert_eq!(parsed.items[5].license_key.as_deref(), Some("AAAA-BBBB"));
        assert_eq!(parsed.items[5].license_licensed_to.as_deref(), Some("Ada"));
        assert_eq!(
            parsed.items[5].license_purchase_date.as_deref(),
            Some("2024-01-01")
        );
        assert!(parsed.items[5].custom_fields.is_empty());
        assert_eq!(parsed.items[6].item_type, ItemType::Passport);
        assert_eq!(
            parsed.items[6].passport_issuing_country.as_deref(),
            Some("United States")
        );
        assert_eq!(
            parsed.items[6].passport_number.as_deref(),
            Some("123456789")
        );
        assert_eq!(
            parsed.items[6].passport_full_name.as_deref(),
            Some("Ada Lovelace")
        );
        assert_eq!(
            parsed.items[6].passport_issue_date.as_deref(),
            Some("2024-01-01")
        );
        assert_eq!(
            parsed.items[6].passport_expiry_date.as_deref(),
            Some("2034-01-01")
        );
        assert!(parsed.items[6].custom_fields.is_empty());
    }

    #[test]
    fn imports_files_from_the_1password_package_attachment_layout() {
        let item_id = "7747B90C52F9405181DBE9868BDA9525";
        let dir = std::env::temp_dir().join(format!("hitsu-1pif-{}", uuid::Uuid::new_v4()));
        let files_dir = dir.join("attachments").join(item_id);
        std::fs::create_dir_all(&files_dir).unwrap();
        std::fs::write(files_dir.join("proof.txt"), b"package attachment").unwrap();
        let path = dir.join("data.1pif");
        std::fs::write(
            &path,
            serde_json::json!({"uuid":item_id,"title":"Only attachments","typeName":"webforms.WebForm","secureContents":{}}).to_string(),
        )
        .unwrap();

        let parsed = parse_1pif(&path).unwrap();
        assert_eq!(parsed.items[0].attachments.len(), 1);
        assert_eq!(parsed.items[0].attachments[0].name, "proof.txt");
        assert_eq!(parsed.items[0].attachments[0].data, b"package attachment");
    }

    #[test]
    fn applies_import_with_protected_secrets_without_duplicate_custom_values() {
        let path = write_export(
            &[
                serde_json::json!({"title":"Login","typeName":"webforms.WebForm","expiresAt":"2030-01-01","secureContents":{"password":"secret","htmlMethod":"POST","fields":[{"designation":"username","id":"username;opid=__1","value":"alice"},{"designation":"password","name":"password","value":"secret","type":"P"},{"id":"login-btn;opid=__3","value":"Log in","type":"I"}],"sections":[{"fields":[{"t":"html Method","v":"POST"},{"t":"duplicate password","v":"secret","k":"concealed"},{"t":"recovery code","v":"hidden","k":"concealed"}]}]}}),
            ],
            None,
        );
        let parsed = parse_1pif(&path).unwrap();
        assert_eq!(parsed.items[0].username.as_deref(), Some("alice"));
        assert_eq!(parsed.items[0].custom_fields.len(), 1);
        assert_eq!(parsed.items[0].custom_fields[0].name, "recovery code");

        let mut db = keepass::Database::new();
        let (items, _, _) = apply_import(&mut db, parsed);
        assert_eq!(items, 1);
        let entry = db.iter_all_entries().next().unwrap();
        assert!(entry.fields.get(fields::PASSWORD).unwrap().is_protected());
        assert!(entry
            .fields
            .get("custom.recovery code")
            .unwrap()
            .is_protected());
        assert!(!entry.fields.contains_key("custom.duplicate password"));
        assert_eq!(entry.times.expires, Some(true));
        assert_eq!(
            entry.times.expiry.unwrap().to_string(),
            "2030-01-01 23:59:59"
        );
    }

    #[test]
    fn ignores_headers_folders_and_trashed_items() {
        let dir = std::env::temp_dir().join(format!("hitsu-1pif-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("export.1pif");
        std::fs::write(
            &path,
            "***header***\n{\"uuid\":\"folder\",\"title\":\"Work\",\"typeName\":\"system.folder.Regular\"}\n{\"title\":\"Deleted\",\"trashed\":true}\n{\"title\":\"Kept\",\"typeName\":\"securenotes.SecureNote\"}\n",
        )
        .unwrap();
        let parsed = parse_1pif(&path).unwrap();
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.skipped_entries.len(), 2);
        assert_eq!(parsed.skipped_entries[0].title, "Work");
        assert_eq!(parsed.skipped_entries[0].reason, "Folders aren't imported");
        assert_eq!(parsed.skipped_entries[1].title, "Deleted");
        assert_eq!(
            parsed.skipped_entries[1].reason,
            "Item is in the 1Password trash"
        );
    }
}
