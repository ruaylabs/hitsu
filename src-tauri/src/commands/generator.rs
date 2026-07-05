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
    let length = opts.length.clamp(8, 100) as usize;

    let mut classes: Vec<Vec<u8>> = Vec::new();
    if opts.uppercase {
        classes.push(UPPERCASE.to_vec());
    }
    if opts.lowercase {
        classes.push(LOWERCASE.to_vec());
    }
    if opts.digits {
        classes.push(DIGITS.to_vec());
    }
    if opts.symbols {
        classes.push(SYMBOLS.to_vec());
    }

    if classes.is_empty() {
        // Default: at least lowercase
        classes.push(LOWERCASE.to_vec());
    }

    // Remove lookalikes if enabled
    if opts.exclude_lookalikes {
        for class in &mut classes {
            class.retain(|c| !LOOKALIKES.contains(c));
        }
        classes.retain(|class| !class.is_empty());
    }

    let charset: Vec<u8> = classes.concat();
    if charset.is_empty() {
        return Err(crate::error::KagiError::Custom(
            "No characters available after filtering".into(),
        ));
    }

    // Guarantee every enabled class appears: pick one char from each class
    // first (length is clamped to >= 8, so all 4 classes always fit), fill
    // the rest uniformly from the merged set, then shuffle so the guaranteed
    // chars don't sit at predictable positions. Same CSPRNG throughout.
    let mut rng = thread_rng();
    let mut bytes: Vec<u8> = Vec::with_capacity(length);
    for class in &classes {
        bytes.push(*class.choose(&mut rng).unwrap());
    }
    while bytes.len() < length {
        bytes.push(*charset.choose(&mut rng).unwrap());
    }
    bytes.shuffle(&mut rng);

    Ok(String::from_utf8(bytes).expect("charset is pure ASCII"))
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
        assert_eq!(pw.len(), 100);

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

    /// Coverage must hold deterministically, even at the minimum length
    /// where uniform sampling would miss a class in most draws.
    #[tokio::test]
    async fn test_every_enabled_class_present_at_min_length() {
        for _ in 0..200 {
            let pw = generate_password(PasswordOptions {
                length: 8,
                uppercase: true,
                lowercase: true,
                digits: true,
                symbols: true,
                exclude_lookalikes: false,
            })
            .await
            .unwrap();

            assert!(
                pw.chars().any(|c| c.is_ascii_uppercase()),
                "no uppercase in {pw}"
            );
            assert!(
                pw.chars().any(|c| c.is_ascii_lowercase()),
                "no lowercase in {pw}"
            );
            assert!(pw.chars().any(|c| c.is_ascii_digit()), "no digit in {pw}");
            assert!(
                pw.bytes().any(|b| SYMBOLS.contains(&b)),
                "no symbol in {pw}"
            );
        }
    }

    #[tokio::test]
    async fn test_class_coverage_with_lookalikes_excluded() {
        for _ in 0..200 {
            let pw = generate_password(PasswordOptions {
                length: 8,
                uppercase: true,
                lowercase: true,
                digits: true,
                symbols: false,
                exclude_lookalikes: true,
            })
            .await
            .unwrap();

            assert!(
                pw.chars().any(|c| c.is_ascii_uppercase()),
                "no uppercase in {pw}"
            );
            assert!(
                pw.chars().any(|c| c.is_ascii_lowercase()),
                "no lowercase in {pw}"
            );
            assert!(pw.chars().any(|c| c.is_ascii_digit()), "no digit in {pw}");
            assert!(
                !pw.bytes().any(|b| LOOKALIKES.contains(&b)),
                "lookalike in {pw}"
            );
        }
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
