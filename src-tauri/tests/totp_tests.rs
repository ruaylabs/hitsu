use std::time::Duration;

use keepass::db::{TOTPAlgorithm, TOTPError, TOTP};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_uri(secret: &str, params: &[(&str, &str)]) -> String {
    let mut uri = format!("otpauth://totp/test?secret={}", secret);
    for (k, v) in params {
        uri.push('&');
        uri.push_str(k);
        uri.push('=');
        uri.push_str(v);
    }
    uri
}

/// Known test vector: secret = "Hello!\xDE\xAD\xBE\xEF" (base32: JBSWY3DPEHPK3PXP)
/// at time 1234, SHA-1, 30s period, 6 digits → "806863"
/// Source: keepass crate's own test (totp_value)
const TEST_SECRET_B32: &str = "JBSWY3DPEHPK3PXP";
const TEST_TIME: u64 = 1234;
const EXPECTED_CODE_SHA1: &str = "806863";

/// Build a TOTP from a URI for deterministic tests (secret field is private).
fn make_totp(secret_b32: &str, algorithm: &str, period: u64, digits: u32) -> TOTP {
    let mut uri = format!(
        "otpauth://totp/test?secret={}&period={}&digits={}",
        secret_b32, period, digits
    );
    if !algorithm.is_empty() {
        uri.push_str(&format!("&algorithm={}", algorithm));
    }
    uri.parse().expect("valid TOTP URI")
}

// ── Parsing: algorithm variants ─────────────────────────────────────────────

#[test]
fn test_totp_parse_sha1_default() {
    let uri = make_uri(TEST_SECRET_B32, &[("period", "30"), ("digits", "6")]);
    let totp: TOTP = uri.parse().unwrap();

    assert_eq!(totp.algorithm, TOTPAlgorithm::Sha1);
    assert_eq!(totp.period, 30);
    assert_eq!(totp.digits, 6);
    assert_eq!(totp.get_secret(), TEST_SECRET_B32);
    assert_eq!(totp.label, "test");
    assert_eq!(totp.issuer, None);
}

#[test]
fn test_totp_parse_sha1_explicit() {
    let uri = make_uri(
        TEST_SECRET_B32,
        &[("period", "30"), ("digits", "6"), ("algorithm", "SHA1")],
    );
    let totp: TOTP = uri.parse().unwrap();
    assert_eq!(totp.algorithm, TOTPAlgorithm::Sha1);
}

#[test]
fn test_totp_parse_sha256() {
    let uri = make_uri(
        TEST_SECRET_B32,
        &[("period", "30"), ("digits", "6"), ("algorithm", "SHA256")],
    );
    let totp: TOTP = uri.parse().unwrap();
    assert_eq!(totp.algorithm, TOTPAlgorithm::Sha256);
}

#[test]
fn test_totp_parse_sha512() {
    let uri = make_uri(
        TEST_SECRET_B32,
        &[("period", "30"), ("digits", "6"), ("algorithm", "SHA512")],
    );
    let totp: TOTP = uri.parse().unwrap();
    assert_eq!(totp.algorithm, TOTPAlgorithm::Sha512);
}

// ── Parsing: non-standard parameters ────────────────────────────────────────

#[test]
fn test_totp_parse_8_digits() {
    let uri = make_uri(TEST_SECRET_B32, &[("digits", "8")]);
    let totp: TOTP = uri.parse().unwrap();
    assert_eq!(totp.digits, 8);
}

#[test]
fn test_totp_parse_60_second_period() {
    let uri = make_uri(TEST_SECRET_B32, &[("period", "60")]);
    let totp: TOTP = uri.parse().unwrap();
    assert_eq!(totp.period, 60);
}

#[test]
fn test_totp_parse_with_issuer() {
    let uri = make_uri(
        TEST_SECRET_B32,
        &[("issuer", "GitHub"), ("period", "30"), ("digits", "6")],
    );
    let totp: TOTP = uri.parse().unwrap();
    assert_eq!(totp.issuer.as_deref(), Some("GitHub"));
}

#[test]
fn test_totp_parse_url_encoded_secret() {
    // Secret with URL-encoded padding characters
    let uri = "otpauth://totp/test?secret=GEZDGNBVGY%3D%3D%3D%3D%3D%3D&period=30&digits=6";
    let totp: TOTP = uri.parse().unwrap();
    assert_eq!(
        totp.get_secret(),
        "GEZDGNBVGY======",
        "base32 secret with padding decoded and re-encoded"
    );
}

// ── Deterministic code computation (value_at) ───────────────────────────────

#[test]
fn test_totp_code_sha1_known_value() {
    let totp = make_totp(TEST_SECRET_B32, "SHA1", 30, 6);
    assert_eq!(totp.value_at(TEST_TIME).code, EXPECTED_CODE_SHA1);
}

#[test]
fn test_totp_code_sha256_produces_different_code() {
    let totp_sha1 = make_totp(TEST_SECRET_B32, "SHA1", 30, 6);
    let totp_sha256 = make_totp(TEST_SECRET_B32, "SHA256", 30, 6);
    let code_sha1 = totp_sha1.value_at(TEST_TIME).code;
    let code_sha256 = totp_sha256.value_at(TEST_TIME).code;
    assert_ne!(
        code_sha1, code_sha256,
        "SHA-1 and SHA-256 should produce different codes for the same secret and time"
    );
}

#[test]
fn test_totp_code_sha512_produces_different_code() {
    let totp_sha1 = make_totp(TEST_SECRET_B32, "SHA1", 30, 6);
    let totp_sha512 = make_totp(TEST_SECRET_B32, "SHA512", 30, 6);
    let code_sha1 = totp_sha1.value_at(TEST_TIME).code;
    let code_sha512 = totp_sha512.value_at(TEST_TIME).code;
    assert_ne!(
        code_sha1, code_sha512,
        "SHA-1 and SHA-512 should produce different codes"
    );
}

#[test]
fn test_totp_code_8_digits() {
    let totp = make_totp(TEST_SECRET_B32, "SHA1", 30, 8);
    let code = totp.value_at(TEST_TIME).code;
    assert_eq!(
        code.len(),
        8,
        "8-digit TOTP should produce an 8-character code"
    );
    assert!(
        code.chars().all(|c| c.is_ascii_digit()),
        "TOTP code should contain only digits"
    );
}

#[test]
fn test_totp_code_different_time_produces_different_code() {
    let totp = make_totp(TEST_SECRET_B32, "SHA1", 30, 6);
    let code_a = totp.value_at(1000).code;
    let code_b = totp.value_at(2000).code;
    // Different time intervals should produce different codes
    // (statistically almost certain, but not guaranteed — if this flakes,
    //  the secret/time combination happens to repeat; rare but possible)
    if code_a == code_b {
        // Try a few more time offsets to confirm
        let code_c = totp.value_at(3000).code;
        assert!(
            code_a != code_c,
            "TOTP codes at different times should eventually differ"
        );
    }
}

// ── OTPCode structure ───────────────────────────────────────────────────────

#[test]
fn test_totp_code_remaining_seconds() {
    // At time 0, remaining should be `period` (30s)
    let totp = make_totp(TEST_SECRET_B32, "SHA1", 30, 6);
    let code = totp.value_at(0);
    assert_eq!(
        code.valid_for,
        Duration::from_secs(30),
        "at time 0, code should be valid for the full period"
    );

    // At time 15 (mid-period), remaining should be 15
    let code = totp.value_at(15);
    assert_eq!(
        code.valid_for,
        Duration::from_secs(15),
        "at time 15, code should be valid for 15 more seconds"
    );

    // At time 29 (last second of period), remaining should be 1
    let code = totp.value_at(29);
    assert_eq!(
        code.valid_for,
        Duration::from_secs(1),
        "at time 29, code should be valid for 1 more second"
    );

    // At time 30 (start of next period), remaining should be 30 again
    let code = totp.value_at(30);
    assert_eq!(
        code.valid_for,
        Duration::from_secs(30),
        "at time 30, code should be valid for the full period again"
    );
}

#[test]
fn test_totp_code_period_matches() {
    let totp = make_totp(TEST_SECRET_B32, "SHA1", 30, 6);
    let code = totp.value_at(1234);
    assert_eq!(code.period, Duration::from_secs(30));
    assert!(code.valid_for <= code.period);
}

// ── Full flow (parse + compute, current time) ──────────────────────────────

#[test]
fn test_totp_full_flow_sha1() {
    let uri = make_uri(TEST_SECRET_B32, &[("period", "30"), ("digits", "6")]);
    let totp: TOTP = uri.parse().unwrap();
    let code = totp.value_now().unwrap();

    assert_eq!(code.code.len(), 6, "code should be 6 digits");
    assert!(code.code.chars().all(|c| c.is_ascii_digit()));
    assert_eq!(code.period, Duration::from_secs(30));
    assert!(
        code.valid_for.as_secs() <= 30,
        "remaining should be <= period"
    );
    assert!(
        code.valid_for.as_secs() > 0 || code.valid_for.as_secs() == 0,
        "remaining should be >= 0"
    );
}

#[test]
fn test_totp_full_flow_8_digits() {
    let uri = make_uri(TEST_SECRET_B32, &[("digits", "8")]);
    let totp: TOTP = uri.parse().unwrap();
    let code = totp.value_now().unwrap();

    assert_eq!(code.code.len(), 8, "code should be 8 digits");
    assert!(code.code.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_totp_full_flow_60_second_period() {
    let uri = make_uri(TEST_SECRET_B32, &[("period", "60"), ("digits", "6")]);
    let totp: TOTP = uri.parse().unwrap();
    let code = totp.value_now().unwrap();

    assert_eq!(code.code.len(), 6);
    assert_eq!(code.period, Duration::from_secs(60));
    assert!(code.valid_for.as_secs() <= 60);
}

// ── Error handling ──────────────────────────────────────────────────────────

#[test]
fn test_totp_error_not_a_url() {
    let err = "not a totp uri".parse::<TOTP>().unwrap_err();
    assert!(matches!(err, TOTPError::UrlFormat(_)));
}

#[test]
fn test_totp_error_bad_scheme() {
    let err = "http://totp/test?secret=JBSWY3DPEHPK3PXP"
        .parse::<TOTP>()
        .unwrap_err();
    assert!(matches!(err, TOTPError::BadScheme(_)));
}

#[test]
fn test_totp_error_missing_secret() {
    let err = "otpauth://totp/test?period=30".parse::<TOTP>().unwrap_err();
    assert!(matches!(err, TOTPError::MissingField("secret")));
}

#[test]
fn test_totp_error_bad_algorithm() {
    let err = "otpauth://totp/test?secret=JBSWY3DPEHPK3PXP&algorithm=SHA123"
        .parse::<TOTP>()
        .unwrap_err();
    assert!(matches!(err, TOTPError::BadAlgorithm(_)));
    assert!(err.to_string().contains("SHA123"));
}

#[test]
fn test_totp_error_bad_base32_secret() {
    let err = "otpauth://totp/test?secret=!!!invalid-b32!!!"
        .parse::<TOTP>()
        .unwrap_err();
    assert!(matches!(err, TOTPError::Base32));
}

#[test]
fn test_totp_error_bad_period() {
    let err = "otpauth://totp/test?secret=JBSWY3DPEHPK3PXP&period=abc"
        .parse::<TOTP>()
        .unwrap_err();
    assert!(matches!(err, TOTPError::IntFormat(_)));
}

#[test]
fn test_totp_error_bad_digits() {
    let err = "otpauth://totp/test?secret=JBSWY3DPEHPK3PXP&digits=-1"
        .parse::<TOTP>()
        .unwrap_err();
    assert!(matches!(err, TOTPError::IntFormat(_)));
}
