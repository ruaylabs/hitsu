//! Integration tests for the entry command workflow.
//!
//! These tests exercise the real `#[tauri::command]` functions through
//! Tauri's actual state management. A headless app is built with
//! `tauri::test::mock_builder()` (no webview, no IPC), the real `AppState`
//! is managed, and a `State<'_, AppState>` is obtained via `Manager::state`
//! — the same handle Tauri would inject into a command. The command
//! functions are then awaited directly, so the full code path (mutex lock,
//! in-memory db mutation, `save_vault` → `atomic_write` → disk) runs exactly
//! as in production.
//!
//! No production source is modified to make this testable beyond exposing the
//! `commands` and `models` modules (`pub mod`) from the library root; the
//! command functions themselves were already `pub`.

use std::io::Cursor;

use kagi_lib::commands::entries::{
    entries_list, entry_create, entry_delete, entry_discard, entry_get, entry_update,
};
use kagi_lib::models::{EntryDraft, EntryPatch};
use kagi_lib::state::{AppState, OpenVault};
use keepass::db::Value;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::Manager;

const TEST_PW: &str = "integration-test-pw";

/// A self-cleaning temp directory holding the test vault's KDBX file.
struct TestVault {
    dir: std::path::PathBuf,
    path: std::path::PathBuf,
    /// The live `State` handle to the app's managed `AppState`. Kept here so
    /// the helper can hand it to command calls; it borrows from `app`, whose
    /// lifetime is tied to the `App` stored alongside via `_app`.
    _app: tauri::App<tauri::test::MockRuntime>,
}

impl TestVault {
    /// Insert an `OpenVault` (freshly created empty KDBX on disk) into the
    /// managed state and return the vault id plus a `State` handle.
    fn state(&self) -> tauri::State<'_, AppState> {
        self._app.state::<AppState>()
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Re-open the on-disk file to inspect exactly what was persisted.
    fn reload_disk(&self) -> keepass::Database {
        let key = keepass::DatabaseKey::new().with_password(TEST_PW);
        let mut file = std::fs::File::open(self.path()).unwrap();
        keepass::Database::open(&mut file, key).unwrap()
    }

    fn disk_entry_count(&self) -> usize {
        self.reload_disk().iter_all_entries().count()
    }
}

impl Drop for TestVault {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Build a headless app with a managed `AppState` containing one fresh vault.
fn setup() -> TestVault {
    let dir = std::env::temp_dir().join(format!("kagi-it-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("vault.kdbx");

    // Create an empty KDBX4 file on disk.
    let key = keepass::DatabaseKey::new().with_password(TEST_PW);
    let db = keepass::Database::new();
    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, key).unwrap();
    std::fs::write(&path, buf.into_inner()).unwrap();

    // Open it into an `OpenVault` and insert into managed state.
    let key = keepass::DatabaseKey::new().with_password(TEST_PW);
    let mut file = std::fs::File::open(&path).unwrap();
    let db = keepass::Database::open(&mut file, key).unwrap();
    let open_vault = OpenVault {
        db,
        path: path.clone(),
        db_key: keepass::DatabaseKey::new().with_password(TEST_PW),
    };

    let app = mock_builder()
        .manage(AppState::new())
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app");

    {
        let state = app.state::<AppState>();
        let mut vaults = state.vaults.lock().unwrap();
        vaults.insert(uuid::Uuid::new_v4(), open_vault);
    }

    TestVault {
        dir,
        path,
        _app: app,
    }
}

fn draft(title: &str) -> EntryDraft {
    EntryDraft {
        title: title.to_string(),
        username: None,
        password: None,
        url: None,
        notes: None,
        totp: None,
    }
}

// ── create: in-memory only, never written to disk ─────────────────────────

#[tokio::test]
async fn create_adds_to_memory_not_to_disk() {
    let tv = setup();
    let state = tv.state();

    let entry = entry_create(state.clone(), "login".to_string(), draft("New login"))
        .await
        .expect("create should succeed");

    // Visible through the command layer (in-memory db)…
    assert_eq!(entries_list(state.clone()).await.unwrap().len(), 1);
    assert!(entry_get(state.clone(), entry.id.clone()).await.is_ok());

    // …but the on-disk file still has zero entries (create does not save).
    assert_eq!(tv.disk_entry_count(), 0, "create must not touch disk");
}

// ── discard: drops from memory without persisting ─────────────────────────

#[tokio::test]
async fn discard_drops_from_memory_without_persisting() {
    let tv = setup();
    let state = tv.state();

    let entry = entry_create(state.clone(), "login".to_string(), draft("New login"))
        .await
        .unwrap();

    entry_discard(state.clone(), entry.id.clone())
        .await
        .expect("discard should succeed");

    assert_eq!(entries_list(state.clone()).await.unwrap().len(), 0);
    assert!(entry_get(state.clone(), entry.id.clone()).await.is_err());
    assert_eq!(tv.disk_entry_count(), 0);
}

#[tokio::test]
async fn discard_nonexistent_entry_errors() {
    let tv = setup();
    let state = tv.state();

    let res = entry_discard(state.clone(), "not-a-uuid".to_string()).await;
    assert!(res.is_err());
}

// ── update (Save): persists to disk ───────────────────────────────────────

#[tokio::test]
async fn update_persists_to_disk() {
    let tv = setup();
    let state = tv.state();

    let entry = entry_create(state.clone(), "login".to_string(), draft("New login"))
        .await
        .unwrap();

    let saved = entry_update(
        state.clone(),
        entry.id.clone(),
        EntryPatch {
            title: Some("Saved title".to_string()),
            username: Some("alice".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("update should succeed");

    assert_eq!(saved.title, "Saved title");

    let disk = tv.reload_disk();
    let disk_entry = disk.iter_all_entries().next().expect("one entry on disk");
    assert_eq!(disk_entry.get_title().unwrap_or(""), "Saved title");
    assert_eq!(disk_entry.get_username().unwrap_or(""), "alice");
}

#[tokio::test]
async fn update_nonexistent_entry_errors() {
    let tv = setup();
    let state = tv.state();

    let res = entry_update(
        state.clone(),
        uuid::Uuid::new_v4().to_string(),
        EntryPatch::default(),
    )
    .await;
    assert!(res.is_err());
}

// ── delete: persists removal to disk ───────────────────────────────────────

#[tokio::test]
async fn delete_persists_removal_to_disk() {
    let tv = setup();
    let state = tv.state();

    let entry = entry_create(state.clone(), "login".to_string(), draft("New login"))
        .await
        .unwrap();
    // Persist first so there's something on disk to delete.
    entry_update(
        state.clone(),
        entry.id.clone(),
        EntryPatch {
            title: Some("Saved".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(tv.disk_entry_count(), 1);

    entry_delete(state.clone(), entry.id.clone())
        .await
        .expect("delete should succeed");

    assert_eq!(entries_list(state.clone()).await.unwrap().len(), 0);
    assert_eq!(tv.disk_entry_count(), 0);
}

// ── the leak the frontend guards against ──────────────────────────────────

#[tokio::test]
async fn unsaved_stub_is_included_in_a_subsequent_save() {
    // Documents the backend contract that motivates the frontend's
    // auto-discard: an unsaved stub lives in the in-memory db, so a later
    // save of ANY entry persists the whole db — including the abandoned
    // stub. The frontend therefore discards the stub before any other save.
    let tv = setup();
    let state = tv.state();

    let _stub = entry_create(state.clone(), "login".to_string(), draft("Abandoned"))
        .await
        .unwrap();

    let other = entry_create(state.clone(), "login".to_string(), draft("Other"))
        .await
        .unwrap();
    entry_update(
        state.clone(),
        other.id.clone(),
        EntryPatch {
            title: Some("Other saved".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Both the saved entry and the abandoned stub are now on disk.
    assert_eq!(tv.disk_entry_count(), 2);
}

#[tokio::test]
async fn discard_before_save_prevents_leak() {
    // The correct workflow the frontend follows: discard the unsaved stub,
    // THEN save another entry. The stub must not appear on disk.
    let tv = setup();
    let state = tv.state();

    let stub = entry_create(state.clone(), "login".to_string(), draft("Abandoned"))
        .await
        .unwrap();
    entry_discard(state.clone(), stub.id.clone()).await.unwrap();

    let other = entry_create(state.clone(), "login".to_string(), draft("Other"))
        .await
        .unwrap();
    entry_update(
        state.clone(),
        other.id.clone(),
        EntryPatch {
            title: Some("Other saved".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let disk = tv.reload_disk();
    assert_eq!(disk.iter_all_entries().count(), 1);
    assert_eq!(
        disk.iter_all_entries()
            .next()
            .unwrap()
            .get_title()
            .unwrap_or(""),
        "Other saved"
    );
}

// ── end-to-end happy path ─────────────────────────────────────────────────

#[tokio::test]
async fn full_create_save_get_delete_workflow() {
    let tv = setup();
    let state = tv.state();

    // 1. Create (in-memory only)
    let entry = entry_create(state.clone(), "login".to_string(), draft("New login"))
        .await
        .unwrap();
    assert_eq!(tv.disk_entry_count(), 0);

    // 2. Save
    let saved = entry_update(
        state.clone(),
        entry.id.clone(),
        EntryPatch {
            title: Some("My Login".to_string()),
            username: Some("bob".to_string()),
            password: Some("s3cret".to_string()),
            url: Some("https://example.com".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(saved.title, "My Login");
    assert_eq!(tv.disk_entry_count(), 1);

    // 3. Get reflects saved values
    let loaded = entry_get(state.clone(), entry.id.clone()).await.unwrap();
    assert_eq!(loaded.title, "My Login");
    assert_eq!(loaded.username.as_deref(), Some("bob"));
    assert_eq!(loaded.password.as_deref(), Some("s3cret"));
    assert_eq!(loaded.url.as_deref(), Some("https://example.com"));

    // 4. List shows one entry
    assert_eq!(entries_list(state.clone()).await.unwrap().len(), 1);

    // 5. Delete removes it from memory and disk
    entry_delete(state.clone(), entry.id.clone()).await.unwrap();
    assert_eq!(entries_list(state.clone()).await.unwrap().len(), 0);
    assert!(entry_get(state.clone(), entry.id.clone()).await.is_err());
    assert_eq!(tv.disk_entry_count(), 0);
}

#[tokio::test]
async fn discard_after_save_still_on_disk() {
    // After an entry has been saved, discarding it only removes it from the
    // in-memory db; it reappears when reloaded from disk. This documents the
    // contract that `entry_discard` is only for never-persisted stubs.
    let tv = setup();
    let state = tv.state();

    let entry = entry_create(state.clone(), "login".to_string(), draft("New login"))
        .await
        .unwrap();
    entry_update(
        state.clone(),
        entry.id.clone(),
        EntryPatch {
            title: Some("Saved".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(tv.disk_entry_count(), 1);

    entry_discard(state.clone(), entry.id.clone())
        .await
        .unwrap();
    assert_eq!(entries_list(state.clone()).await.unwrap().len(), 0);

    // Still on disk; reload sees it again.
    let disk = tv.reload_disk();
    assert_eq!(disk.iter_all_entries().count(), 1);
}

#[tokio::test]
async fn saved_password_is_protected_on_disk() {
    // Regression guard: the password must round-trip as a Protected value
    // (kept) and be readable back through entry_get.
    let tv = setup();
    let state = tv.state();

    let entry = entry_create(
        state.clone(),
        "login".to_string(),
        EntryDraft {
            title: "Bank".to_string(),
            username: Some("u".to_string()),
            password: Some("p4ssw0rd".to_string()),
            ..draft("")
        },
    )
    .await
    .unwrap();
    entry_update(
        state.clone(),
        entry.id.clone(),
        EntryPatch {
            password: Some("hunter2".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let loaded = entry_get(state.clone(), entry.id.clone()).await.unwrap();
    assert_eq!(loaded.password.as_deref(), Some("hunter2"));

    // And the raw on-disk field is the protected password value.
    let disk = tv.reload_disk();
    let de = disk.iter_all_entries().next().unwrap();
    match de.fields.get(keepass::db::fields::PASSWORD) {
        Some(Value::Protected(_)) => {}
        other => panic!("expected Protected password on disk, got {:?}", other),
    }
}
