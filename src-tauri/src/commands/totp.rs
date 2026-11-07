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

fn parse_totp(uri: &str) -> HitsuResult<keepass::db::TOTP> {
    let url =
        url::Url::parse(uri).map_err(|e| HitsuError::Custom(format!("Invalid TOTP URI: {e}")))?;
    if url.scheme() != "otpauth"
        || !url
            .host_str()
            .is_some_and(|host| host.eq_ignore_ascii_case("totp"))
    {
        return Err(HitsuError::Custom(
            "Unsupported OTP type; expected TOTP".into(),
        ));
    }
    for (key, value) in url.query_pairs() {
        if key.eq_ignore_ascii_case("encoder") && !value.eq_ignore_ascii_case("numeric") {
            return Err(HitsuError::Custom("Unsupported TOTP encoder".into()));
        }
        if key.eq_ignore_ascii_case("counter") {
            return Err(HitsuError::Custom("HOTP counters are not supported".into()));
        }
    }
    let mut totp: keepass::db::TOTP = uri.parse().map_err(|e: keepass::db::TOTPError| {
        HitsuError::Custom(format!("Invalid TOTP URI: {e}"))
    })?;
    // KeePass defaults to 8 digits; Key URI defaults to 6. Apply this at read
    // time so existing imports work without rewriting their stored secrets.
    if !url.query_pairs().any(|(key, _)| key == "digits") {
        totp.digits = 6;
    }
    if totp.period == 0 {
        return Err(HitsuError::Custom("Invalid TOTP period".into()));
    }
    // totp-lite computes 10^digits in u64; cap digits to its documented range
    // so malformed imported vaults cannot trigger arithmetic overflow.
    if !(1..=10).contains(&totp.digits) {
        return Err(HitsuError::Custom("Invalid TOTP digit count".into()));
    }

    Ok(totp)
}

pub(crate) fn compute_totp(uri: &str) -> HitsuResult<TotpCode> {
    let totp = parse_totp(uri)?;
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
    use super::{compute_totp, parse_totp};

    #[test]
    fn imported_uri_without_digits_uses_six_digit_standard_default() {
        let uri = "otpauth://totp/imported?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let totp = parse_totp(uri).unwrap();

        assert_eq!(totp.digits, 6);
        assert_eq!(totp.period, 30);
        assert_eq!(totp.value_at(59).code, "287082");
    }

    #[test]
    fn explicit_eight_digits_are_preserved() {
        let uri = "otpauth://totp/imported?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&digits=8";
        let totp = parse_totp(uri).unwrap();

        assert_eq!(totp.digits, 8);
        assert_eq!(totp.value_at(59).code, "94287082");
    }

    #[test]
    fn explicit_six_digits_and_custom_period_are_preserved() {
        let uri =
            "otpauth://totp/imported?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&digits=6&period=60";
        let totp = parse_totp(uri).unwrap();

        assert_eq!(totp.digits, 6);
        assert_eq!(totp.period, 60);
        assert_eq!(totp.value_at(119).code, "287082");
    }

    const SECRET: &str = "JBSWY3DPEHPK3PXP";

    #[test]
    fn rejects_unsupported_otp_formats() {
        for uri in [
            format!("otpauth://hotp/test?secret={SECRET}&counter=0"),
            format!("otpauth://steam/test?secret={SECRET}"),
            format!("https://totp/test?secret={SECRET}"),
            format!("otpauth://totp/test?secret={SECRET}&encoder=steam"),
            format!("otpauth://totp/test?secret={SECRET}&encoder=unknown"),
            format!("otpauth://totp/test?secret={SECRET}&encoder="),
            format!("otpauth://totp/test?secret={SECRET}&encoder=numeric&encoder=steam"),
            format!("otpauth://totp/test?secret={SECRET}&Encoder=Steam"),
            format!("otpauth://totp/test?secret={SECRET}&counter=0"),
        ] {
            assert!(compute_totp(&uri).is_err());
        }
    }

    #[test]
    fn explicit_numeric_encoder_preserves_standard_code() {
        for encoder in ["numeric", "NUMERIC"] {
            let uri = format!(
                "otpauth://totp/test?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&encoder={encoder}"
            );
            assert_eq!(parse_totp(&uri).unwrap().value_at(59).code, "287082");
        }
    }

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
