use serde::{Deserialize, Serialize};

use super::item_type::ItemType;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryDraft {
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub totp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EntryPatch {
    pub title: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub totp: Option<String>,
    pub tags: Option<Vec<String>>,
    pub favorite: Option<bool>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub card_holder: Option<String>,
    pub card_number: Option<String>,
    pub card_type: Option<String>,
    pub card_exp_month: Option<String>,
    pub card_exp_year: Option<String>,
    pub card_cvv: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub title: String,
    pub subtitle: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub favorite: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<CardFields>,
    pub attachments: Vec<AttachmentMeta>,
    pub custom_fields: Vec<CustomField>,
    pub modified_at: String,
    pub created_at: String,
    pub history_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntrySummary {
    pub version: u32,
    pub modified_at: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrySummary {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub title: String,
    pub subtitle: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub tags: Vec<String>,
    pub favorite: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentMeta {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dob: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub card_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp_month: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp_year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cvv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultMeta {
    pub path: String,
    pub name: String,
    pub item_count: usize,
    pub sync_provider: String,
}
