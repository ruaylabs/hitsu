#[cfg(unix)]
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(unix)]
const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

#[cfg(unix)]
fn socket_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("hitsu-browser-{}.sock", unsafe { libc::geteuid() }))
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
    let request = read_native_message()?;
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
fn read_native_message() -> Result<Vec<u8>, String> {
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
        .map_err(|_| "Invalid extension request".to_string())?;
    Ok(message)
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
