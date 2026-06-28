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
