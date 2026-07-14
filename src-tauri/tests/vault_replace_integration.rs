//! Integration tests for the single-vault invariant: opening or creating a
//! vault replaces any previously open one.
//!
//! The app treats `AppState.vaults` as a single-vault map — every read
//! command does `vaults.iter().next()`. Opening a second vault without
//! clearing the first leaks the old vault's entries through, since
//! `HashMap::iter().next()` returns an arbitrary entry. These tests pin the
//! fix: `vault_open` / `vault_create` clear the map before inserting.

use std::io::Cursor;

use kagi_lib::commands::entries::build_entry_summaries;
use kagi_lib::commands::vault::{vault_create, vault_open};
use kagi_lib::models::EntrySummary;
use kagi_lib::state::AppState;
use keepass::db::fields;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::Manager;

const PW: &str = "integration-test-pw";

/// Summaries for the open vault, straight from the in-memory db. Stand-in for
/// the removed `entries_list` command.
fn entries_list(state: tauri::State<'_, AppState>) -> Vec<EntrySummary> {
    let vaults = state.vaults.lock();
    let (_id, vault) = vaults.iter().next().expect("no open vault");
    build_entry_summaries(&vault.db)
}

/// A self-cleaning temp dir holding two distinct vault files.
struct Fixture {
    dir: std::path::PathBuf,
    /// Vault A: 1 entry titled "A1".
    path_a: std::path::PathBuf,
    /// Vault B: 22 entries titled "B1".."B22".
    path_b: std::path::PathBuf,
    _app: tauri::App<tauri::test::MockRuntime>,
}

impl Fixture {
    fn state(&self) -> tauri::State<'_, AppState> {
        self._app.state::<AppState>()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Write a KDBX file with `count` entries titled `B{i}` (for i in 1..=count).
fn write_vault(path: &std::path::Path, count: usize) {
    let mut db = keepass::Database::new();
    for i in 1..=count {
        db.root_mut().add_entry().edit(|e| {
            e.set_unprotected(fields::TITLE, format!("B{i}"));
        });
    }
    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, keepass::DatabaseKey::new().with_password(PW))
        .unwrap();
    std::fs::write(path, buf.into_inner()).unwrap();
}

/// Build a KDBX file with a single entry titled `title`.
fn write_vault_one(path: &std::path::Path, title: &str) {
    let mut db = keepass::Database::new();
    db.root_mut().add_entry().edit(|e| {
        e.set_unprotected(fields::TITLE, title);
    });
    let mut buf = Cursor::new(Vec::new());
    db.save(&mut buf, keepass::DatabaseKey::new().with_password(PW))
        .unwrap();
    std::fs::write(path, buf.into_inner()).unwrap();
}

fn setup() -> Fixture {
    let dir = std::env::temp_dir().join(format!("kagi-vault-replace-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path_a = dir.join("a.kdbx");
    let path_b = dir.join("b.kdbx");
    write_vault_one(&path_a, "A1");
    write_vault(&path_b, 22);

    let app = mock_builder()
        .manage(AppState::new())
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app");

    Fixture {
        dir,
        path_a,
        path_b,
        _app: app,
    }
}

#[tokio::test]
async fn opening_a_second_vault_replaces_the_first() {
    let fx = setup();
    let state = fx.state();

    // Open A first.
    let meta_a = vault_open(
        state.clone(),
        fx.path_a.to_string_lossy().to_string(),
        PW.to_string(),
    )
    .await
    .expect("open A should succeed");
    assert_eq!(meta_a.item_count, 1);
    assert_eq!(entries_list(state.clone()).len(), 1);

    // Now open B in the same app state — without the fix, the stale vault A
    // would remain in the map and entries_list could return A's single entry.
    let meta_b = vault_open(
        state.clone(),
        fx.path_b.to_string_lossy().to_string(),
        PW.to_string(),
    )
    .await
    .expect("open B should succeed");
    assert_eq!(meta_b.item_count, 22);

    // The list must reflect B, not A.
    let summaries = entries_list(state.clone());
    assert_eq!(
        summaries.len(),
        22,
        "entries_list should reflect the newly opened vault"
    );
    assert!(
        summaries.iter().any(|s| s.title == "B1"),
        "expected entries from vault B, got {:?}",
        summaries.iter().map(|s| &s.title).collect::<Vec<_>>()
    );
    assert!(
        !summaries.iter().any(|s| s.title == "A1"),
        "stale entry from vault A leaked into the new vault's list"
    );

    // Only one vault should be resident.
    let vaults = state.vaults.lock();
    assert_eq!(
        vaults.len(),
        1,
        "exactly one vault should be open after replacement"
    );
}

#[tokio::test]
async fn creating_a_vault_replaces_an_open_one() {
    let fx = setup();
    let state = fx.state();

    // Open A first (1 entry).
    vault_open(
        state.clone(),
        fx.path_a.to_string_lossy().to_string(),
        PW.to_string(),
    )
    .await
    .unwrap();
    assert_eq!(entries_list(state.clone()).len(), 1);

    // Create a brand-new vault at a third path — must replace A.
    let path_c = fx.dir.join("c.kdbx");
    let meta = vault_create(
        state.clone(),
        path_c.to_string_lossy().to_string(),
        PW.to_string(),
        String::new(),
    )
    .await
    .expect("create should succeed");
    assert_eq!(meta.item_count, 0);

    // New vault is empty; A's entry must not leak through.
    let summaries = entries_list(state.clone());
    assert_eq!(summaries.len(), 0, "newly created vault should be empty");
    assert!(
        !summaries.iter().any(|s| s.title == "A1"),
        "stale entry from the previous vault leaked into the new one"
    );

    let vaults = state.vaults.lock();
    assert_eq!(
        vaults.len(),
        1,
        "exactly one vault should be open after create"
    );
}
