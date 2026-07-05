#[derive(Debug, thiserror::Error)]
pub enum KagiError {
    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("No vault is currently open")]
    NoOpenVault,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("KDBX error: {0}")]
    KeepassOpen(#[from] keepass::error::DatabaseOpenError),

    #[error("KDBX save error: {0}")]
    KeepassSave(#[from] keepass::error::DatabaseSaveError),

    #[error("{0}")]
    Custom(String),
}

impl KagiError {
    /// Short, user-safe message for the UI.
    ///
    /// Raw library/OS errors can mention file paths or crate internals, so
    /// they are logged locally (see `Serialize` below) and never sent over
    /// IPC. `Custom` and the domain variants carry messages we authored, so
    /// their `Display` text is already safe to show.
    fn user_message(&self) -> String {
        match self {
            KagiError::EntryNotFound(_) | KagiError::NoOpenVault | KagiError::Custom(_) => {
                self.to_string()
            }
            KagiError::Io(_) => "A file operation failed.".to_string(),
            KagiError::KeepassOpen(_) => {
                "Couldn't unlock the vault. Check your master password; if it is correct, \
                 the file may be corrupted."
                    .to_string()
            }
            KagiError::KeepassSave(_) => {
                "Couldn't save the vault. Your changes were not written to disk.".to_string()
            }
        }
    }
}

impl serde::Serialize for KagiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Single choke point where command errors cross IPC to the webview:
        // log the full detail locally, send only the sanitized message.
        // LOGGING RULE: never log secret-bearing values here or anywhere —
        // no passwords, key material, TOTP seeds/URIs, or entry field values.
        // TODO: consider replacing stderr logging with `tracing` (leveled,
        // RUST_LOG-filterable) once more diagnostics land, e.g. the async
        // save-queue work in IMPROVEMENT_PLAN.md item 12.
        eprintln!("command failed: {self}");
        serializer.serialize_str(&self.user_message())
    }
}

pub type KagiResult<T> = Result<T, KagiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_message_is_sanitized() {
        let err = KagiError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "open /home/user/secret-vault.kdbx",
        ));
        let msg = err.user_message();
        assert!(!msg.contains("secret-vault"), "must not leak paths: {msg}");
    }

    #[test]
    fn test_authored_messages_pass_through() {
        assert_eq!(
            KagiError::Custom("Wrong password".into()).user_message(),
            "Wrong password"
        );
        assert_eq!(
            KagiError::NoOpenVault.user_message(),
            "No vault is currently open"
        );
        assert!(KagiError::EntryNotFound("abc".into())
            .user_message()
            .contains("abc"));
    }

    #[test]
    fn test_serialize_emits_user_message() {
        let err = KagiError::Io(std::io::Error::other("raw os detail"));
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"A file operation failed.\"");
    }
}
