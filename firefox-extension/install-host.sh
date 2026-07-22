#!/usr/bin/env bash
set -euo pipefail

if [[ $# -gt 2 ]]; then
  echo "Usage: $0 [EXTENSION_ID] [NATIVE_HOST_PATH]" >&2
  exit 2
fi

production_extension_id="hitsu@ruaylabs.com"
extension_id="${1:-$production_extension_id}"
root="$(cd "$(dirname "$0")/.." && pwd)"
host_path="${2:-$root/chrome-extension/native-host/target/release/hitsu-native-host}"

if [[ ! -x "$host_path" ]]; then
  echo "Building hitsu-native-host…"
  (cd "$root" && cargo build --release --manifest-path chrome-extension/native-host/Cargo.toml)
fi
host_path="$(cd "$(dirname "$host_path")" && pwd)/$(basename "$host_path")"

if [[ "$(uname -s)" == "Darwin" ]]; then
  directory="$HOME/Library/Application Support/Mozilla/NativeMessagingHosts"
else
  directory="$HOME/.mozilla/native-messaging-hosts"
fi

mkdir -p "$directory"
python3 - "$directory/com.ruaylabs.hitsu.browser.json" "$host_path" "$extension_id" <<'PY'
import json
import sys

output, host, extension_id = sys.argv[1:]
with open(output, "w", encoding="utf-8") as file:
    json.dump({
        "name": "com.ruaylabs.hitsu.browser",
        "description": "Hitsu Password Manager native messaging host",
        "path": host,
        "type": "stdio",
        "allowed_extensions": [extension_id],
    }, file, indent=2)
    file.write("\n")
PY
chmod 600 "$directory/com.ruaylabs.hitsu.browser.json"

echo "Installed com.ruaylabs.hitsu.browser for Firefox."
