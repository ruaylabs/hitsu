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
    let mut em = db.entry_mut(entry_id).unwrap();
    em.fields.remove(fields::URL);
    em.fields.remove(fields::NOTES);
    drop(em);

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
    let mut em = db.entry_mut(entry_id).unwrap();
    em.fields.remove("identity.firstName");
    em.fields.remove("identity.email");
    drop(em);

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

        let mut em = db.entry_mut(login_id).unwrap();
        em.edit_tracking(|e| {
            e.set_unprotected(fields::PASSWORD, "new_secret");
        });
        em.times.last_modification = Some(chrono::Utc::now().naive_utc());
        drop(em);

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

    let mut prefs = Preferences::default();
    prefs.idle_lock_minutes = 10;
    prefs.clipboard_clear_seconds = 30;
    prefs.last_vault = Some("/tmp/test.kdbx".into());

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

    let mut prefs = Preferences::default();
    prefs.idle_lock_minutes = 0;
    prefs.clipboard_clear_seconds = 0;

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
