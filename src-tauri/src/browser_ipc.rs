//! Narrow local IPC API used by the browser native-messaging host.
//!
//! The Unix socket is owner-only. Requests are restricted to exact HTTP(S)
//! host matches, and secret values are returned only for an explicitly chosen
//! entry while the vault is unlocked.

#![cfg(unix)]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use url::Url;

use crate::state::AppState;

const MAX_REQUEST_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum BrowserRequest {
    ListLogins { origin: String },
    GetCredentials { id: String, origin: String },
}

pub fn socket_path() -> PathBuf {
    std::env::temp_dir().join(format!("kagi-browser-{}.sock", unsafe { libc::geteuid() }))
}

pub fn start(app: AppHandle) -> std::io::Result<()> {
    let path = socket_path();
    if path.exists() {
        fs::remove_file(&path)?;
    }
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
    Ok(())
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
        return json!({ "ok": false, "error": "Kagi is locked" });
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
    use super::{entry_host, origin_host};

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
    fn rejects_non_web_and_deceptive_origins() {
        assert!(origin_host("chrome://extensions").is_err());
        assert!(origin_host("https://example.com@attacker.test").is_err());
        assert_ne!(
            origin_host("https://example.com.attacker.test").unwrap(),
            entry_host("example.com").unwrap()
        );
    }
}
