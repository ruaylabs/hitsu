use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tauri::State;
use zeroize::Zeroizing;

use crate::biometric::{self, BiometricError};
use crate::error::{HitsuError, HitsuResult};
use crate::models::VaultMeta;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiometricStatus {
    pub available: bool,
    pub enabled: bool,
}

fn map_biometric_error(error: BiometricError) -> HitsuError {
    match error {
        BiometricError::Unavailable => HitsuError::BiometricUnavailable,
        BiometricError::NotFound => HitsuError::BiometricNotEnabled,
        BiometricError::Canceled => HitsuError::BiometricCanceled,
        BiometricError::AuthenticationFailed => HitsuError::BiometricFailed(
            "Touch ID couldn't verify your identity. Use your master password.".into(),
        ),
        error @ (BiometricError::InvalidData
        | BiometricError::Path(_)
        | BiometricError::Keychain(_)) => {
            tracing::debug!(error = %error, "Touch ID keychain operation failed");
            HitsuError::BiometricFailed(
                "Touch ID couldn't access the saved vault password. Use your master password."
                    .into(),
            )
        }
    }
}

fn paths_match(left: &Path, right: &Path) -> bool {
    left == right
        || std::fs::canonicalize(left)
            .ok()
            .zip(std::fs::canonicalize(right).ok())
            .is_some_and(|(left, right)| left == right)
}

#[tauri::command]
pub async fn biometric_status(path: String) -> HitsuResult<BiometricStatus> {
    let path = PathBuf::from(path);
    tauri::async_runtime::spawn_blocking(move || {
        let available = biometric::is_available();
        match biometric::item_exists(&path) {
            Ok(enabled) => BiometricStatus { available, enabled },
            Err(error) => {
                // Missing code-signing entitlements and inaccessible keychains
                // make the feature unavailable rather than breaking Settings.
                tracing::debug!(error = %error, "Touch ID status probe failed");
                BiometricStatus {
                    available: false,
                    enabled: false,
                }
            }
        }
    })
    .await
    .map_err(HitsuError::from_join)
}

#[tauri::command]
pub async fn biometric_enable(
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> HitsuResult<()> {
    let password = Zeroizing::new(password);
    let path = PathBuf::from(path);

    // Serialize verification and storage with password changes so an old
    // password cannot be cached after vault_change_password deletes its item.
    let _save_guard = state.save_lock.lock().await;
    {
        let vault = state.open_vault()?;
        if !paths_match(&vault.path, &path) {
            return Err(HitsuError::BiometricFailed(
                "Touch ID can only be enabled for the open vault.".into(),
            ));
        }

        let password_hash = Sha256::digest(password.as_bytes());
        if vault.password_hash[..].ct_ne(&*password_hash).into() {
            return Err(HitsuError::Custom("Wrong password".into()));
        }
    }

    tauri::async_runtime::spawn_blocking(move || biometric::store_password(&path, &password))
        .await
        .map_err(HitsuError::from_join)?
        .map_err(map_biometric_error)
}

#[tauri::command]
pub async fn biometric_disable(path: String) -> HitsuResult<()> {
    let path = PathBuf::from(path);
    tauri::async_runtime::spawn_blocking(move || biometric::delete_password(&path))
        .await
        .map_err(HitsuError::from_join)?
        .map_err(map_biometric_error)
}

#[tauri::command]
pub async fn biometric_unlock(state: State<'_, AppState>, path: String) -> HitsuResult<VaultMeta> {
    let keychain_path = PathBuf::from(&path);
    let password =
        tauri::async_runtime::spawn_blocking(move || biometric::read_password(&keychain_path))
            .await
            .map_err(HitsuError::from_join)?
            .map_err(map_biometric_error)?;

    let result = super::vault::open_vault_with_password(&state, path.clone(), password).await;
    if matches!(result, Err(HitsuError::KeepassOpen(_))) {
        let stale_path = PathBuf::from(path);
        match tauri::async_runtime::spawn_blocking(move || biometric::delete_password(&stale_path))
            .await
        {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                tracing::debug!(error = %error, "failed to remove stale Touch ID password")
            }
            Err(error) => {
                tracing::debug!(error = %error, "stale Touch ID cleanup task failed")
            }
        }
        return Err(HitsuError::BiometricFailed(
            "The saved password no longer unlocks this vault. Use your master password.".into(),
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serializes_for_the_frontend() {
        let status = BiometricStatus {
            available: true,
            enabled: false,
        };
        assert_eq!(
            serde_json::to_value(status).unwrap(),
            serde_json::json!({ "available": true, "enabled": false })
        );
    }
}
