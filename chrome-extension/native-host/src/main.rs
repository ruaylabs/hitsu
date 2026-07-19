#[cfg(unix)]
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(unix)]
const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

/// Directory holding the browser IPC socket and token file. Mirrors
/// `runtime_dir` in the desktop app's `browser_ipc.rs` — the two must resolve
/// identically or this host will look for the socket in the wrong place.
#[cfg(unix)]
fn runtime_dir() -> std::path::PathBuf {
    use std::os::unix::fs::MetadataExt;
    if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR").map(std::path::PathBuf::from) {
        let owned_by_us = dir.metadata().is_ok_and(|metadata| {
            metadata.is_dir() && metadata.uid() == unsafe { libc::geteuid() }
        });
        if dir.is_absolute() && owned_by_us {
            return dir;
        }
    }
    std::env::temp_dir()
}

#[cfg(unix)]
fn socket_path() -> std::path::PathBuf {
    runtime_dir().join(format!("hitsu-browser-{}.sock", unsafe { libc::geteuid() }))
}

#[cfg(unix)]
fn token_path() -> std::path::PathBuf {
    runtime_dir().join(format!("hitsu-browser-{}.token", unsafe {
        libc::geteuid()
    }))
}

/// Read this session's browser-IPC token, written by the desktop app to an
/// owner-only file. Its absence means the app isn't running (or is too old to
/// expose the token), which is indistinguishable to us from "not unlocked".
#[cfg(unix)]
fn read_token() -> Result<String, String> {
    let token = std::fs::read_to_string(token_path())
        .map(|token| token.trim().to_string())
        .map_err(|_| "Open and unlock the Hitsu desktop app first".to_string())?;
    if token.is_empty() {
        return Err("Open and unlock the Hitsu desktop app first".to_string());
    }
    Ok(token)
}

#[cfg(unix)]
fn main() {
    if let Err(error) = run() {
        let response = serde_json::json!({ "ok": false, "error": error });
        let _ = write_native_message(&response);
    }
}

#[cfg(not(unix))]
fn main() {
    let response = serde_json::json!({
        "ok": false,
        "error": "Hitsu browser integration is not supported on this platform"
    });
    let bytes = serde_json::to_vec(&response).unwrap_or_default();
    let _ = std::io::Write::write_all(&mut std::io::stdout(), &(bytes.len() as u32).to_ne_bytes());
    let _ = std::io::Write::write_all(&mut std::io::stdout(), &bytes);
}

#[cfg(unix)]
fn run() -> Result<(), String> {
    let mut request = read_native_message()?;
    let token = read_token()?;

    // Inject the session token the extension can't read for itself (it has no
    // filesystem access). The backend verifies it before doing any work.
    let object = request
        .as_object_mut()
        .ok_or_else(|| "Invalid extension request".to_string())?;
    object.insert("token".to_string(), serde_json::Value::String(token));
    let request = serde_json::to_vec(&request).map_err(|_| "Could not encode request")?;

    let mut stream = UnixStream::connect(socket_path())
        .map_err(|_| "Open and unlock the Hitsu desktop app first".to_string())?;
    stream
        .write_all(&request)
        .and_then(|_| stream.write_all(b"\n"))
        .map_err(|_| "Could not contact Hitsu".to_string())?;

    let mut response = Vec::new();
    BufReader::new(stream)
        .take(MAX_MESSAGE_BYTES as u64)
        .read_until(b'\n', &mut response)
        .map_err(|_| "Could not read Hitsu's response".to_string())?;
    if response.last() == Some(&b'\n') {
        response.pop();
    }
    let response: serde_json::Value =
        serde_json::from_slice(&response).map_err(|_| "Hitsu returned an invalid response")?;
    write_native_message(&response)
}

#[cfg(unix)]
fn read_native_message() -> Result<serde_json::Value, String> {
    let mut length = [0u8; 4];
    std::io::stdin()
        .read_exact(&mut length)
        .map_err(|_| "Could not read the extension request".to_string())?;
    let length = u32::from_ne_bytes(length) as usize;
    if length == 0 || length > MAX_MESSAGE_BYTES {
        return Err("Extension request is too large".to_string());
    }
    let mut message = vec![0u8; length];
    std::io::stdin()
        .read_exact(&mut message)
        .map_err(|_| "Incomplete extension request".to_string())?;
    serde_json::from_slice::<serde_json::Value>(&message)
        .map_err(|_| "Invalid extension request".to_string())
}

#[cfg(unix)]
fn write_native_message(message: &serde_json::Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(message).map_err(|_| "Could not encode response".to_string())?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err("Response is too large".to_string());
    }
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(&(bytes.len() as u32).to_ne_bytes())
        .and_then(|_| stdout.write_all(&bytes))
        .and_then(|_| stdout.flush())
        .map_err(|_| "Could not write response".to_string())
}
