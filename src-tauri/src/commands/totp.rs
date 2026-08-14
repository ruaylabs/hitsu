use serde::{Deserialize, Serialize};
use tauri::State;
use zeroize::Zeroizing;

use crate::error::{HitsuError, HitsuResult};
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

pub(crate) fn compute_totp(uri: &str) -> HitsuResult<TotpCode> {
    let totp: keepass::db::TOTP = uri.parse().map_err(|e: keepass::db::TOTPError| {
        HitsuError::Custom(format!("Invalid TOTP URI: {e}"))
    })?;
    if totp.period == 0 {
        return Err(HitsuError::Custom("Invalid TOTP period".into()));
    }
    // totp-lite computes 10^digits in u64; cap digits to its documented range
    // so malformed imported vaults cannot trigger arithmetic overflow.
    if !(1..=10).contains(&totp.digits) {
        return Err(HitsuError::Custom("Invalid TOTP digit count".into()));
    }

    let code = totp
        .value_now()
        .map_err(|e| HitsuError::Custom(format!("Clock error: {e}")))?;

    Ok(TotpCode {
        code: code.code,
        remaining: code.valid_for.as_secs(),
        period: code.period.as_secs(),
    })
}

#[tauri::command]
pub async fn totp_compute(state: State<'_, AppState>, id: String) -> HitsuResult<TotpCode> {
    let uri = {
        let vault = state.open_vault()?;
        let entry_ref = super::entries::find_entry_ref(&vault.db, &id)
            .ok_or_else(|| HitsuError::EntryNotFound(id.clone()))?;
        Zeroizing::new(
            super::entries::read_totp_seed(&entry_ref)
                .ok_or_else(|| HitsuError::Custom("Entry has no TOTP configured".into()))?,
        )
    };

    compute_totp(&uri)
}

#[cfg(test)]
mod tests {
    use super::compute_totp;

    const SECRET: &str = "JBSWY3DPEHPK3PXP";

    #[test]
    fn rejects_zero_period_before_code_generation() {
        let uri = format!("otpauth://totp/test?secret={SECRET}&period=0&digits=6");
        let error = compute_totp(&uri).unwrap_err();

        assert!(error.to_string().contains("Invalid TOTP period"));
    }

    #[test]
    fn rejects_digit_counts_that_would_overflow_totp_lite() {
        let uri = format!("otpauth://totp/test?secret={SECRET}&period=30&digits=20");
        let error = compute_totp(&uri).unwrap_err();

        assert!(error.to_string().contains("Invalid TOTP digit count"));
    }
}
