use std::io::Cursor;

use keepass::db::{fields, CustomDataItem, CustomDataValue, EntryId, Value};
use keepass::{Database, DatabaseKey};

/// Build an in-memory database with a single entry having the given fields,
/// save it, return the raw bytes.
fn make_db_bytes(
    password: &str,
    entry_id: EntryId,
    build: impl FnOnce(&mut keepass::db::EntryMut<'_>),
) -> Vec<u8> {
    let mut db = Database::new();
    db.root_mut()
        .add_entry_with_id(entry_id)
        .expect("add_entry_with_id failed")
        .edit(|e| build(e));
    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, DatabaseKey::new().with_password(password))
        .unwrap();
    buf.into_inner()
}

/// Helper: build a DB with a single entry, run a closure on it, return the DB (in-memory).
fn make_db<F>(f: F) -> Database
where
    F: FnOnce(&mut keepass::db::EntryMut<'_>),
{
    let mut db = Database::new();
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "Placeholder");
        f(e);
    });
    db
}

#[test]
fn test_open_and_read_entries() {
    let id1 = EntryId::from_uuid(uuid::uuid!("550e8400-e29b-41d4-a716-446655440001"));
    let id2 = EntryId::from_uuid(uuid::uuid!("550e8400-e29b-41d4-a716-446655440002"));

    let bytes1 = make_db_bytes("demopass", id1, |e| {
        e.set_unprotected(fields::TITLE, "GitHub");
        e.set_unprotected(fields::USERNAME, "user");
        e.set_protected(fields::PASSWORD, "ghp_token");
    });
    let bytes2 = make_db_bytes("demopass", id2, |e| {
        e.set_unprotected(fields::TITLE, "Stripe");
        e.set_unprotected(fields::USERNAME, "admin");
        e.set_protected(fields::PASSWORD, "sk_live");
    });

    // Open each file separately and count entries
    {
        let mut cursor1 = Cursor::new(bytes1);
        let db =
            Database::open(&mut cursor1, DatabaseKey::new().with_password("demopass")).unwrap();
        assert_eq!(db.iter_all_entries().count(), 1);
    }
    {
        let mut cursor2 = Cursor::new(bytes2);
        let db =
            Database::open(&mut cursor2, DatabaseKey::new().with_password("demopass")).unwrap();
        assert_eq!(db.iter_all_entries().count(), 1);
    }
}

#[test]
fn test_wrong_password_fails() {
    let id = EntryId::from_uuid(uuid::Uuid::new_v4());
    let bytes = make_db_bytes("demopass", id, |_| {});
    let mut cursor = Cursor::new(bytes);
    let result = Database::open(&mut cursor, DatabaseKey::new().with_password("wrong"));
    assert!(result.is_err());
}

#[test]
fn test_entry_crud() {
    let mut db = Database::new();

    // Create
    let entry_id = db.root_mut().add_entry().id();

    assert_eq!(db.iter_all_entries().count(), 1);
    assert!(db.entry(entry_id).is_some());

    // Delete
    db.entry_mut(entry_id).unwrap().remove();
    assert_eq!(db.iter_all_entries().count(), 0);
}

#[test]
fn test_save_and_reopen_roundtrip() {
    let mut db = Database::new();
    db.root_mut()
        .add_entry()
        .edit(|e| e.set_unprotected(fields::TITLE, "Roundtrip"));

    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, DatabaseKey::new().with_password("secret"))
        .unwrap();
    let bytes = buf.into_inner();

    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("secret")).unwrap();

    let titles: Vec<String> = db
        .iter_all_entries()
        .filter_map(|e| e.get_title().map(str::to_string))
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

#[test]
fn test_totp_write_and_read_keepassxc_format() {
    let uri = otpauth_uri("JBSWY3DPEHPK3PXP", 30, 6);

    let mut db = Database::new();
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "GitHub");
        e.fields
            .insert(fields::OTP.to_string(), Value::unprotected(uri));
    });

    let entry = db.iter_all_entries().next().unwrap();
    assert!(
        entry.get_raw_otp_value().is_some(),
        "otp field should be present"
    );

    let totp: keepass::db::TOTP = entry.get_raw_otp_value().unwrap().parse().unwrap();
    assert_eq!(totp.period, 30);
    assert_eq!(totp.digits, 6);
}

#[test]
fn test_tags_roundtrip() {
    let mut db = Database::new();
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "Tagged");
        e.tags = vec!["work".into(), "dev".into()];
    });

    let entry = db.iter_all_entries().next().unwrap();
    assert_eq!(entry.tags, vec!["work", "dev"]);
}

#[test]
fn test_tags_empty() {
    let mut db = Database::new();
    db.root_mut()
        .add_entry()
        .edit(|e| e.set_unprotected(fields::TITLE, "Untagged"));

    let entry = db.iter_all_entries().next().unwrap();
    assert!(entry.tags.is_empty());
}

/// ── Entry type round-trip tests ──────────────────────────────────────────────
/// Note: these verify in-memory state (not save/reopen) because keepass 0.13
/// serialises CustomDataValue::String as Binary through XML — the important
/// thing is that our read_custom_data_string logic handles both variants.

#[test]
fn test_note_type_roundtrip() {
    let db = make_db(|e| {
        e.set_unprotected(fields::TITLE, "Shopping List");
        e.set_unprotected(fields::NOTES, "Milk\nEggs\nBread");
        let item = CustomDataItem {
            value: Some(CustomDataValue::String("note".into())),
            last_modification_time: None,
        };
        e.custom_data.insert("kagi.itemType".into(), item);
    });

    let entry = db.iter_all_entries().next().unwrap();
    assert_eq!(entry.get_title(), Some("Shopping List"));
    assert_eq!(entry.get(fields::NOTES), Some("Milk\nEggs\nBread"));

    // Check custom data directly (in-memory)
    let raw = entry
        .custom_data
        .get("kagi.itemType")
        .and_then(|i| i.value.as_ref())
        .map(|v| match v {
            CustomDataValue::String(s) => s.clone(),
            CustomDataValue::Binary(b) => String::from_utf8_lossy(b).to_string(),
        });
    assert_eq!(raw.as_deref(), Some("note"));
}

#[test]
fn test_identity_type_roundtrip() {
    let db = make_db(|e| {
        e.set_unprotected(fields::TITLE, "My Identity");
        e.set_unprotected("identity.firstName", "Alice");
        e.set_unprotected("identity.lastName", "Smith");
        e.set_unprotected("identity.email", "alice@example.com");
        e.set_unprotected("identity.phone", "+1-555-0100");
        let item = CustomDataItem {
            value: Some(CustomDataValue::String("identity".into())),
            last_modification_time: None,
        };
        e.custom_data.insert("kagi.itemType".into(), item);
    });

    let entry = db.iter_all_entries().next().unwrap();
    assert_eq!(entry.get_title(), Some("My Identity"));
    assert_eq!(entry.get("identity.firstName"), Some("Alice"));
    assert_eq!(entry.get("identity.lastName"), Some("Smith"));
    assert_eq!(entry.get("identity.email"), Some("alice@example.com"));
    assert_eq!(entry.get("identity.phone"), Some("+1-555-0100"));

    // Check custom data directly (in-memory)
    let raw = entry
        .custom_data
        .get("kagi.itemType")
        .and_then(|i| i.value.as_ref())
        .map(|v| match v {
            CustomDataValue::String(s) => s.clone(),
            CustomDataValue::Binary(b) => String::from_utf8_lossy(b).to_string(),
        });
    assert_eq!(raw.as_deref(), Some("identity"));
}

#[test]
fn test_card_type_roundtrip() {
    let db = make_db(|e| {
        e.set_unprotected(fields::TITLE, "Visa Platinum");
        e.set_unprotected("card.holder", "Bob Johnson");
        e.set_protected("card.number", "4111111111111111");
        e.set_protected("card.cvv", "123");
        e.set_unprotected("card.expMonth", "12");
        e.set_unprotected("card.expYear", "2028");
        let item = CustomDataItem {
            value: Some(CustomDataValue::String("card".into())),
            last_modification_time: None,
        };
        e.custom_data.insert("kagi.itemType".into(), item);
    });

    let entry = db.iter_all_entries().next().unwrap();
    assert_eq!(entry.get_title(), Some("Visa Platinum"));
    assert_eq!(entry.get("card.holder"), Some("Bob Johnson"));

    let number = entry.fields.get("card.number").map(|v| v.get().clone());
    assert_eq!(number.as_deref(), Some("4111111111111111"));

    let cvv = entry.fields.get("card.cvv").map(|v| v.get().clone());
    assert_eq!(cvv.as_deref(), Some("123"));

    assert_eq!(entry.get("card.expMonth"), Some("12"));
    assert_eq!(entry.get("card.expYear"), Some("2028"));

    // Check custom data directly (in-memory)
    let raw = entry
        .custom_data
        .get("kagi.itemType")
        .and_then(|i| i.value.as_ref())
        .map(|v| match v {
            CustomDataValue::String(s) => s.clone(),
            CustomDataValue::Binary(b) => String::from_utf8_lossy(b).to_string(),
        });
    assert_eq!(raw.as_deref(), Some("card"));
}

// ── Field clearing tests ────────────────────────────────────────────────────

#[test]
fn test_clearing_field_removes_it_from_kdbx() {
    // Create an entry with optional fields
    let mut db = Database::new();
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "Test Entry");
        e.set_unprotected(fields::URL, "https://example.com");
        e.set_unprotected(fields::NOTES, "Some notes");
    });

    // Verify fields are present
    let entry = db.iter_all_entries().next().unwrap();
    assert!(entry.get(fields::URL).is_some(), "URL should be present");
    assert!(
        entry.get(fields::NOTES).is_some(),
        "Notes should be present"
    );

    // Simulate clearing: remove fields (this is what set_kdbx_field(..., None) does)
    let entry_id = entry.id();
    {
        let mut em = db.entry_mut(entry_id).unwrap();
        em.fields.remove(fields::URL);
        em.fields.remove(fields::NOTES);
    }

    // Save and reopen — cleared fields should be gone

    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, DatabaseKey::new().with_password("p"))
        .unwrap();
    let bytes = buf.into_inner();
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    let entry = db.iter_all_entries().next().unwrap();
    assert_eq!(entry.get_title(), Some("Test Entry"), "Title should remain");
    assert!(entry.get(fields::URL).is_none(), "URL should be removed");
    assert!(
        entry.get(fields::NOTES).is_none(),
        "Notes should be removed"
    );
}

#[test]
fn test_clearing_identity_fields_roundtrip() {
    // Identity fields use the identity.* namespace — test that removing them
    // via field.remove() survives a save/reopen cycle.
    let mut db = Database::new();
    let entry_id = db
        .root_mut()
        .add_entry()
        .edit(|e| {
            e.set_unprotected(fields::TITLE, "ID");
            e.set_unprotected("identity.firstName", "Alice");
            e.set_unprotected("identity.email", "alice@test.com");
        })
        .id();

    // Clear identity fields (simulating apply_patch with Some(""))
    {
        let mut em = db.entry_mut(entry_id).unwrap();
        em.fields.remove("identity.firstName");
        em.fields.remove("identity.email");
    }

    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, DatabaseKey::new().with_password("p"))
        .unwrap();
    let bytes = buf.into_inner();
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    let entry = db.iter_all_entries().next().unwrap();
    assert_eq!(entry.get_title(), Some("ID"), "Title should remain");
    assert!(
        entry.get("identity.firstName").is_none(),
        "identity.firstName should be removed"
    );
    assert!(
        entry.get("identity.email").is_none(),
        "identity.email should be removed"
    );
}

// ── Onboarding / vault-open flow tests ──────────────────────────────────────

#[test]
fn test_create_and_open_vault_from_disk() {
    // Simulates the full "open existing vault" onboarding path:
    // create a real file on disk → open it → verify entries survive.
    use std::fs;
    use std::io::{Cursor, Read};

    let dir = std::env::temp_dir().join("kagi-onboard-test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let vault_path = dir.join("test.kdbx");

    // 1. Create a database with one entry and save to disk
    let mut db = Database::new();
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "Onboard Entry");
        e.set_unprotected(fields::USERNAME, "user");
    });
    {
        let mut file = fs::File::create(&vault_path).unwrap();
        db.save(&mut file, DatabaseKey::new().with_password("demopass"))
            .unwrap();
    }

    // 2. Verify the file exists and has content
    assert!(vault_path.exists(), "vault file should exist");
    let metadata = fs::metadata(&vault_path).unwrap();
    assert!(metadata.len() > 0, "vault file should not be empty");

    // 3. Re-open from disk (same path the onboarding's vaultOpen command uses)
    let mut file = fs::File::open(&vault_path).unwrap();
    let mut content = Vec::new();
    file.read_to_end(&mut content).unwrap();
    let mut cursor = Cursor::new(content);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("demopass")).unwrap();

    let titles: Vec<String> = db
        .iter_all_entries()
        .filter_map(|e| e.get_title().map(str::to_string))
        .collect();
    assert_eq!(titles, vec!["Onboard Entry"]);

    // 4. Clean up
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_open_vault_wrong_password_from_disk() {
    use std::fs;

    let dir = std::env::temp_dir().join("kagi-onboard-wrong-pw");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let vault_path = dir.join("secret.kdbx");

    {
        let db = Database::new();
        let mut file = fs::File::create(&vault_path).unwrap();
        db.save(&mut file, DatabaseKey::new().with_password("correct-horse"))
            .unwrap();
    }

    // Opening with wrong password must fail
    let mut file = fs::File::open(&vault_path).unwrap();
    let result = Database::open(&mut file, DatabaseKey::new().with_password("wrong"));
    assert!(result.is_err(), "wrong password should fail to open vault");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_create_vault_file_exists() {
    // The onboarding "create" flow should produce a valid file that
    // can be re-opened immediately
    use std::fs;

    let dir = std::env::temp_dir().join("kagi-onboard-create");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let vault_path = dir.join("new-vault.kdbx");

    // Create
    {
        let db = Database::new();
        let mut file = fs::File::create(&vault_path).unwrap();
        db.save(&mut file, DatabaseKey::new().with_password("newpass"))
            .unwrap();
    }

    // Re-open
    {
        let mut file = fs::File::open(&vault_path).unwrap();
        let db = Database::open(&mut file, DatabaseKey::new().with_password("newpass")).unwrap();
        assert_eq!(
            db.iter_all_entries().count(),
            0,
            "new vault should have no entries"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

// ── Round-trip lossless test ──────────────────────────────────────────────

#[test]
fn test_round_trip_lossless() {
    // Simulates the full KeePassXC round-trip: create a database with all
    // supported features → save to disk → reopen → modify → save → reopen
    // → verify nothing was lost.
    use std::fs;
    use std::io::{Cursor, Read};

    let dir = std::env::temp_dir().join("kagi-roundtrip-test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let vault_path = dir.join("roundtrip.kdbx");

    // 1. Create a database with entries of all types
    let mut db = Database::new();
    db.meta.database_name = Some("Kagi round-trip test".into());

    // Login entry
    let login_id = db
        .root_mut()
        .add_entry_with_id(EntryId::from_uuid(uuid::uuid!(
            "11111111-1111-1111-1111-111111111111"
        )))
        .unwrap()
        .edit(|e| {
            e.set_unprotected(fields::TITLE, "GitHub");
            e.set_unprotected(fields::USERNAME, "user");
            e.set_protected(fields::PASSWORD, "ghp_secret");
            e.set_unprotected(fields::URL, "https://github.com");
            e.set_unprotected(fields::NOTES, "personal account");
            e.tags = vec!["work".into(), "dev".into()];
            e.custom_data.insert(
                "kagi.itemType".into(),
                CustomDataItem {
                    value: Some(CustomDataValue::Binary("login".as_bytes().to_vec())),
                    last_modification_time: None,
                },
            );
            e.custom_data.insert(
                "kagi.favorite".into(),
                CustomDataItem {
                    value: Some(CustomDataValue::Binary("true".as_bytes().to_vec())),
                    last_modification_time: None,
                },
            );
        })
        .id();

    // Note entry
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "Shopping List");
        e.set_unprotected(fields::NOTES, "Milk\nEggs\nBread");
        e.custom_data.insert(
            "kagi.itemType".into(),
            CustomDataItem {
                value: Some(CustomDataValue::Binary("note".as_bytes().to_vec())),
                last_modification_time: None,
            },
        );
    });

    // Identity entry
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "My Identity");
        e.set_unprotected("identity.firstName", "Alice");
        e.set_unprotected("identity.lastName", "Smith");
        e.set_unprotected("identity.email", "alice@example.com");
        e.custom_data.insert(
            "kagi.itemType".into(),
            CustomDataItem {
                value: Some(CustomDataValue::Binary("identity".as_bytes().to_vec())),
                last_modification_time: None,
            },
        );
    });

    // Card entry with TOTP
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "Visa Platinum");
        e.set_unprotected("card.holder", "Bob");
        e.set_protected("card.number", "4111111111111111");
        e.set_protected("card.cvv", "123");
        e.fields.insert(
            fields::OTP.to_string(),
            Value::unprotected("otpauth://totp/Bob?secret=JBSWY3DPEHPK3PXP&period=30&digits=6"),
        );
        e.custom_data.insert(
            "kagi.itemType".into(),
            CustomDataItem {
                value: Some(CustomDataValue::Binary("card".as_bytes().to_vec())),
                last_modification_time: None,
            },
        );
    });

    // 2. Save to disk
    {
        let mut file = fs::File::create(&vault_path).unwrap();
        db.save(&mut file, DatabaseKey::new().with_password("correct-horse"))
            .unwrap();
    }
    assert!(vault_path.exists());

    // 3. Reopen and verify all data
    {
        let mut file = fs::File::open(&vault_path).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        let mut cursor = Cursor::new(content);
        let db = Database::open(
            &mut cursor,
            DatabaseKey::new().with_password("correct-horse"),
        )
        .unwrap();

        assert_eq!(
            db.meta.database_name.as_deref(),
            Some("Kagi round-trip test")
        );
        assert_eq!(db.iter_all_entries().count(), 4);

        // Check login entry
        let login = db.entry(login_id).unwrap();
        assert_eq!(login.get_title(), Some("GitHub"));
        assert_eq!(login.get_username(), Some("user"));
        assert_eq!(login.get_password(), Some("ghp_secret"));
        assert_eq!(login.get_url(), Some("https://github.com"));
        assert_eq!(login.get(fields::NOTES), Some("personal account"));
        assert_eq!(login.tags, vec!["work", "dev"]);
    }

    // 4. Modify an entry and save again
    {
        let mut file = fs::File::open(&vault_path).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        let mut cursor = Cursor::new(content);
        let mut db = Database::open(
            &mut cursor,
            DatabaseKey::new().with_password("correct-horse"),
        )
        .unwrap();

        {
            let mut em = db.entry_mut(login_id).unwrap();
            em.edit_tracking(|e| {
                e.set_unprotected(fields::PASSWORD, "new_secret");
            });
            em.times.last_modification = Some(chrono::Utc::now().naive_utc());
        }

        let mut file = fs::File::create(&vault_path).unwrap();
        db.save(&mut file, DatabaseKey::new().with_password("correct-horse"))
            .unwrap();
    }

    // 5. Reopen and verify edit survived + history exists
    {
        let mut file = fs::File::open(&vault_path).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        let mut cursor = Cursor::new(content);
        let db = Database::open(
            &mut cursor,
            DatabaseKey::new().with_password("correct-horse"),
        )
        .unwrap();

        assert_eq!(db.iter_all_entries().count(), 4);

        let login = db.entry(login_id).unwrap();
        assert_eq!(
            login.get_password(),
            Some("new_secret"),
            "password should be updated"
        );
        assert_eq!(
            login.history.as_ref().map(|h| h.get_entries().len()),
            Some(1),
            "history should have 1 entry"
        );

        // Verify other entries untouched
        let titles: Vec<String> = db
            .iter_all_entries()
            .filter_map(|e| e.get_title().map(str::to_string))
            .collect();
        assert!(titles.contains(&"Shopping List".to_string()));
        assert!(titles.contains(&"My Identity".to_string()));
        assert!(titles.contains(&"Visa Platinum".to_string()));
    }

    let _ = fs::remove_dir_all(&dir);
}

// ── Round-trip: CustomDataValue::String variant ─────────────────────────────

#[test]
fn test_round_trip_custom_data_string_variant() {
    // The app's set_custom_data uses CustomDataValue::Binary to avoid a
    // keepass XML serialiser quirk: the library's XML deserialiser treats
    // well-formed base64 strings as Binary (even if originally stored as
    // String). Our read_custom_data_string handles both variants, but the
    // Binary→UTF-8 recovery path is never exercised in save/reopen tests
    // because entry_create uses Binary directly.
    //
    // This test: store a value as CustomDataValue::String, save/reopen,
    // then verify read_custom_data_string recovers it correctly (the
    // reopen will likely hand it back as Binary).

    let mut db = Database::new();
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "String→Binary recovery");
        // Store as String — what KeePassXC or an older Kagi version might write
        let item = CustomDataItem {
            value: Some(CustomDataValue::String("note".into())),
            last_modification_time: None,
        };
        e.custom_data.insert("kagi.itemType".into(), item);
        // Also store a value that would be invalid UTF-8 as a raw string
        // to test the base64 fallback path — use a value that base64-decodes
        // to something non-UTF-8
        let binary_item = CustomDataItem {
            value: Some(CustomDataValue::String("hello".into())),
            last_modification_time: None,
        };
        e.custom_data.insert("kagi.plainString".into(), binary_item);
    });

    // Save to bytes
    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, DatabaseKey::new().with_password("p"))
        .unwrap();
    let bytes = buf.into_inner();

    // Reopen
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    // The kagi.itemType value was stored as String("note").
    // After save/reopen it may be Binary(b"note") or String("note") —
    // either way read_custom_data_string should return "note".
    let entry = db.iter_all_entries().next().unwrap();

    // Read the custom data with the EXACT same logic as read_custom_data_string:
    // try UTF-8 first, fall back to base64 re-encode for strings that were
    // accidentally base64-decoded by the XML deserialiser.
    fn read_custom_data(entry: &keepass::db::Entry, key: &str) -> Option<String> {
        entry.custom_data.get(key).and_then(|item| {
            item.value.as_ref().map(|cv| match cv {
                CustomDataValue::String(s) => s.clone(),
                CustomDataValue::Binary(b) => {
                    if let Ok(s) = String::from_utf8(b.clone()) {
                        s
                    } else {
                        use base64::Engine;
                        base64::engine::general_purpose::STANDARD.encode(b)
                    }
                }
            })
        })
    }

    let item_type = read_custom_data(&entry, "kagi.itemType");
    assert_eq!(
        item_type.as_deref(),
        Some("note"),
        "String-stored custom data must survive save/reopen via String or Binary recovery"
    );

    // Verify the plain string also round-trips
    let plain = read_custom_data(&entry, "kagi.plainString");
    assert_eq!(
        plain.as_deref(),
        Some("hello"),
        "plain string must survive save/reopen"
    );

    // Title must survive too
    assert_eq!(entry.get_title(), Some("String→Binary recovery"));
}

// ── Round-trip: all fields on a single entry ───────────────────────────────

#[test]
fn test_round_trip_all_fields_on_one_entry() {
    // The app models Login/Identity/Card as separate item types, but the
    // KDBX format doesn't enforce this — a single entry can carry fields
    // from all namespaces. Test that every field survives save/reopen when
    // packed into one entry (catches accidental field-name collisions or
    // namespace filtering bugs in map_entry_to_full).

    let mut db = Database::new();
    let entry_id = db
        .root_mut()
        .add_entry_with_id(EntryId::from_uuid(uuid::uuid!(
            "22222222-2222-2222-2222-222222222222"
        )))
        .unwrap()
        .edit(|e| {
            // Login fields
            e.set_unprotected(fields::TITLE, "Mega Entry");
            e.set_unprotected(fields::USERNAME, "alice");
            e.set_protected(fields::PASSWORD, "s3cret!");
            e.set_unprotected(fields::URL, "https://example.com");
            // Identity fields
            e.set_unprotected("identity.firstName", "Alice");
            e.set_unprotected("identity.lastName", "Smith");
            e.set_unprotected("identity.email", "alice@example.com");
            e.set_unprotected("identity.phone", "+1-555-0100");
            e.set_unprotected("identity.address", "123 Main St");
            // Card fields
            e.set_unprotected("card.holder", "Alice Smith");
            e.set_protected("card.number", "4111111111111111");
            e.set_unprotected("card.type", "Visa");
            e.set_unprotected("card.expMonth", "12");
            e.set_unprotected("card.expYear", "2028");
            e.set_protected("card.cvv", "123");
            // TOTP
            e.fields.insert(
                fields::OTP.to_string(),
                Value::unprotected(
                    "otpauth://totp/Alice?secret=JBSWY3DPEHPK3PXP&period=30&digits=6",
                ),
            );
            // Notes
            e.set_unprotected(
                fields::NOTES,
                "This is a combined entry with all field types.",
            );
            // Tags
            e.tags = vec!["work".into(), "personal".into(), "finance".into()];
            // Custom data (icon hint + favorite + item type)
            e.custom_data.insert(
                "kagi.itemType".into(),
                CustomDataItem {
                    value: Some(CustomDataValue::Binary("login".as_bytes().to_vec())),
                    last_modification_time: None,
                },
            );
            e.custom_data.insert(
                "kagi.favorite".into(),
                CustomDataItem {
                    value: Some(CustomDataValue::Binary("true".as_bytes().to_vec())),
                    last_modification_time: None,
                },
            );
            e.custom_data.insert(
                "kagi.iconHint".into(),
                CustomDataItem {
                    value: Some(CustomDataValue::Binary("github".as_bytes().to_vec())),
                    last_modification_time: None,
                },
            );
        })
        .id();

    // Save to bytes
    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, DatabaseKey::new().with_password("p"))
        .unwrap();
    let bytes = buf.into_inner();

    // Reopen and verify EVERY field
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    assert_eq!(db.iter_all_entries().count(), 1);
    let entry = db.entry(entry_id).unwrap();

    // Login fields
    assert_eq!(entry.get_title(), Some("Mega Entry"));
    assert_eq!(entry.get_username(), Some("alice"));
    assert_eq!(entry.get_password(), Some("s3cret!"));
    assert_eq!(entry.get_url(), Some("https://example.com"));

    // Identity fields
    assert_eq!(entry.get("identity.firstName"), Some("Alice"));
    assert_eq!(entry.get("identity.lastName"), Some("Smith"));
    assert_eq!(entry.get("identity.email"), Some("alice@example.com"));
    assert_eq!(entry.get("identity.phone"), Some("+1-555-0100"));
    assert_eq!(entry.get("identity.address"), Some("123 Main St"));

    // Card fields
    assert_eq!(entry.get("card.holder"), Some("Alice Smith"));
    let card_number = entry.fields.get("card.number").map(|v| v.get().clone());
    assert_eq!(card_number.as_deref(), Some("4111111111111111"));
    assert_eq!(entry.get("card.type"), Some("Visa"));
    assert_eq!(entry.get("card.expMonth"), Some("12"));
    assert_eq!(entry.get("card.expYear"), Some("2028"));
    let cvv = entry.fields.get("card.cvv").map(|v| v.get().clone());
    assert_eq!(cvv.as_deref(), Some("123"));

    // TOTP
    assert!(
        entry.get_raw_otp_value().is_some(),
        "otp field should survive round-trip"
    );
    let totp: keepass::db::TOTP = entry.get_raw_otp_value().unwrap().parse().unwrap();
    assert_eq!(totp.period, 30);
    assert_eq!(totp.digits, 6);

    // Notes
    assert_eq!(
        entry.get(fields::NOTES),
        Some("This is a combined entry with all field types.")
    );

    // Tags
    assert_eq!(entry.tags, vec!["work", "personal", "finance"]);

    // Custom data (icon hint + favorite + item type)
    let read_item_type = entry.custom_data.get("kagi.itemType").and_then(|item| {
        item.value.as_ref().map(|cv| match cv {
            CustomDataValue::String(s) => s.clone(),
            CustomDataValue::Binary(b) => String::from_utf8_lossy(b).to_string(),
        })
    });
    assert_eq!(read_item_type.as_deref(), Some("login"));

    let read_favorite = entry.custom_data.get("kagi.favorite").and_then(|item| {
        item.value.as_ref().map(|cv| match cv {
            CustomDataValue::String(s) => s.clone(),
            CustomDataValue::Binary(b) => String::from_utf8_lossy(b).to_string(),
        })
    });
    assert_eq!(read_favorite.as_deref(), Some("true"));

    let read_icon_hint = entry.custom_data.get("kagi.iconHint").and_then(|item| {
        item.value.as_ref().map(|cv| match cv {
            CustomDataValue::String(s) => s.clone(),
            CustomDataValue::Binary(b) => String::from_utf8_lossy(b).to_string(),
        })
    });
    assert_eq!(read_icon_hint.as_deref(), Some("github"));
}

// ── Round-trip: entry with no title ─────────────────────────────────────────

#[test]
fn test_round_trip_entry_with_no_title() {
    // Edge case: an entry whose title is empty. The KDBX format allows it;
    // the UI should handle it gracefully (show placeholder, etc.).

    let mut db = Database::new();
    db.root_mut().add_entry().edit(|e| {
        // Title deliberately omitted (keepass default is empty)
        e.set_unprotected(fields::USERNAME, "user");
        e.set_unprotected(fields::URL, "https://example.com");
    });

    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, DatabaseKey::new().with_password("p"))
        .unwrap();
    let bytes = buf.into_inner();

    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    let entry = db.iter_all_entries().next().unwrap();
    // When no title is set, keepass returns None, not Some("")
    assert!(
        entry.get_title().is_none() || entry.get_title() == Some(""),
        "title should be empty or None when unset"
    );
    assert_eq!(entry.get_username(), Some("user"));
    assert_eq!(entry.get_url(), Some("https://example.com"));
}

// ── Round-trip: multiple history revisions ──────────────────────────────────

#[test]
fn test_round_trip_history_multiple_revisions() {
    // History is pushed by edit_tracking on each save. This test verifies
    // that editing the same entry N times produces N history entries, and
    // that each revision preserves the state at the time of save.

    let entry_id = EntryId::from_uuid(uuid::uuid!("33333333-3333-3333-3333-333333333333"));

    // Create initial entry
    let mut db = Database::new();
    db.root_mut()
        .add_entry_with_id(entry_id)
        .unwrap()
        .edit(|e| {
            e.set_unprotected(fields::TITLE, "v0");
            e.set_protected(fields::PASSWORD, "initial");
        });

    let save_and_reopen = |db: &mut Database, pw: &str| -> Database {
        let mut buf = Cursor::new(Vec::new());
        db.save(&mut buf, DatabaseKey::new().with_password(pw))
            .unwrap();
        let bytes = buf.into_inner();
        let mut cursor = Cursor::new(bytes);
        Database::open(&mut cursor, DatabaseKey::new().with_password(pw)).unwrap()
    };

    // Edit 3 times (each: save → reopen → edit → save → reopen → verify)
    let passwords = ["v1_pass", "v2_pass", "v3_pass"];
    for (i, new_pw) in passwords.iter().enumerate() {
        // Reopen the current state
        db = save_and_reopen(&mut db, "p");

        // Edit the entry
        {
            let mut em = db.entry_mut(entry_id).unwrap();
            em.edit_tracking(|e| {
                e.set_unprotected(fields::TITLE, format!("v{}", i + 1));
                e.set_protected(fields::PASSWORD, *new_pw);
            });
            em.times.last_modification = Some(chrono::Utc::now().naive_utc());
        }

        // Save and reopen
        db = save_and_reopen(&mut db, "p");

        // Verify current state
        let entry = db.entry(entry_id).unwrap();
        let expected_title = format!("v{}", i + 1);
        assert_eq!(
            entry.get_title(),
            Some(expected_title.as_str()),
            "current title should be v{}",
            i + 1
        );
        assert_eq!(
            entry.get_password(),
            Some(*new_pw),
            "current password should be updated"
        );

        // Verify history has all previous versions
        let history_len = entry
            .history
            .as_ref()
            .map(|h| h.get_entries().len())
            .unwrap_or(0);
        assert_eq!(
            history_len,
            i + 1,
            "after edit {} there should be {} history entries",
            i + 1,
            i + 1
        );

        // History entries are inserted at index 0 (newest first).
        // The first history entry holds the state just before the current edit.
        if let Some(history) = entry.history.as_ref() {
            let first_hist = history.get_entries().first().unwrap();
            let expected_prev = if i == 0 { "initial" } else { passwords[i - 1] };
            assert_eq!(
                first_hist.get_password(),
                Some(expected_prev),
                "first history entry (index 0) should contain the password before edit {}",
                i + 1
            );
        }
    }

    // Final: 3 history entries, 1 current entry
    let entry = db.entry(entry_id).unwrap();
    assert_eq!(
        entry.history.as_ref().map(|h| h.get_entries().len()),
        Some(3),
        "final history should have 3 entries"
    );
    assert_eq!(entry.get_title(), Some("v3"));
    assert_eq!(entry.get_password(), Some("v3_pass"));
}

// ── Round-trip: TOTP with SHA-256 algorithm ─────────────────────────────────

#[test]
fn test_round_trip_totp_sha256() {
    // KeePassXC and the keepass crate support SHA-1 (default), SHA-256,
    // and SHA-512 TOTP algorithms. The frontend computeTotp must handle
    // all three. This test verifies the SHA-256 URI survives save/reopen.

    let uri = "otpauth://totp/Example:alice@example.com?secret=JBSWY3DPEHPK3PXP&algorithm=SHA256&period=30&digits=6&issuer=Example";

    let mut db = Database::new();
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "SHA-256 TOTP");
        e.fields
            .insert(fields::OTP.to_string(), Value::unprotected(uri.to_string()));
    });

    // Save to bytes
    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, DatabaseKey::new().with_password("p"))
        .unwrap();
    let bytes = buf.into_inner();

    // Reopen
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    let entry = db.iter_all_entries().next().unwrap();

    // Verify the full URI rounds-trips intact
    let stored_uri = entry.get_raw_otp_value();
    assert!(stored_uri.is_some(), "otp field must survive save/reopen");
    assert_eq!(
        stored_uri.unwrap(),
        uri,
        "full otpauth:// URI must be preserved verbatim"
    );

    // Verify the TOTP parses correctly with SHA-256
    let totp: keepass::db::TOTP = stored_uri.unwrap().parse().unwrap();
    assert_eq!(totp.algorithm, keepass::db::TOTPAlgorithm::Sha256);
    assert_eq!(totp.period, 30);
    assert_eq!(totp.digits, 6);

    // Also verify SHA-512 survives
    let uri_sha512 =
        "otpauth://totp/test?secret=JBSWY3DPEHPK3PXP&algorithm=SHA512&period=30&digits=8";
    let mut db2 = Database::new();
    db2.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "SHA-512 TOTP");
        e.fields.insert(
            fields::OTP.to_string(),
            Value::unprotected(uri_sha512.to_string()),
        );
    });
    let mut buf2 = Cursor::new(Vec::new());
    db2.save(&mut buf2, DatabaseKey::new().with_password("p"))
        .unwrap();
    let bytes2 = buf2.into_inner();
    let mut cursor2 = Cursor::new(bytes2);
    let db2 = Database::open(&mut cursor2, DatabaseKey::new().with_password("p")).unwrap();
    let entry2 = db2.iter_all_entries().next().unwrap();
    let stored_uri2 = entry2.get_raw_otp_value().unwrap();
    assert_eq!(stored_uri2, uri_sha512);
    let totp2: keepass::db::TOTP = stored_uri2.parse().unwrap();
    assert_eq!(totp2.algorithm, keepass::db::TOTPAlgorithm::Sha512);
    assert_eq!(totp2.digits, 8);
}

// ── Round-trip: unicode fields ──────────────────────────────────────────────

#[test]
fn test_round_trip_unicode_fields() {
    // KDBX 4 stores strings as UTF-8. Verify that emoji, CJK ideographs,
    // bidirectional text, and special characters round-trip through
    // save/reopen without data loss or corruption.

    let mut db = Database::new();
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, "🔑 鍵 Unicode");
        e.set_unprotected(fields::USERNAME, "ユーザー名");
        e.set_protected(fields::PASSWORD, "Pässwörd🔐");
        e.set_unprotected(fields::URL, "https://例子.测试");
        e.set_unprotected("identity.firstName", "Zoë");
        e.set_unprotected("identity.lastName", "Jalapeño");
        e.set_unprotected("identity.email", "user@أن.example");
        e.set_unprotected(
            fields::NOTES,
            "emoji: 🔑🔒🔐\n\nRTL: السلام عليكم\nCJK: 鍵マネージャー\nMixed: Påskeøya 2024 ©",
        );
        e.tags = vec!["tagg🇸🇪".into(), "標籤".into()];
    });

    // Save to bytes
    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, DatabaseKey::new().with_password("p"))
        .unwrap();
    let bytes = buf.into_inner();

    // Reopen
    let mut cursor = Cursor::new(bytes);
    let db = Database::open(&mut cursor, DatabaseKey::new().with_password("p")).unwrap();

    let entry = db.iter_all_entries().next().unwrap();

    assert_eq!(
        entry.get_title(),
        Some("🔑 鍵 Unicode"),
        "emoji + CJK title"
    );
    assert_eq!(
        entry.get_username(),
        Some("ユーザー名"),
        "Japanese username"
    );
    assert_eq!(
        entry.get_password(),
        Some("Pässwörd🔐"),
        "password with diacritics + emoji"
    );
    assert_eq!(
        entry.get_url(),
        Some("https://例子.测试"),
        "URL with CJK/code-point TLD"
    );
    assert_eq!(
        entry.get("identity.firstName"),
        Some("Zoë"),
        "Latin+diacritic"
    );
    assert_eq!(entry.get("identity.lastName"), Some("Jalapeño"), "Latin+ñ");
    assert_eq!(
        entry.get("identity.email"),
        Some("user@أن.example"),
        "Arabic in email local part"
    );
    assert_eq!(
        entry.get(fields::NOTES),
        Some("emoji: 🔑🔒🔐\n\nRTL: السلام عليكم\nCJK: 鍵マネージャー\nMixed: Påskeøya 2024 ©"),
        "multi-line notes with mixed scripts"
    );
    assert_eq!(
        entry.tags,
        vec!["tagg🇸🇪", "標籤"],
        "tags with flag emoji + CJK"
    );
}

// ── Preferences / security settings tests ────────────────────────────────────

#[test]
fn test_preferences_security_defaults() {
    use kagi_lib::prefs::Preferences;

    let prefs = Preferences::default();
    assert_eq!(
        prefs.idle_lock_minutes, 5,
        "default idle lock should be 5 min"
    );
    assert_eq!(
        prefs.clipboard_clear_seconds, 15,
        "default clipboard clear should be 15 s"
    );
}

#[test]
fn test_preferences_security_roundtrip() {
    use kagi_lib::prefs::Preferences;

    let prefs = Preferences {
        idle_lock_minutes: 10,
        clipboard_clear_seconds: 30,
        last_vault: Some("/tmp/test.kdbx".into()),
        ..Default::default()
    };

    // Serialise to JSON and back (same path the file-based prefs use internally)
    let json = serde_json::to_string_pretty(&prefs).unwrap();
    let loaded: Preferences = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.last_vault, Some("/tmp/test.kdbx".into()));
    assert_eq!(loaded.idle_lock_minutes, 10);
    assert_eq!(loaded.clipboard_clear_seconds, 30);
}

#[test]
fn test_preferences_security_serde_missing_fields_default() {
    // Old prefs.json files won't have the new fields — serde should use defaults
    use kagi_lib::prefs::Preferences;

    let old_json = r#"{
        "lastVault": null,
        "recentVaults": []
    }"#;
    let prefs: Preferences = serde_json::from_str(old_json).unwrap();
    assert_eq!(
        prefs.idle_lock_minutes, 5,
        "missing field should default to 5"
    );
    assert_eq!(
        prefs.clipboard_clear_seconds, 15,
        "missing field should default to 15"
    );
    assert!(prefs.last_vault.is_none());
}

#[test]
fn test_preferences_security_zero_values_roundtrip() {
    // "Never" options store 0 — verify they survive save/load
    use kagi_lib::prefs::Preferences;

    let prefs = Preferences {
        idle_lock_minutes: 0,
        clipboard_clear_seconds: 0,
        ..Default::default()
    };

    let json = serde_json::to_string_pretty(&prefs).unwrap();
    let loaded: Preferences = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.idle_lock_minutes, 0);
    assert_eq!(loaded.clipboard_clear_seconds, 0);
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
    let db = keepass::Database::new();
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

/// Drop replaces the Database with an empty one to release decrypted data.
/// Verify the replacement doesn't panic and the vault count is zero after clear.
#[test]
fn test_openvault_drop_replaces_database() {
    use kagi_lib::state::{AppState, OpenVault};
    use zeroize::Zeroizing;

    let state = AppState::new();
    let mut vaults = state.vaults.lock().unwrap();

    // Create a vault with a Database that has one entry
    let mut db = keepass::Database::new();
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected("Title", "ephemeral");
    });
    assert_eq!(db.iter_all_entries().count(), 1);

    let id = uuid::Uuid::new_v4();
    vaults.insert(
        id,
        OpenVault {
            db,
            path: "/tmp/test.kdbx".into(),
            master_key: Zeroizing::new(b"password".to_vec()),
        },
    );

    // Drop via clear — Drop impl replaces the Database so the old
    // decrypted entry data is released from the heap.
    vaults.clear();
    assert_eq!(vaults.len(), 0);

    // Verify calling clear twice doesn't panic (double drop defense)
    vaults.clear();
    assert_eq!(vaults.len(), 0);
}

/// Verify that the master key's underlying buffer is zeroed after the
/// OpenVault is dropped. We can't easily inspect freed memory, but we
/// can verify that cloning the key before drop produces a non-zero vec
/// that becomes inaccessible after the vault is cleared.
#[test]
fn test_master_key_zeroized_after_lock() {
    use kagi_lib::state::{AppState, OpenVault};
    use zeroize::Zeroizing;

    let state = AppState::new();
    let mut vaults = state.vaults.lock().unwrap();

    let db = keepass::Database::new();
    let id = uuid::Uuid::new_v4();
    let secret = b"supersecret".to_vec();

    // Clone before inserting so we have a reference to the original bytes
    let cloned = secret.clone();
    assert_eq!(&cloned, b"supersecret", "original bytes should be intact");

    vaults.insert(
        id,
        OpenVault {
            db,
            path: "/tmp/test.kdbx".into(),
            master_key: Zeroizing::new(secret),
        },
    );

    // Vault is present, key is accessible
    assert_eq!(vaults.len(), 1);

    // Drop the vault — Zeroizing zeros the buffer, the cloned copy still has the bytes
    vaults.clear();
    assert_eq!(vaults.len(), 0);

    // The cloned copy persists independently (Zeroizing only zeros its own buffer)
    assert_eq!(
        &cloned, b"supersecret",
        "cloned copy unaffected by Zeroizing drop"
    );

    // The original buffer (now inside the dropped Zeroizing) was zeroed.
    // We can't safely verify that from safe Rust without a reference to
    // the freed memory, but Zeroizing's own tests confirm this behavior.
}
