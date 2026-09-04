use std::path::Path;

#[cfg(any(target_os = "macos", test))]
use sha2::{Digest, Sha256};
#[cfg(not(target_os = "macos"))]
use zeroize::Zeroizing;

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[derive(Debug, thiserror::Error)]
pub(crate) enum BiometricError {
    #[error("Touch ID is unavailable")]
    Unavailable,
    #[error("Touch ID is not enabled for this vault")]
    NotFound,
    #[error("Touch ID authentication was canceled")]
    Canceled,
    #[error("Touch ID authentication failed")]
    AuthenticationFailed,
    #[error("the stored password is invalid")]
    InvalidData,
    #[error("the vault path could not be resolved")]
    Path(#[source] std::io::Error),
    #[error("keychain operation failed with status {0}")]
    Keychain(i32),
}

pub(crate) type BiometricResult<T> = Result<T, BiometricError>;

#[cfg(any(target_os = "macos", test))]
fn account_for_canonical_path(path: &Path) -> String {
    let mut hasher = Sha256::new();

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        hasher.update(path.as_os_str().as_bytes());
    }

    #[cfg(not(unix))]
    hasher.update(path.to_string_lossy().as_bytes());

    format!("{:x}", hasher.finalize())
}

#[cfg(target_os = "macos")]
fn account_for_path(path: &Path) -> BiometricResult<String> {
    let canonical = std::fs::canonicalize(path).map_err(BiometricError::Path)?;
    Ok(account_for_canonical_path(&canonical))
}

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub(crate) use macos::{delete_password, is_available, item_exists, read_password, store_password};

#[cfg(not(target_os = "macos"))]
pub(crate) fn is_available() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn item_exists(_path: &Path) -> BiometricResult<bool> {
    Ok(false)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn store_password(_path: &Path, _password: &str) -> BiometricResult<()> {
    Err(BiometricError::Unavailable)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn read_password(_path: &Path) -> BiometricResult<Zeroizing<String>> {
    Err(BiometricError::Unavailable)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn delete_password(_path: &Path) -> BiometricResult<()> {
    Err(BiometricError::Unavailable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_accounts_are_stable_hex_hashes() {
        let first = account_for_canonical_path(Path::new("/Users/alice/Vaults/personal.kdbx"));
        let same = account_for_canonical_path(Path::new("/Users/alice/Vaults/personal.kdbx"));
        let other = account_for_canonical_path(Path::new("/Users/alice/Vaults/work.kdbx"));

        assert_eq!(first, same);
        assert_ne!(first, other);
        assert_eq!(first.len(), 64);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert!(!first.contains("personal"));
    }
}
