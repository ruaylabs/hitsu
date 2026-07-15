use keepass::{Database, DatabaseKey};
// parking_lot::Mutex cannot be poisoned: a panic while holding the lock
// simply releases it, so commands don't need per-call poison handling.
use parking_lot::{Condvar, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub type VaultId = Uuid;

pub struct OpenVault {
    pub db: Database,
    pub path: PathBuf,
    pub db_key: DatabaseKey,
    /// SHA-256 of the raw master-password bytes, stored for constant-time
    /// verification on password change.
    pub password_hash: [u8; 32],
    /// SHA-256 of the vault file's bytes as we last read or wrote them.
    /// Checked before every save to detect external modification (sync
    /// clients, other KeePass apps) instead of silently clobbering it.
    pub disk_hash: [u8; 32],
}

// Zeroize sensitive key material when the vault is dropped (lock, close, …)
impl Drop for OpenVault {
    fn drop(&mut self) {
        // DatabaseKey implements ZeroizeOnDrop — its password field is zeroized
        // automatically when the struct is dropped.
        // Replace the Database with an empty one so decrypted entry data is
        // released from the heap. Note: the allocator may not immediately
        // overwrite the freed pages — a proper scrub would require the keepass
        // crate to implement Zeroize internally.
        self.db = keepass::Database::new();
        // Scrub the cached password hash.
        self.password_hash.iter_mut().for_each(|b| *b = 0);
        // Path is not sensitive; no need to scrub.
    }
}

struct IdleLockState {
    timeout: Option<Duration>,
    last_activity: Instant,
    armed: bool,
}

pub struct AppState {
    pub vaults: Mutex<HashMap<VaultId, OpenVault>>,
    /// Serializes all vault-file writers (entry update/delete, password
    /// change, KDF upgrade). Writers snapshot the database under a brief
    /// `vaults` lock and run KDF + serialize + fsync on a blocking thread;
    /// holding this lock across snapshot *and* write keeps saves from
    /// hitting the disk out of order.
    ///
    /// Lock ordering: acquire `save_lock` BEFORE `vaults`, and never await
    /// while holding `vaults` (it is a sync mutex).
    pub save_lock: tokio::sync::Mutex<()>,
    idle_lock: Mutex<IdleLockState>,
    idle_lock_changed: Condvar,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            vaults: Mutex::new(HashMap::new()),
            save_lock: tokio::sync::Mutex::new(()),
            idle_lock: Mutex::new(IdleLockState {
                timeout: None,
                last_activity: Instant::now(),
                armed: false,
            }),
            idle_lock_changed: Condvar::new(),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply the persisted idle-lock preference. Zero disables the watchdog.
    /// Changing the preference counts as activity and starts a fresh deadline.
    pub fn configure_idle_lock(&self, minutes: u32) {
        let mut idle = self.idle_lock.lock();
        idle.timeout = (minutes > 0).then(|| Duration::from_secs(u64::from(minutes) * 60));
        idle.last_activity = Instant::now();
        self.idle_lock_changed.notify_all();
    }

    /// Arm the watchdog after a vault has been opened successfully.
    pub fn arm_idle_lock(&self) {
        let mut idle = self.idle_lock.lock();
        idle.armed = true;
        idle.last_activity = Instant::now();
        self.idle_lock_changed.notify_all();
    }

    /// Refresh the watchdog deadline for a backend IPC command.
    pub fn reset_idle_lock(&self) {
        let mut idle = self.idle_lock.lock();
        if idle.armed {
            idle.last_activity = Instant::now();
            self.idle_lock_changed.notify_all();
        }
    }

    /// Drop all decrypted vault state and disarm the watchdog.
    pub fn lock_open_vaults(&self) {
        let mut idle = self.idle_lock.lock();
        idle.armed = false;
        self.idle_lock_changed.notify_all();

        // Keep the idle lock until the vault is cleared so an IPC reset cannot
        // race with a timeout that has already committed to locking. The global
        // lock order is idle_lock before vaults.
        self.vaults.lock().clear();
    }

    /// Block until an armed deadline expires, then atomically drop the vault.
    /// Called only by the process-long watchdog thread.
    pub(crate) fn wait_for_idle_timeout_and_lock(&self) {
        let mut idle = self.idle_lock.lock();
        loop {
            let Some(timeout) = idle.timeout.filter(|_| idle.armed) else {
                self.idle_lock_changed.wait(&mut idle);
                continue;
            };

            let remaining = timeout.saturating_sub(idle.last_activity.elapsed());
            if remaining.is_zero() {
                // Hold idle_lock while acquiring vaults: a command that arrived
                // before the deadline can reset first; one arriving after this
                // point observes the vault as locked.
                self.vaults.lock().clear();
                idle.armed = false;
                return;
            }

            self.idle_lock_changed.wait_for(&mut idle, remaining);
        }
    }

    /// Record the on-disk hash after a successful save. No-op if the vault
    /// was locked or swapped for a different file while the save ran.
    pub fn commit_disk_hash(&self, path: &std::path::Path, hash: [u8; 32]) {
        let mut vaults = self.vaults.lock();
        if let Some((_id, vault)) = vaults.iter_mut().next() {
            if vault.path == path {
                vault.disk_hash = hash;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{mpsc, Arc};
    use std::thread;

    use super::*;

    fn open_test_vault(state: &AppState) {
        state.vaults.lock().insert(
            Uuid::new_v4(),
            OpenVault {
                db: Database::new(),
                path: PathBuf::from("test.kdbx"),
                db_key: DatabaseKey::new().with_password("test-password"),
                password_hash: [0; 32],
                disk_hash: [0; 32],
            },
        );
    }

    fn set_test_timeout(state: &AppState, timeout: Duration) {
        state.idle_lock.lock().timeout = Some(timeout);
    }

    #[test]
    fn watchdog_drops_an_open_vault_after_timeout() {
        let state = Arc::new(AppState::new());
        set_test_timeout(&state, Duration::from_millis(20));
        open_test_vault(&state);
        state.arm_idle_lock();

        let worker_state = Arc::clone(&state);
        let worker = thread::spawn(move || worker_state.wait_for_idle_timeout_and_lock());
        worker.join().unwrap();

        assert!(state.vaults.lock().is_empty());
        assert!(!state.idle_lock.lock().armed);
    }

    #[test]
    fn ipc_activity_refreshes_the_watchdog_deadline() {
        let state = Arc::new(AppState::new());
        set_test_timeout(&state, Duration::from_millis(100));
        open_test_vault(&state);
        state.arm_idle_lock();

        let worker_state = Arc::clone(&state);
        let (done_tx, done_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            worker_state.wait_for_idle_timeout_and_lock();
            done_tx.send(()).unwrap();
        });

        thread::sleep(Duration::from_millis(30));
        state.reset_idle_lock();
        assert!(done_rx.recv_timeout(Duration::from_millis(60)).is_err());
        done_rx.recv_timeout(Duration::from_millis(100)).unwrap();
        worker.join().unwrap();
        assert!(state.vaults.lock().is_empty());
    }

    #[test]
    fn zero_minute_preference_disables_the_deadline() {
        let state = AppState::new();
        state.configure_idle_lock(5);
        assert_eq!(
            state.idle_lock.lock().timeout,
            Some(Duration::from_secs(300))
        );

        state.configure_idle_lock(0);
        assert_eq!(state.idle_lock.lock().timeout, None);
    }
}
