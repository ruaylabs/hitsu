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

#[derive(Clone, Serialize, Deserialize, Zeroize)]
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
            .field("card_holder", &self.card_holder)
            .field("card_number", &redacted_opt(&self.card_number))
            .field("card_type", &self.card_type)
            .field("card_exp_month", &self.card_exp_month)
            .field("card_exp_year", &self.card_exp_year)
            .field("card_cvv", &redacted_opt(&self.card_cvv))
            .field("card_pin", &redacted_opt(&self.card_pin))
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
    pub card_holder: Option<String>,
    pub card_number: Option<String>,
    pub card_type: Option<String>,
    pub card_exp_month: Option<String>,
    pub card_exp_year: Option<String>,
    pub card_cvv: Option<String>,
    pub card_pin: Option<String>,
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
            card_holder: Some("Alice".into()),
            card_number: Some("4242424242424242".into()),
            card_type: Some("Visa".into()),
            card_exp_month: Some("12".into()),
            card_exp_year: Some("2030".into()),
            card_cvv: Some("123".into()),
            card_pin: Some("0000".into()),
        };
        let s = format!("{patch:?}");

        // PII / metadata should pass through.
        for visible in ["email", "Alice", "Visa", "12", "2030", "alice@example.com"] {
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
        let without_password = EntryDraft {
            password: None,
            ..with_password.clone()
        };
        let with = format!("{with_password:?}");
        let without = format!("{without_password:?}");
        assert!(with.contains("Some(<redacted>)"));
        assert!(without.contains("None"));
        assert_ne!(with, without);
    }
}
