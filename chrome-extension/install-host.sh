#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 EXTENSION_ID [NATIVE_HOST_PATH]" >&2
  exit 2
fi

extension_id="$1"
root="$(cd "$(dirname "$0")/.." && pwd)"
host_path="${2:-$root/src-tauri/target/release/kagi-native-host}"

if [[ ! -x "$host_path" ]]; then
  echo "Building kagi-native-host…"
  (cd "$root" && cargo build --release --manifest-path src-tauri/Cargo.toml --bin kagi-native-host)
fi
host_path="$(cd "$(dirname "$host_path")" && pwd)/$(basename "$host_path")"

if [[ "$(uname -s)" == "Darwin" ]]; then
  bases=(
    "$HOME/Library/Application Support/Google/Chrome"
    "$HOME/Library/Application Support/Chromium"
    "$HOME/Library/Application Support/BraveSoftware/Brave-Browser"
    "$HOME/Library/Application Support/Microsoft Edge"
  )
else
  bases=(
    "$HOME/.config/google-chrome"
    "$HOME/.config/chromium"
    "$HOME/.config/BraveSoftware/Brave-Browser"
    "$HOME/.config/microsoft-edge"
  )
fi

for base in "${bases[@]}"; do
  directory="$base/NativeMessagingHosts"
  mkdir -p "$directory"
  python3 - "$directory/com.ruaylabs.kagi.browser.json" "$host_path" "$extension_id" <<'PY'
import json
import sys

output, host, extension_id = sys.argv[1:]
with open(output, "w", encoding="utf-8") as file:
    json.dump({
        "name": "com.ruaylabs.kagi.browser",
        "description": "Kagi Password Manager native messaging host",
        "path": host,
        "type": "stdio",
        "allowed_origins": [f"chrome-extension://{extension_id}/"],
    }, file, indent=2)
    file.write("\n")
PY
  chmod 600 "$directory/com.ruaylabs.kagi.browser.json"
done

echo "Installed com.ruaylabs.kagi.browser for Chrome, Chromium, Brave, and Edge."
