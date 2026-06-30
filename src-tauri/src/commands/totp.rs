use serde::{Deserialize, Serialize};

use crate::error::{KagiError, KagiResult};

/// Result of a TOTP computation, returned to the frontend.
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
pub async fn totp_compute(uri: String) -> KagiResult<TotpCode> {
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
