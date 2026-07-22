//! Narrow local IPC API used by the browser native-messaging host.
//!
//! The integration is opt-in (Settings → Features, off by default): nothing
//! listens and no native-host manifest is registered until the user enables
//! it, since the socket widens the local attack surface described below.
//!
//! The Unix socket is owner-only, but on a shared machine "owner-only" means
//! *every* process running as the same user — not just our native host. To
//! keep an unrelated same-user process from blindly speaking this protocol and
//! harvesting credentials, each request must carry a per-session token: the app
//! writes a fresh random token to an owner-only file at startup, the native
//! host (which the browser launches from a manifest we control) reads that file and
//! injects the token into every request, and the backend verifies it in
//! constant time before doing any work. This is not a hard boundary against a
//! same-user attacker — such a process can also read the 0600 token file — but
//! it stops naive enumeration by anything that doesn't know to look.
//!
//! On top of that, requests are restricted to exact HTTP(S) host matches, and
//! secret values are returned only for an explicitly chosen entry while the
//! vault is unlocked.

#![cfg(unix)]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use parking_lot::Mutex;
use rand::RngCore;
use serde::Deserialize;
use serde_json::{json, Value};
use subtle::ConstantTimeEq;
use tauri::{AppHandle, Manager};
use url::Url;

use crate::state::AppState;
use crate::vault::atomic_write;

const MAX_REQUEST_BYTES: u64 = 1024 * 1024;
/// Native-messaging requests are written immediately after connecting. Bound
/// idle reads so a client that never sends a newline cannot retain a worker
/// indefinitely.
const CONNECTION_READ_TIMEOUT: Duration = Duration::from_secs(5);
const NATIVE_HOST_NAME: &str = "com.ruaylabs.hitsu.browser";
const PRODUCTION_EXTENSION_ID: &str = "pkickpkkbgpaffpdloplecfleckoopjc";
const FIREFOX_EXTENSION_ID: &str = "hitsu@ruaylabs.com";

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
enum BrowserRequest {
    ListLogins {
        token: String,
        origin: String,
    },
    GetCredentials {
        token: String,
        id: String,
        origin: String,
    },
}

impl BrowserRequest {
    /// The per-session token injected by the native host. Verified against the
    /// token this process wrote to disk before any request is processed.
    fn token(&self) -> &str {
        match self {
            BrowserRequest::ListLogins { token, .. }
            | BrowserRequest::GetCredentials { token, .. } => token,
        }
    }
}

/// Directory holding the browser IPC socket and token file.
///
/// Prefers `$XDG_RUNTIME_DIR` (per-user, mode 0700, cleaned on logout): the
/// temp dir on Linux is world-writable `/tmp`, where the sticky bit stops other
/// users from *removing* our socket but not from pre-binding its predictable
/// path before we start. Falls back to the temp dir when the variable is unset
/// or doesn't hold what the XDG spec promises — on macOS that fallback is
/// `$TMPDIR`, itself per-user and 0700.
///
/// The native host resolves this identically (`chrome-extension/native-host`);
/// keep the two in sync or the host will look for the socket in the wrong place.
fn runtime_dir() -> PathBuf {
    runtime_dir_from(std::env::var_os("XDG_RUNTIME_DIR"))
}

fn runtime_dir_from(candidate: Option<std::ffi::OsString>) -> PathBuf {
    if let Some(dir) = candidate.map(PathBuf::from) {
        let owned_by_us = dir.metadata().is_ok_and(|metadata| {
            metadata.is_dir() && metadata.uid() == unsafe { libc::geteuid() }
        });
        if dir.is_absolute() && owned_by_us {
            return dir;
        }
    }
    std::env::temp_dir()
}

pub fn socket_path() -> PathBuf {
    runtime_dir().join(format!("hitsu-browser-{}.sock", unsafe { libc::geteuid() }))
}

/// Owner-only file holding this session's browser-IPC token. Sits beside the
/// socket; the native host reads it to authenticate each request.
fn token_path() -> PathBuf {
    runtime_dir().join(format!("hitsu-browser-{}.token", unsafe {
        libc::geteuid()
    }))
}

/// Generate a fresh 256-bit session token, URL-safe base64 encoded.
fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Constant-time token comparison. A length mismatch only reveals our token's
/// (fixed, public) length, so short-circuiting on it is fine.
fn token_matches(provided: &str, expected: &str) -> bool {
    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}

fn native_host_manifest_directories() -> std::io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME is not set"))?;

    #[cfg(target_os = "macos")]
    let chromium = [
        "Library/Application Support/Google/Chrome/NativeMessagingHosts",
        "Library/Application Support/Chromium/NativeMessagingHosts",
        "Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts",
        "Library/Application Support/Microsoft Edge/NativeMessagingHosts",
    ]
    .into_iter()
    .map(|relative| home.join(relative))
    .collect();

    #[cfg(target_os = "macos")]
    let firefox = vec![home.join("Library/Application Support/Mozilla/NativeMessagingHosts")];

    #[cfg(not(target_os = "macos"))]
    let (chromium, firefox) = {
        let config_home = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));
        let chromium = [
            "google-chrome/NativeMessagingHosts",
            "chromium/NativeMessagingHosts",
            "BraveSoftware/Brave-Browser/NativeMessagingHosts",
            "microsoft-edge/NativeMessagingHosts",
        ]
        .into_iter()
        .map(|relative| config_home.join(relative))
        .collect();
        let firefox = vec![home.join(".mozilla/native-messaging-hosts")];
        (chromium, firefox)
    };

    Ok((chromium, firefox))
}

fn write_native_host_manifests(
    host_path: &Path,
    chromium_directories: &[PathBuf],
    firefox_directories: &[PathBuf],
    chromium_extension_id: &str,
    firefox_extension_id: &str,
) -> std::io::Result<()> {
    let host_path = host_path.to_str().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "native host path is not valid UTF-8",
        )
    })?;
    let base = json!({
        "name": NATIVE_HOST_NAME,
        "description": "Hitsu Password Manager native messaging host",
        "path": host_path,
        "type": "stdio",
    });
    let mut chromium_manifest = base.clone();
    chromium_manifest["allowed_origins"] =
        json!([format!("chrome-extension://{chromium_extension_id}/")]);
    let chromium_manifest =
        serde_json::to_vec_pretty(&chromium_manifest).map_err(std::io::Error::other)?;

    let mut firefox_manifest = base;
    firefox_manifest["allowed_extensions"] = json!([firefox_extension_id]);
    let firefox_manifest =
        serde_json::to_vec_pretty(&firefox_manifest).map_err(std::io::Error::other)?;

    for (directories, manifest) in [
        (chromium_directories, chromium_manifest.as_slice()),
        (firefox_directories, firefox_manifest.as_slice()),
    ] {
        for directory in directories {
            fs::create_dir_all(directory)?;
            let destination = directory.join(format!("{NATIVE_HOST_NAME}.json"));
            atomic_write(&destination, manifest)?;
        }
    }
    Ok(())
}

pub fn register_production_native_host() -> std::io::Result<()> {
    let executable = std::env::current_exe()?;
    let host_path = executable
        .parent()
        .ok_or_else(|| std::io::Error::other("application executable has no parent directory"))?
        .join("hitsu-native-host");
    if !host_path.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("native messaging host not found at {}", host_path.display()),
        ));
    }
    let (chromium_directories, firefox_directories) = native_host_manifest_directories()?;
    write_native_host_manifests(
        &host_path,
        &chromium_directories,
        &firefox_directories,
        PRODUCTION_EXTENSION_ID,
        FIREFOX_EXTENSION_ID,
    )
}

pub struct BrowserIpcSocket {
    path: PathBuf,
    token_path: PathBuf,
    shutdown: Arc<AtomicBool>,
}

impl Drop for BrowserIpcSocket {
    fn drop(&mut self) {
        // Wake the blocked accept loop so its thread observes the flag and
        // exits (closing the listener), then remove the socket and token.
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = UnixStream::connect(&self.path);
        let _ = fs::remove_file(&self.path);
        let _ = fs::remove_file(&self.token_path);
    }
}

/// Managed handle to the browser IPC listener. Holds `None` while the
/// integration is disabled (the developer-preview default), so the Settings
/// toggle can start and stop it without an app restart.
pub struct BrowserIpc(pub Mutex<Option<BrowserIpcSocket>>);

/// Start or stop the browser integration to match the preference. Errors are
/// only possible when starting; stopping just drops the listener handle.
pub fn set_enabled(app: &AppHandle, enabled: bool) -> std::io::Result<()> {
    let ipc = app.state::<BrowserIpc>();
    let mut guard = ipc.0.lock();
    if !enabled {
        *guard = None;
        return Ok(());
    }
    if guard.is_none() {
        #[cfg(not(debug_assertions))]
        if let Err(error) = register_production_native_host() {
            eprintln!("native host registration unavailable: {error}");
        }
        *guard = Some(start(app.clone())?);
    }
    Ok(())
}

fn remove_stale_socket(path: &std::path::Path) -> std::io::Result<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if !metadata.file_type().is_socket() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "browser IPC path exists and is not a socket",
        ));
    }

    match UnixStream::connect(path) {
        Ok(_) => Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "browser IPC is already running",
        )),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
            ) =>
        {
            if path.exists() {
                fs::remove_file(path)?;
            }
            Ok(())
        }
        Err(error) => Err(error),
    }
}

pub fn start(app: AppHandle) -> std::io::Result<BrowserIpcSocket> {
    let path = socket_path();
    let token_path = token_path();

    remove_stale_socket(&path)?;
    let listener = UnixListener::bind(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;

    // Write the token only after the socket is bound, so a failed start never
    // clobbers a running instance's token (remove_stale_socket already refuses
    // to proceed when another instance holds the socket). atomic_write creates
    // the file 0600.
    let token = generate_token();
    atomic_write(&token_path, token.as_bytes())?;

    let shutdown = Arc::new(AtomicBool::new(false));
    let thread_shutdown = Arc::clone(&shutdown);
    std::thread::Builder::new()
        .name("browser-ipc".into())
        .spawn(move || {
            for stream in listener.incoming() {
                if thread_shutdown.load(Ordering::SeqCst) {
                    break;
                }
                match stream {
                    Ok(stream) => {
                        let connection_app = app.clone();
                        let connection_token = token.clone();
                        if let Err(error) = std::thread::Builder::new()
                            .name("browser-ipc-connection".into())
                            .spawn(move || {
                                if let Err(error) =
                                    stream.set_read_timeout(Some(CONNECTION_READ_TIMEOUT))
                                {
                                    eprintln!("browser IPC timeout setup failed: {error}");
                                    return;
                                }
                                handle_connection(&connection_app, &connection_token, stream);
                            })
                        {
                            eprintln!("browser IPC worker failed to start: {error}");
                        }
                    }
                    Err(error) => eprintln!("browser IPC connection failed: {error}"),
                }
            }
        })?;
    Ok(BrowserIpcSocket {
        path,
        token_path,
        shutdown,
    })
}

fn read_request(stream: &UnixStream) -> Result<Option<BrowserRequest>, String> {
    let mut line = String::new();
    let mut reader = BufReader::new(stream).take(MAX_REQUEST_BYTES);
    match reader.read_line(&mut line) {
        Ok(0) => Ok(None),
        Ok(_) => serde_json::from_str::<BrowserRequest>(&line)
            .map(Some)
            .map_err(|_| "Invalid browser request".to_string()),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ) =>
        {
            Err("Browser request timed out".to_string())
        }
        Err(_) => Err("Could not read browser request".to_string()),
    }
}

fn handle_connection(app: &AppHandle, expected_token: &str, mut stream: UnixStream) {
    let request = match read_request(&stream) {
        Ok(Some(request)) => request,
        Ok(None) => return,
        Err(error) => {
            let response = json!({ "ok": false, "error": error });
            let _ = serde_json::to_writer(&mut stream, &response);
            let _ = stream.write_all(b"\n");
            return;
        }
    };

    // Verify the session token before touching vault state or resetting the
    // idle-lock watchdog — a request that fails authentication is not activity.
    let response = if token_matches(request.token(), expected_token) {
        process_request(app, request)
    } else {
        json!({ "ok": false, "error": "Unauthorized browser request" })
    };
    let _ = serde_json::to_writer(&mut stream, &response);
    let _ = stream.write_all(b"\n");
}

fn process_request(app: &AppHandle, request: BrowserRequest) -> Value {
    let state = app.state::<AppState>();
    // Native-messaging requests are backend IPC too; an actively used browser
    // integration must refresh the same watchdog as webview commands.
    state.reset_idle_lock();
    let vaults = state.vaults.lock();
    let Some((_vault_id, vault)) = vaults.iter().next() else {
        return json!({ "ok": false, "error": "Hitsu is locked" });
    };

    match request {
        BrowserRequest::ListLogins { origin, .. } => {
            let Ok(host) = origin_host(&origin) else {
                return json!({ "ok": false, "error": "Invalid page origin" });
            };
            let mut entries = vault
                .db
                .iter_all_entries()
                .filter(|entry| {
                    !crate::commands::entries::entry_is_trashed(&vault.db, entry)
                        && entry
                            .get_password()
                            .is_some_and(|password| !password.is_empty())
                        && entry
                            .get_url()
                            .and_then(entry_host)
                            .is_some_and(|entry_host| entry_host == host)
                })
                .map(|entry| {
                    json!({
                        "id": entry.id().uuid().to_string(),
                        "title": entry.get_title().unwrap_or(""),
                        "username": entry.get_username().unwrap_or(""),
                    })
                })
                .collect::<Vec<_>>();
            entries.sort_by(|a, b| a["title"].as_str().cmp(&b["title"].as_str()));
            json!({ "ok": true, "entries": entries })
        }
        BrowserRequest::GetCredentials { id, origin, .. } => {
            let Ok(host) = origin_host(&origin) else {
                return json!({ "ok": false, "error": "Invalid page origin" });
            };
            if !valid_entry_id(&id) {
                return json!({ "ok": false, "error": "Invalid entry ID" });
            }
            let Some(entry) = crate::commands::entries::find_entry_ref(&vault.db, &id) else {
                return json!({ "ok": false, "error": "Entry not found" });
            };
            if crate::commands::entries::entry_is_trashed(&vault.db, &entry) {
                return json!({ "ok": false, "error": "Entry not found" });
            }
            let matches_origin = entry
                .get_url()
                .and_then(entry_host)
                .is_some_and(|entry_host| entry_host == host);
            if !matches_origin {
                return json!({ "ok": false, "error": "Entry does not match this site" });
            }
            let Some(password) = entry.get_password().filter(|password| !password.is_empty())
            else {
                return json!({ "ok": false, "error": "Entry has no password" });
            };
            json!({
                "ok": true,
                "username": entry.get_username().unwrap_or(""),
                "password": password,
            })
        }
    }
}

fn valid_entry_id(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok()
}

fn origin_host(origin: &str) -> Result<String, ()> {
    let url = Url::parse(origin).map_err(|_| ())?;
    if !matches!(url.scheme(), "http" | "https") || url.username() != "" || url.password().is_some()
    {
        return Err(());
    }
    url.host_str()
        .map(|host| host.to_ascii_lowercase())
        .ok_or(())
}

fn entry_host(raw: &str) -> Option<String> {
    let value = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };
    Url::parse(&value)
        .ok()?
        .host_str()
        .map(|host| host.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{
        entry_host, generate_token, origin_host, read_request, remove_stale_socket,
        runtime_dir_from, token_matches, valid_entry_id, write_native_host_manifests,
        BrowserRequest, NATIVE_HOST_NAME,
    };
    use std::fs;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::time::{Duration, Instant};

    #[test]
    fn accepts_http_origins_and_bare_entry_urls() {
        assert_eq!(
            origin_host("https://example.com:443"),
            Ok("example.com".into())
        );
        assert_eq!(
            entry_host("example.com/login").as_deref(),
            Some("example.com")
        );
    }

    #[test]
    fn writes_chromium_and_firefox_native_host_manifests() {
        let root = std::env::temp_dir().join(format!("hitsu-host-{}", uuid::Uuid::new_v4()));
        let chromium_directory = root.join("chromium/NativeMessagingHosts");
        let firefox_directory = root.join("firefox/native-messaging-hosts");
        let host_path = root.join("hitsu-native-host");

        write_native_host_manifests(
            &host_path,
            std::slice::from_ref(&chromium_directory),
            std::slice::from_ref(&firefox_directory),
            "chromium-extension-id",
            "firefox-extension-id",
        )
        .unwrap();

        let chromium_manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(chromium_directory.join(format!("{NATIVE_HOST_NAME}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(chromium_manifest["name"], NATIVE_HOST_NAME);
        assert_eq!(chromium_manifest["path"], host_path.to_str().unwrap());
        assert_eq!(
            chromium_manifest["allowed_origins"][0],
            "chrome-extension://chromium-extension-id/"
        );
        assert!(chromium_manifest.get("allowed_extensions").is_none());

        let firefox_manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(firefox_directory.join(format!("{NATIVE_HOST_NAME}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(firefox_manifest["name"], NATIVE_HOST_NAME);
        assert_eq!(firefox_manifest["path"], host_path.to_str().unwrap());
        assert_eq!(
            firefox_manifest["allowed_extensions"][0],
            "firefox-extension-id"
        );
        assert!(firefox_manifest.get("allowed_origins").is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stale_socket_is_removed_without_disrupting_a_live_listener() {
        let root = std::env::temp_dir().join(format!("hitsu-ipc-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("browser.sock");

        let listener = UnixListener::bind(&path).unwrap();
        let error = remove_stale_socket(&path).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AddrInUse);
        drop(listener);

        remove_stale_socket(&path).unwrap();
        assert!(!path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn idle_connection_read_is_bounded_by_timeout() {
        let (server, _client) = UnixStream::pair().unwrap();
        server
            .set_read_timeout(Some(Duration::from_millis(25)))
            .unwrap();

        let started = Instant::now();
        assert_eq!(
            read_request(&server).unwrap_err(),
            "Browser request timed out"
        );
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn rejects_unknown_request_fields_and_invalid_entry_ids() {
        assert!(serde_json::from_str::<BrowserRequest>(
            r#"{"type":"listLogins","token":"t","origin":"https://example.com","extra":true}"#,
        )
        .is_err());
        assert!(serde_json::from_str::<BrowserRequest>(r#"{"type":"unknown"}"#).is_err());
        assert!(!valid_entry_id("not-an-entry-id"));
        assert!(valid_entry_id(&uuid::Uuid::new_v4().to_string()));
    }

    #[test]
    fn requests_require_a_token_and_expose_it() {
        let parsed: BrowserRequest = serde_json::from_str(
            r#"{"type":"listLogins","token":"abc123","origin":"https://example.com"}"#,
        )
        .unwrap();
        assert_eq!(parsed.token(), "abc123");

        // A request with no token is rejected outright.
        assert!(serde_json::from_str::<BrowserRequest>(
            r#"{"type":"listLogins","origin":"https://example.com"}"#,
        )
        .is_err());
    }

    #[test]
    fn token_comparison_accepts_only_the_exact_token() {
        let token = generate_token();
        assert!(token_matches(&token, &token));
        assert!(!token_matches("", &token));
        assert!(!token_matches(&token, "different"));
        assert!(!token_matches(&format!("{token}x"), &token));
    }

    #[test]
    fn generated_tokens_are_unique_and_nontrivial() {
        let first = generate_token();
        let second = generate_token();
        assert_ne!(first, second);
        // 32 random bytes → 43 URL-safe base64 chars (no padding).
        assert_eq!(first.len(), 43);
    }

    #[test]
    fn runtime_dir_prefers_an_owned_absolute_directory() {
        let owned = std::env::temp_dir().join(format!("hitsu-runtime-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&owned).unwrap();
        assert_eq!(
            runtime_dir_from(Some(owned.clone().into_os_string())),
            owned
        );
        fs::remove_dir_all(owned).unwrap();
    }

    #[test]
    fn runtime_dir_falls_back_to_temp_dir_when_unset_or_unusable() {
        let temp = std::env::temp_dir();
        assert_eq!(runtime_dir_from(None), temp);
        assert_eq!(runtime_dir_from(Some("relative/path".into())), temp);
        assert_eq!(
            runtime_dir_from(Some("/nonexistent-hitsu-runtime-dir".into())),
            temp
        );
    }

    #[test]
    fn rejects_non_web_and_deceptive_origins() {
        assert!(origin_host("chrome://extensions").is_err());
        assert!(origin_host("https://example.com@attacker.test").is_err());
        assert_ne!(
            origin_host("https://example.com.attacker.test").unwrap(),
            entry_host("example.com").unwrap()
        );
    }
}
