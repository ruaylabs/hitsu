//! Narrow local IPC API used by the browser native-messaging host.
//!
//! The Unix socket is owner-only. Requests are restricted to exact HTTP(S)
//! host matches, and secret values are returned only for an explicitly chosen
//! entry while the vault is unlocked.

#![cfg(unix)]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use url::Url;

use crate::state::AppState;
use crate::vault::atomic_write;

const MAX_REQUEST_BYTES: u64 = 1024 * 1024;
const NATIVE_HOST_NAME: &str = "com.ruaylabs.hitsu.browser";
const PRODUCTION_EXTENSION_ID: &str = "pkickpkkbgpaffpdloplecfleckoopjc";

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
enum BrowserRequest {
    ListLogins { origin: String },
    GetCredentials { id: String, origin: String },
}

pub fn socket_path() -> PathBuf {
    std::env::temp_dir().join(format!("hitsu-browser-{}.sock", unsafe { libc::geteuid() }))
}

fn native_host_manifest_directories() -> std::io::Result<Vec<PathBuf>> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME is not set"))?;

    #[cfg(target_os = "macos")]
    let directories = [
        "Library/Application Support/Google/Chrome/NativeMessagingHosts",
        "Library/Application Support/Chromium/NativeMessagingHosts",
        "Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts",
        "Library/Application Support/Microsoft Edge/NativeMessagingHosts",
    ]
    .into_iter()
    .map(|relative| home.join(relative))
    .collect();

    #[cfg(not(target_os = "macos"))]
    let directories = {
        let config_home = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));
        [
            "google-chrome/NativeMessagingHosts",
            "chromium/NativeMessagingHosts",
            "BraveSoftware/Brave-Browser/NativeMessagingHosts",
            "microsoft-edge/NativeMessagingHosts",
        ]
        .into_iter()
        .map(|relative| config_home.join(relative))
        .collect()
    };

    Ok(directories)
}

fn write_native_host_manifests(
    host_path: &Path,
    directories: &[PathBuf],
    extension_id: &str,
) -> std::io::Result<()> {
    let host_path = host_path.to_str().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "native host path is not valid UTF-8",
        )
    })?;
    let manifest = serde_json::to_vec_pretty(&json!({
        "name": NATIVE_HOST_NAME,
        "description": "Hitsu Password Manager native messaging host",
        "path": host_path,
        "type": "stdio",
        "allowed_origins": [format!("chrome-extension://{extension_id}/")],
    }))
    .map_err(std::io::Error::other)?;

    for directory in directories {
        fs::create_dir_all(directory)?;
        let destination = directory.join(format!("{NATIVE_HOST_NAME}.json"));
        atomic_write(&destination, &manifest)?;
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
    write_native_host_manifests(
        &host_path,
        &native_host_manifest_directories()?,
        PRODUCTION_EXTENSION_ID,
    )
}

pub struct BrowserIpcSocket {
    path: PathBuf,
}

impl Drop for BrowserIpcSocket {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
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
    remove_stale_socket(&path)?;
    let listener = UnixListener::bind(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;

    std::thread::Builder::new()
        .name("browser-ipc".into())
        .spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => handle_connection(&app, stream),
                    Err(error) => eprintln!("browser IPC connection failed: {error}"),
                }
            }
        })?;
    Ok(BrowserIpcSocket { path })
}

fn handle_connection(app: &AppHandle, mut stream: UnixStream) {
    let request = {
        let mut line = String::new();
        let mut reader = BufReader::new(&stream).take(MAX_REQUEST_BYTES);
        match reader.read_line(&mut line) {
            Ok(0) => return,
            Ok(_) => serde_json::from_str::<BrowserRequest>(&line)
                .map_err(|_| "Invalid browser request".to_string()),
            Err(_) => Err("Could not read browser request".to_string()),
        }
    };

    let response = match request {
        Ok(request) => process_request(app, request),
        Err(error) => json!({ "ok": false, "error": error }),
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
        BrowserRequest::ListLogins { origin } => {
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
        BrowserRequest::GetCredentials { id, origin } => {
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
        entry_host, origin_host, remove_stale_socket, valid_entry_id, write_native_host_manifests,
        BrowserRequest, NATIVE_HOST_NAME,
    };
    use std::fs;
    use std::os::unix::net::UnixListener;

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
    fn writes_native_host_manifest_with_production_shape() {
        let root = std::env::temp_dir().join(format!("hitsu-host-{}", uuid::Uuid::new_v4()));
        let directory = root.join("NativeMessagingHosts");
        let host_path = root.join("hitsu-native-host");

        write_native_host_manifests(&host_path, std::slice::from_ref(&directory), "extension-id")
            .unwrap();

        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(directory.join(format!("{NATIVE_HOST_NAME}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(manifest["name"], NATIVE_HOST_NAME);
        assert_eq!(manifest["path"], host_path.to_str().unwrap());
        assert_eq!(
            manifest["allowed_origins"][0],
            "chrome-extension://extension-id/"
        );
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
    fn rejects_unknown_request_fields_and_invalid_entry_ids() {
        assert!(serde_json::from_str::<BrowserRequest>(
            r#"{"type":"listLogins","origin":"https://example.com","extra":true}"#,
        )
        .is_err());
        assert!(serde_json::from_str::<BrowserRequest>(r#"{"type":"unknown"}"#).is_err());
        assert!(!valid_entry_id("not-an-entry-id"));
        assert!(valid_entry_id(&uuid::Uuid::new_v4().to_string()));
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
