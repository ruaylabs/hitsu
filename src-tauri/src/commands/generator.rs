use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;

use crate::error::KagiResult;

const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{}|;:,.<>?";
const LOOKALIKES: &[u8] = b"il1Lo0O";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordOptions {
    pub length: u32,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub symbols: bool,
    pub exclude_lookalikes: bool,
}

#[tauri::command]
pub async fn generate_password(opts: PasswordOptions) -> KagiResult<String> {
    let length = opts.length.clamp(8, 128) as usize;
    let mut charset = Vec::new();

    if opts.uppercase {
        charset.extend_from_slice(UPPERCASE);
    }
    if opts.lowercase {
        charset.extend_from_slice(LOWERCASE);
    }
    if opts.digits {
        charset.extend_from_slice(DIGITS);
    }
    if opts.symbols {
        charset.extend_from_slice(SYMBOLS);
    }

    if charset.is_empty() {
        // Default: at least lowercase
        charset.extend_from_slice(LOWERCASE);
    }

    // Remove lookalikes if enabled
    if opts.exclude_lookalikes {
        charset.retain(|c| !LOOKALIKES.contains(c));
    }

    if charset.is_empty() {
        return Err(crate::error::KagiError::Custom(
            "No characters available after filtering".into(),
        ));
    }

    let mut rng = thread_rng();
    let password: String = (0..length)
        .map(|_| *charset.choose(&mut rng).unwrap() as char)
        .collect();

    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generates_correct_length() {
        for len in [8u32, 16, 32, 64] {
            let pw = generate_password(PasswordOptions {
                length: len,
                uppercase: true,
                lowercase: true,
                digits: true,
                symbols: false,
                exclude_lookalikes: false,
            })
            .await
            .unwrap();
            assert_eq!(pw.len(), len as usize, "length {} failed", len);
        }
    }

    #[tokio::test]
    async fn test_clamps_length() {
        let pw = generate_password(PasswordOptions {
            length: 200, // above max
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: false,
            exclude_lookalikes: false,
        })
        .await
        .unwrap();
        assert_eq!(pw.len(), 128);

        let pw = generate_password(PasswordOptions {
            length: 1, // below min
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: false,
            exclude_lookalikes: false,
        })
        .await
        .unwrap();
        assert_eq!(pw.len(), 8);
    }

    #[tokio::test]
    async fn test_contains_expected_chars() {
        let pw = generate_password(PasswordOptions {
            length: 100,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: true,
            exclude_lookalikes: false,
        })
        .await
        .unwrap();

        assert!(pw.chars().any(|c| c.is_uppercase()), "no uppercase");
        assert!(pw.chars().any(|c| c.is_lowercase()), "no lowercase");
        assert!(pw.chars().any(|c| c.is_ascii_digit()), "no digit");
        assert!(
            pw.chars().any(|c| "!@#$%^&*()-_=+[]{}|;:,.<>?".contains(c)),
            "no symbol"
        );
    }

    #[tokio::test]
    async fn test_defaults_to_lowercase() {
        let pw = generate_password(PasswordOptions {
            length: 16,
            uppercase: false,
            lowercase: false,
            digits: false,
            symbols: false,
            exclude_lookalikes: false,
        })
        .await
        .unwrap();
        assert!(pw.chars().all(|c| c.is_lowercase()));
    }

    #[tokio::test]
    async fn test_exclude_lookalikes() {
        let pw = generate_password(PasswordOptions {
            length: 100,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: false,
            exclude_lookalikes: true,
        })
        .await
        .unwrap();
        assert!(!pw.contains('i'), "contains i");
        assert!(!pw.contains('l'), "contains l");
        assert!(!pw.contains('1'), "contains 1");
        assert!(!pw.contains('L'), "contains L");
        assert!(!pw.contains('o'), "contains o");
        assert!(!pw.contains('0'), "contains 0");
        assert!(!pw.contains('O'), "contains O");
    }

    #[tokio::test]
    async fn test_generates_different_passwords() {
        let pw1 = generate_password(PasswordOptions {
            length: 16,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: false,
            exclude_lookalikes: false,
        })
        .await
        .unwrap();
        let pw2 = generate_password(PasswordOptions {
            length: 16,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: false,
            exclude_lookalikes: false,
        })
        .await
        .unwrap();
        assert_ne!(pw1, pw2, "two generations produced the same password");
    }
}
