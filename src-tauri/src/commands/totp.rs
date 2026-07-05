use serde::{Deserialize, Serialize};
use tauri::State;
use zeroize::Zeroizing;

use crate::error::{KagiError, KagiResult};
use crate::state::AppState;

/// Result of a TOTP computation, returned to the frontend.
///
/// Carries only the ephemeral 6/8-digit code — the otpauth:// URI (the
/// long-lived seed) is read backend-side from the entry and never crosses IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpCode {
    /// The current one-time password (zero-padded digits string).
    pub code: String,
    /// Seconds remaining before the current code expires.
    pub remaining: u64,
    /// Total period in seconds.
    pub period: u64,
}

#[tauri::command]
pub async fn totp_compute(state: State<'_, AppState>, id: String) -> KagiResult<TotpCode> {
    let uri = {
        let vaults = state.vaults.lock();
        let (_vault_id, vault) = vaults.iter().next().ok_or(KagiError::NoOpenVault)?;
        let entry_ref = super::entries::find_entry_ref(&vault.db, &id)
            .ok_or_else(|| KagiError::EntryNotFound(id.clone()))?;
        Zeroizing::new(
            super::entries::read_totp_seed(&entry_ref)
                .ok_or_else(|| KagiError::Custom("Entry has no TOTP configured".into()))?,
        )
    };

    let totp: keepass::db::TOTP = uri
        .parse()
        .map_err(|e: keepass::db::TOTPError| KagiError::Custom(format!("Invalid TOTP URI: {e}")))?;

    let code = totp
        .value_now()
        .map_err(|e| KagiError::Custom(format!("Clock error: {e}")))?;

    let remaining = code.valid_for.as_secs();
    let period = code.period.as_secs();

    Ok(TotpCode {
        code: code.code,
        remaining,
        period,
    })
}
