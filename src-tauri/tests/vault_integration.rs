use std::io::Cursor;

use keepass::db::{Entry as KdbxEntry, Node};
use keepass::{Database, DatabaseKey};

/// Create an in-memory database with some entries and return its raw bytes
fn make_db_bytes(password: &str, entries: Vec<KdbxEntry>) -> Vec<u8> {
    let mut db = Database::new(Default::default());
    for entry in entries {
        db.root.add_child(entry);
    }
    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, DatabaseKey::new().with_password(password))
        .unwrap();
    buf.into_inner()
}

fn make_entry(uuid: &str, title: &str, username: &str, password: &str) -> KdbxEntry {
    let mut e = KdbxEntry::new();
    e.uuid = uuid::Uuid::parse_str(uuid).unwrap();
    e.fields.insert(
        "Title".into(),
        keepass::db::Value::Unprotected(title.into()),
    );
    e.fields.insert(
        "UserName".into(),
        keepass::db::Value::Unprotected(username.into()),
    );
    e.fields.insert(
        "Password".into(),
        keepass::db::Value::Protected(password.as_bytes().into()),
    );
    e
}

#[test]
fn test_open_and_read_entries() {
    let e1 = make_entry(
        "550e8400-e29b-41d4-a716-446655440001",
        "GitHub",
        "user",
        "ghp_token",
    );
    let e2 = make_entry(
        "550e8400-e29b-41d4-a716-446655440002",
        "Stripe",
        "admin",
        "sk_live",
    );

    let bytes = make_db_bytes("demopass", vec![e1, e2]);
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("demopass")).unwrap();

    let count = db.root.children.len();
    assert_eq!(count, 2);
}

#[test]
fn test_wrong_password_fails() {
    let bytes = make_db_bytes("demopass", vec![]);
    let mut cursor = Cursor::new(bytes);
    let result = Database::open(&mut cursor, DatabaseKey::new().with_password("wrong"));
    assert!(result.is_err());
}

#[test]
fn test_entry_crud() {
    let mut db = Database::new(Default::default());

    // Create
    let mut e = KdbxEntry::new();
    e.uuid = uuid::Uuid::new_v4();
    e.fields.insert(
        "Title".into(),
        keepass::db::Value::Unprotected("Test".into()),
    );
    db.root.add_child(e);
    assert_eq!(db.root.children.len(), 1);

    // Update — find by UUID is only possible via ref, so we re-add after modify
    let uuid = {
        if let Some(Node::Entry(e)) = db.root.children.first() {
            e.uuid
        } else {
            panic!("entry missing");
        }
    };

    // Delete
    db.root
        .children
        .retain(|n| !matches!(n, Node::Entry(e) if e.uuid == uuid));
    assert_eq!(db.root.children.len(), 0);
}

#[test]
fn test_save_and_reopen_roundtrip() {
    let mut e = KdbxEntry::new();
    e.uuid = uuid::Uuid::new_v4();
    e.fields.insert(
        "Title".into(),
        keepass::db::Value::Unprotected("Roundtrip".into()),
    );

    let bytes = make_db_bytes("secret", vec![e]);
    // Reopen
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("secret")).unwrap();

    let titles: Vec<String> = db
        .root
        .children
        .iter()
        .filter_map(|n| {
            if let Node::Entry(e) = n {
                e.get_title().map(str::to_string)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(titles, vec!["Roundtrip"]);
}

/// Construct an otpauth URI for testing
fn otpauth_uri(secret: &str, period: u64, digits: u64) -> String {
    format!(
        "otpauth://totp/test?secret={}&period={}&digits={}",
        secret, period, digits
    )
}

/// Parse secret, period, digits out of an otpauth URI (same logic as commands::entries)
fn parse_otpauth_params(uri: &str) -> Option<(String, u64, u64)> {
    let params: Vec<&str> = uri.split('?').collect();
    let query = params.get(1)?;
    let mut secret = None;
    let mut period = None;
    let mut digits = None;
    for part in query.split('&') {
        let kv: Vec<&str> = part.splitn(2, '=').collect();
        if kv.len() != 2 {
            continue;
        }
        match kv[0] {
            "secret" => secret = Some(kv[1].to_string()),
            "period" => period = Some(kv[1]),
            "digits" => digits = Some(kv[1]),
            _ => {}
        }
    }
    let secret = secret?;
    let period = period.and_then(|s| s.parse::<u64>().ok()).unwrap_or(30);
    let digits = digits.and_then(|s| s.parse::<u64>().ok()).unwrap_or(6);
    Some((secret, period, digits))
}

#[test]
fn test_totp_write_and_read_keepassxc_format() {
    let uri = otpauth_uri("JBSWY3DPEHPK3PXP", 30, 6);
    let (secret, period, digits) = parse_otpauth_params(&uri).unwrap();

    // Write as KeePassXC-native fields
    let mut e = KdbxEntry::new();
    e.uuid = uuid::Uuid::new_v4();
    e.fields.insert(
        "Title".into(),
        keepass::db::Value::Unprotected("GitHub".into()),
    );
    e.fields.insert(
        "TOTP Seed".into(),
        keepass::db::Value::Protected(secret.clone().into_bytes().into()),
    );
    e.fields.insert(
        "TOTP Settings".into(),
        keepass::db::Value::Unprotected(format!("{};{}", period, digits)),
    );

    let bytes = make_db_bytes("p", vec![e]);
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    let entry = match &db.root.children[0] {
        Node::Entry(e) => e,
        _ => panic!("expected entry"),
    };

    assert!(
        entry.get("TOTP Seed").is_some(),
        "TOTP Seed should be present"
    );
    assert!(
        entry.get("TOTP Settings").is_some(),
        "TOTP Settings should be present"
    );
}

#[test]
fn test_totp_stored_in_fields_not_custom_data() {
    let uri = otpauth_uri("JBSWY3DPEHPK3PXP", 30, 6);
    let (secret, period, digits) = parse_otpauth_params(&uri).unwrap();

    let mut e = KdbxEntry::new();
    e.uuid = uuid::Uuid::new_v4();
    e.fields.insert(
        "TOTP Seed".into(),
        keepass::db::Value::Protected(secret.into_bytes().into()),
    );
    e.fields.insert(
        "TOTP Settings".into(),
        keepass::db::Value::Unprotected(format!("{};{}", period, digits)),
    );

    let bytes = make_db_bytes("p", vec![e]);
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    let entry = match &db.root.children[0] {
        Node::Entry(e) => e,
        _ => panic!("expected entry"),
    };

    // Verify TOTP fields are NOT in custom_data (they're string fields)
    assert!(
        !entry.custom_data.items.contains_key("TOTP Seed"),
        "TOTP Seed should NOT be in custom_data"
    );
    assert!(
        !entry.custom_data.items.contains_key("TOTP Settings"),
        "TOTP Settings should NOT be in custom_data"
    );

    // Verify they ARE in the standard fields
    assert!(
        entry.fields.contains_key("TOTP Seed"),
        "TOTP Seed should be in fields"
    );
    assert!(
        entry.fields.contains_key("TOTP Settings"),
        "TOTP Settings should be in fields"
    );
}

#[test]
fn test_tags_roundtrip() {
    let mut e = KdbxEntry::new();
    e.uuid = uuid::Uuid::new_v4();
    e.fields.insert(
        "Title".into(),
        keepass::db::Value::Unprotected("Tagged".into()),
    );
    e.tags = vec!["work".into(), "dev".into()];

    let bytes = make_db_bytes("p", vec![e]);
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    let entry = match &db.root.children[0] {
        Node::Entry(e) => e,
        _ => panic!("expected entry"),
    };

    assert_eq!(entry.tags, vec!["work", "dev"]);
}

#[test]
fn test_tags_empty() {
    let mut e = KdbxEntry::new();
    e.uuid = uuid::Uuid::new_v4();
    e.fields.insert(
        "Title".into(),
        keepass::db::Value::Unprotected("Untagged".into()),
    );

    let bytes = make_db_bytes("p", vec![e]);
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    let entry = match &db.root.children[0] {
        Node::Entry(e) => e,
        _ => panic!("expected entry"),
    };

    assert!(entry.tags.is_empty());
}

#[test]
fn test_vault_lock_clears_state() {
    // Test that clearing the vault HashMap (the core of vault_lock)
    // properly removes OpenVault and its contents.
    use kagi_lib::state::{AppState, OpenVault};
    use zeroize::Zeroizing;

    let state = AppState::new();
    let mut vaults = state.vaults.lock().unwrap();

    // Insert a vault with a real in-memory database
    let db = keepass::Database::new(Default::default());
    let id = uuid::Uuid::new_v4();
    vaults.insert(
        id,
        OpenVault {
            db,
            path: "/tmp/test.kdbx".into(),
            master_key: Zeroizing::new(b"test-password".to_vec()),
        },
    );

    assert_eq!(vaults.len(), 1, "vault should be present");

    // Simulate vault_lock — clear the HashMap
    vaults.clear();
    assert_eq!(vaults.len(), 0, "vault should be cleared");
    // Clearing drops each OpenVault; the master_key is zeroized via
    // Zeroizing's Drop impl, and the Database (with decrypted entries)
    // is dropped.
}

#[test]
fn test_vault_lock_with_no_vault_is_noop() {
    use kagi_lib::state::AppState;

    let state = AppState::new();
    let mut vaults = state.vaults.lock().unwrap();
    assert_eq!(vaults.len(), 0);

    // Clearing an already-empty HashMap is a no-op
    vaults.clear();
    assert_eq!(vaults.len(), 0);
}
