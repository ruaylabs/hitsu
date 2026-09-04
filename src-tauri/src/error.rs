#[derive(Debug, thiserror::Error)]
pub enum HitsuError {
    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("No vault is currently open")]
    NoOpenVault,

    #[error("Vault file changed on disk")]
    ExternalModification,

    #[error("Touch ID is not available on this Mac.")]
    BiometricUnavailable,

    #[error("Touch ID is not enabled for this vault. Use your master password.")]
    BiometricNotEnabled,

    #[error("Touch ID was canceled.")]
    BiometricCanceled,

    #[error("{0}")]
    BiometricFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("KDBX error: {0}")]
    KeepassOpen(#[from] keepass::error::DatabaseOpenError),

    #[error("KDBX save error: {0}")]
    KeepassSave(#[from] keepass::error::DatabaseSaveError),

    #[error("{0}")]
    Custom(String),
}

#[derive(serde::Serialize)]
struct IpcError<'a> {
    kind: &'static str,
    message: &'a str,
}

impl HitsuError {
    /// Map a failed `spawn_blocking` join (task panicked or the runtime is
    /// shutting down) to a user-safe error, logging the detail locally.
    pub fn from_join(err: impl std::fmt::Display) -> Self {
        tracing::error!("background task failed");
        tracing::debug!(error = %err, "background task failure detail");
        HitsuError::Custom("An internal error occurred".to_string())
    }

    fn kind(&self) -> &'static str {
        match self {
            HitsuError::EntryNotFound(_) => "entry_not_found",
            HitsuError::NoOpenVault => "no_open_vault",
            HitsuError::ExternalModification => "external_modification",
            HitsuError::BiometricUnavailable => "biometric_unavailable",
            HitsuError::BiometricNotEnabled => "biometric_not_enabled",
            HitsuError::BiometricCanceled => "biometric_canceled",
            HitsuError::BiometricFailed(_) => "biometric_failed",
            HitsuError::Io(_) => "io",
            HitsuError::KeepassOpen(_) => "keepass_open",
            HitsuError::KeepassSave(_) => "keepass_save",
            HitsuError::Custom(_) => "custom",
        }
    }

    /// Short, user-safe message for the UI.
    ///
    /// Raw library/OS errors can mention file paths or crate internals, so
    /// they are logged locally (see `Serialize` below) and never sent over
    /// IPC. `Custom` and the domain variants carry messages we authored, so
    /// their `Display` text is already safe to show.
    fn user_message(&self) -> String {
        match self {
            HitsuError::EntryNotFound(_)
            | HitsuError::NoOpenVault
            | HitsuError::BiometricUnavailable
            | HitsuError::BiometricNotEnabled
            | HitsuError::BiometricCanceled
            | HitsuError::BiometricFailed(_)
            | HitsuError::Custom(_) => self.to_string(),
            HitsuError::ExternalModification => {
                "The vault file was changed on disk by another program (a sync client?). \
                 Nothing was saved. Hitsu reloads the latest version automatically — \
                 finish or discard any edit in progress, then retry your change."
                    .to_string()
            }
            HitsuError::Io(_) => "A file operation failed.".to_string(),
            HitsuError::KeepassOpen(_) => {
                "Couldn't unlock the vault. Check your master password; if it is correct, \
                 the file may be corrupted."
                    .to_string()
            }
            HitsuError::KeepassSave(_) => {
                "Couldn't save the vault. Your changes were not written to disk.".to_string()
            }
        }
    }
}

impl serde::Serialize for HitsuError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Single choke point where command errors cross IPC to the webview.
        // Never log passwords, key material, TOTP seeds/URIs, or entry field
        // values. Raw errors may include paths, so their detail is debug-only.
        tracing::warn!(error_kind = self.kind(), "command failed");
        match self {
            HitsuError::Io(error) => {
                tracing::debug!(error = %error, error_kind = self.kind(), "command failure detail");
            }
            HitsuError::KeepassOpen(error) => {
                tracing::debug!(error = %error, error_kind = self.kind(), "command failure detail");
            }
            HitsuError::KeepassSave(error) => {
                tracing::debug!(error = %error, error_kind = self.kind(), "command failure detail");
            }
            // Authored custom messages can contain user-controlled labels or
            // attachment names, so do not write their values to logs.
            _ => {}
        }
        let message = self.user_message();
        IpcError {
            kind: self.kind(),
            message: &message,
        }
        .serialize(serializer)
    }
}

pub type HitsuResult<T> = Result<T, HitsuError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_message_is_sanitized() {
        let err = HitsuError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "open /home/user/secret-vault.kdbx",
        ));
        let msg = err.user_message();
        assert!(!msg.contains("secret-vault"), "must not leak paths: {msg}");
    }

    #[test]
    fn test_authored_messages_pass_through() {
        assert_eq!(
            HitsuError::Custom("Wrong password".into()).user_message(),
            "Wrong password"
        );
        assert_eq!(
            HitsuError::NoOpenVault.user_message(),
            "No vault is currently open"
        );
        assert!(HitsuError::EntryNotFound("abc".into())
            .user_message()
            .contains("abc"));
    }

    #[test]
    fn test_serialize_emits_user_safe_ipc_error() {
        let err = HitsuError::Io(std::io::Error::other("raw os detail"));
        assert_eq!(
            serde_json::to_value(&err).unwrap(),
            serde_json::json!({
                "kind": "io",
                "message": "A file operation failed."
            })
        );
    }

    #[test]
    fn test_biometric_error_kinds_and_messages() {
        assert_eq!(HitsuError::BiometricCanceled.kind(), "biometric_canceled");
        assert_eq!(
            HitsuError::BiometricCanceled.user_message(),
            "Touch ID was canceled."
        );
        assert_eq!(
            HitsuError::BiometricUnavailable.kind(),
            "biometric_unavailable"
        );
    }
}
