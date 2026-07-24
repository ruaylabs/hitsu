use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::item_type::ItemType;

/// Custom `Debug` for `EntryDraft`: secret-bearing fields (`password`,
/// `totp`, `notes`) print as `<redacted>` so a `dbg!` or panic
/// message can't leak them into a log file.
///
/// `title`, `username`, and `url` are non-secret metadata and render
/// normally.
impl std::fmt::Debug for EntryDraft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryDraft")
            .field("title", &self.title)
            .field("username", &self.username)
            .field("password", &redacted_opt(&self.password))
            .field("url", &self.url)
            .field("notes", &redacted_opt(&self.notes))
            .field("totp", &redacted_opt(&self.totp))
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct EntryDraft {
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub totp: Option<String>,
}

/// Custom `Debug` for `EntryPatch`: secret fields (`password`, `totp`,
/// `notes`, `card_number`, `card_cvv`, `card_pin`) are redacted. The
/// rest (title, identity fields, expiry) render normally — they're
/// PII but not credentials.
impl std::fmt::Debug for EntryPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryPatch")
            .field("title", &self.title)
            .field("username", &self.username)
            .field("password", &redacted_opt(&self.password))
            .field("url", &self.url)
            .field("notes", &redacted_opt(&self.notes))
            .field("totp", &redacted_opt(&self.totp))
            .field("tags", &self.tags)
            .field("favorite", &self.favorite)
            .field("first_name", &self.first_name)
            .field("last_name", &self.last_name)
            .field("email", &self.email)
            .field("phone", &self.phone)
            .field("address", &self.address)
            .field("dob", &self.dob)
            .field("card_holder", &self.card_holder)
            .field("card_number", &redacted_opt(&self.card_number))
            .field("card_type", &self.card_type)
            .field("card_exp_month", &self.card_exp_month)
            .field("card_exp_year", &self.card_exp_year)
            .field("card_cvv", &redacted_opt(&self.card_cvv))
            .field("card_pin", &redacted_opt(&self.card_pin))
            .field("license_version", &self.license_version)
            .field("license_key", &redacted_opt(&self.license_key))
            .field("license_licensed_to", &self.license_licensed_to)
            .field("license_registered_email", &self.license_registered_email)
            .field("license_company", &self.license_company)
            .field("license_download_page", &self.license_download_page)
            .field("license_publisher", &self.license_publisher)
            .field("license_website", &self.license_website)
            .field("license_retail_price", &self.license_retail_price)
            .field("license_support_email", &self.license_support_email)
            .field("license_purchase_date", &self.license_purchase_date)
            .field("license_order_number", &self.license_order_number)
            .field("license_order_total", &self.license_order_total)
            .field("passport_type", &self.passport_type)
            .field("passport_issuing_country", &self.passport_issuing_country)
            .field("passport_number", &redacted_opt(&self.passport_number))
            .field("passport_full_name", &self.passport_full_name)
            .field("passport_sex", &self.passport_sex)
            .field("passport_nationality", &self.passport_nationality)
            .field(
                "passport_issuing_authority",
                &self.passport_issuing_authority,
            )
            .field("passport_birth_date", &self.passport_birth_date)
            .field("passport_birth_place", &self.passport_birth_place)
            .field("passport_issue_date", &self.passport_issue_date)
            .field("passport_expiry_date", &self.passport_expiry_date)
            .field("expires_at", &self.expires_at)
            .field("custom_fields", &self.custom_fields)
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Zeroize, ZeroizeOnDrop)]
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
    pub dob: Option<String>,
    pub card_holder: Option<String>,
    pub card_number: Option<String>,
    pub card_type: Option<String>,
    pub card_exp_month: Option<String>,
    pub card_exp_year: Option<String>,
    pub card_cvv: Option<String>,
    pub card_pin: Option<String>,
    pub license_version: Option<String>,
    pub license_key: Option<String>,
    pub license_licensed_to: Option<String>,
    pub license_registered_email: Option<String>,
    pub license_company: Option<String>,
    pub license_download_page: Option<String>,
    pub license_publisher: Option<String>,
    pub license_website: Option<String>,
    pub license_retail_price: Option<String>,
    pub license_support_email: Option<String>,
    pub license_purchase_date: Option<String>,
    pub license_order_number: Option<String>,
    pub license_order_total: Option<String>,
    pub passport_type: Option<String>,
    pub passport_issuing_country: Option<String>,
    pub passport_number: Option<String>,
    pub passport_full_name: Option<String>,
    pub passport_sex: Option<String>,
    pub passport_nationality: Option<String>,
    pub passport_issuing_authority: Option<String>,
    pub passport_birth_date: Option<String>,
    pub passport_birth_place: Option<String>,
    pub passport_issue_date: Option<String>,
    pub passport_expiry_date: Option<String>,
    pub expires_at: Option<String>,
    pub custom_fields: Option<Vec<CustomField>>,
}

/// Secret-bearing values required to populate the edit form. This DTO is
/// fetched only when an entry contains protected fields and is zeroized as
/// soon as the command response is dropped.
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct EntryEditPayload {
    pub password: String,
    pub totp: String,
    pub card_number: String,
    pub card_cvv: String,
    pub card_pin: String,
    pub license_key: String,
    pub passport_number: String,
    pub custom_fields: Vec<CustomField>,
}

/// Detail view of an entry sent to the webview.
///
/// Deliberately carries **no secret material**: passwords, TOTP URIs, card
/// numbers, CVVs and PINs stay in the Rust process. The frontend only learns
/// *that* a secret exists (`has_password`, `has_totp`, `CardFields` flags)
/// and fetches plaintext on demand via `entry_reveal_field`, or copies it
/// without ever seeing it via `entry_copy_field`.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
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
    pub has_password: bool,
    pub has_totp: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub trashed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<CardFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_license: Option<SoftwareLicenseFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passport: Option<PassportFields>,
    pub attachments: Vec<AttachmentMeta>,
    pub custom_fields: Vec<CustomField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub modified_at: String,
    pub created_at: String,
    pub history_count: u32,
}

/// Render an `Option<T>` for `Debug` as `Some(<redacted>)` or `None`,
/// without exposing the wrapped value. Used by the redacted `Debug`
/// impls of the secret-bearing DTOs above.
fn redacted_opt<T>(opt: &Option<T>) -> RedactedOpt<'_, T> {
    RedactedOpt(opt)
}

struct RedactedOpt<'a, T>(&'a Option<T>);

impl<T> std::fmt::Debug for RedactedOpt<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(_) => f.write_str("Some(<redacted>)"),
            None => f.write_str("None"),
        }
    }
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
pub struct FolderSummary {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
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
    pub has_password: bool,
    pub has_totp: bool,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub trashed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_hint: Option<String>,
    pub modified_at: String,
}

/// Custom `Debug` for `CustomField`: the `value` is redacted because
/// it can hold protected secrets (and unprotected values are often
/// sensitive too — recovery codes, security questions, etc.).
impl std::fmt::Debug for CustomField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomField")
            .field("name", &self.name)
            .field("value", &"<redacted>")
            .field("protected", &self.protected)
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct CustomField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentMeta {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
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

/// Passport fields as shown in the detail view. The passport number stays
/// backend-side and is represented only by `has_number`.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct PassportFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub passport_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuing_country: Option<String>,
    pub has_number: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nationality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuing_authority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_place: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_date: Option<String>,
}

/// Software-license fields as shown in the detail view. The license key stays
/// backend-side and is represented only by `has_license_key`.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareLicenseFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub has_license_key: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licensed_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registered_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_page: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retail_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_total: Option<String>,
}

/// Card fields as shown in the detail view: the number is pre-masked and
/// CVV/PIN are reduced to presence flags (see [`Entry`]).
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct CardFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_masked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub card_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp_month: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp_year: Option<u32>,
    pub has_number: bool,
    pub has_cvv: bool,
    pub has_pin: bool,
}

/// A secret field that can be revealed or copied on demand.
/// Deserialized from the camelCase strings the frontend sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SecretField {
    Password,
    /// The full otpauth:// URI (for the edit form, not for code display).
    Totp,
    CardNumber,
    CardCvv,
    CardPin,
    LicenseKey,
    PassportNumber,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultMeta {
    pub path: String,
    pub name: String,
    pub item_count: usize,
    pub sync_provider: String,
    #[serde(default)]
    pub kdf_needs_upgrade: bool,
    /// Entry summaries returned inline from vault_open so the frontend
    /// doesn't need a second entries_list round-trip after unlock.
    pub entries: Vec<EntrySummary>,
    pub folders: Vec<FolderSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultRefreshResult {
    pub changed: bool,
    pub reloaded: bool,
    pub vault: Option<VaultMeta>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `dbg!`/panic-message regression test: formatting an
    /// `EntryDraft` that contains real secrets must not print the
    /// secret values.
    #[test]
    fn entrydraft_debug_redacts_secrets() {
        let draft = EntryDraft {
            title: "Gmail".into(),
            username: Some("alice@example.com".into()),
            password: Some("hunter2-SECRET".into()),
            url: Some("https://gmail.com".into()),
            notes: Some("recovery: ANSWER-SECRET".into()),
            totp: Some("otpauth://totp/TOTP-SECRET".into()),
        };
        let s = format!("{draft:?}");

        assert!(s.contains("Gmail"), "non-secret title should appear: {s}");
        assert!(
            s.contains("alice@example.com"),
            "non-secret username should appear: {s}"
        );
        assert!(
            s.contains("https://gmail.com"),
            "non-secret url should appear: {s}"
        );
        for secret in ["hunter2-SECRET", "ANSWER-SECRET", "TOTP-SECRET"] {
            assert!(
                !s.contains(secret),
                "secret {secret:?} leaked into Debug: {s}"
            );
        }
        // Make sure redaction markers are actually present (not
        // accidentally elided).
        assert!(
            s.contains("Some(<redacted>)"),
            "missing redaction marker: {s}"
        );
    }

    #[test]
    fn entrypatch_debug_redacts_secrets_and_keeps_metadata() {
        let patch = EntryPatch {
            title: None,
            username: None,
            password: Some("hunter2-SECRET".into()),
            url: None,
            notes: Some("notes-SECRET".into()),
            totp: Some("otpauth://totp/TOTP-SECRET".into()),
            tags: Some(vec!["email".into()]),
            favorite: Some(true),
            first_name: None,
            last_name: None,
            email: Some("alice@example.com".into()),
            phone: None,
            address: None,
            dob: Some("1990-01-02".into()),
            card_holder: Some("Alice".into()),
            card_number: Some("4242424242424242".into()),
            card_type: Some("Visa".into()),
            card_exp_month: Some("12".into()),
            card_exp_year: Some("2030".into()),
            card_cvv: Some("123".into()),
            card_pin: Some("0000".into()),
            license_version: None,
            license_key: Some("LICENSE-KEY-SECRET".into()),
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
            passport_number: Some("PASSPORT-NUMBER-SECRET".into()),
            passport_full_name: None,
            passport_sex: None,
            passport_nationality: None,
            passport_issuing_authority: None,
            passport_birth_date: None,
            passport_birth_place: None,
            passport_issue_date: None,
            passport_expiry_date: None,
            expires_at: Some("2030-01-01".into()),
            custom_fields: Some(vec![CustomField {
                name: "Recovery answer".into(),
                value: "custom-SECRET".into(),
                protected: true,
            }]),
        };
        let s = format!("{patch:?}");

        // PII / metadata should pass through.
        for visible in [
            "email",
            "Alice",
            "Visa",
            "12",
            "2030",
            "1990-01-02",
            "alice@example.com",
        ] {
            assert!(s.contains(visible), "expected {visible:?} in Debug: {s}");
        }
        // Secrets must be redacted.
        for secret in [
            "hunter2-SECRET",
            "notes-SECRET",
            "TOTP-SECRET",
            "4242424242424242",
            "123",
            "0000",
            "LICENSE-KEY-SECRET",
            "PASSPORT-NUMBER-SECRET",
            "custom-SECRET",
        ] {
            assert!(
                !s.contains(secret),
                "secret {secret:?} leaked into Debug: {s}"
            );
        }
    }

    #[test]
    fn customfield_debug_redacts_value() {
        let cf = CustomField {
            name: "Backup email".into(),
            value: "recovery-SECRET@example.com".into(),
            protected: true,
        };
        let s = format!("{cf:?}");
        assert!(s.contains("Backup email"), "name should appear: {s}");
        assert!(
            s.contains("protected: true"),
            "protected flag should appear: {s}"
        );
        assert!(
            !s.contains("recovery-SECRET@example.com"),
            "value leaked into Debug: {s}"
        );
        assert!(
            s.contains("value: \"<redacted>\""),
            "missing redaction marker: {s}"
        );
    }

    #[test]
    fn debug_distinguishes_some_vs_none_for_redacted_fields() {
        // Knowing whether a field is set is OK — the leak risk is
        // only the value. Make sure `Some(<redacted>)` and `None`
        // still come through distinctly.
        let with_password = EntryDraft {
            title: "x".into(),
            username: None,
            password: Some("p".into()),
            url: None,
            notes: None,
            totp: None,
        };
        let mut without_password = with_password.clone();
        without_password.password = None;
        let with = format!("{with_password:?}");
        let without = format!("{without_password:?}");
        assert!(with.contains("Some(<redacted>)"));
        assert!(without.contains("None"));
        assert_ne!(with, without);
    }
}
