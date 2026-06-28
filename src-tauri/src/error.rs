#[derive(Debug, thiserror::Error)]
pub enum KagiError {
    #[error("Vault error: {0}")]
    Vault(String),

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

impl serde::Serialize for KagiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type KagiResult<T> = Result<T, KagiError>;
