#!/usr/bin/env bash
# Sign the helper without the main app's provisioning-restricted Touch ID entitlements.
set -euo pipefail
cd "$(dirname "$0")/.."

[[ "$(uname -s)" == Darwin ]] || {
  echo "Requires macOS" >&2
  exit 1
}
: "${APPLE_SIGNING_IDENTITY:?Set a Developer ID Application signing identity}"
signing_dir="${1:-.touch-id-signing}"
release="${2:-}"
app="src-tauri/target/release/bundle/macos/Hitsu.app"
helper="$app/Contents/MacOS/hitsu-native-host"

node scripts/prepare-macos-touch-id-signing.mjs "$signing_dir"
# Tauri applies the same entitlements to every executable. Use ad-hoc signing
# here, then replace both signatures explicitly before notarizing anything.
env -u APPLE_CERTIFICATE -u APPLE_CERTIFICATE_PASSWORD -u APPLE_ID \
  -u APPLE_PASSWORD -u APPLE_API_KEY -u APPLE_API_ISSUER -u APPLE_API_KEY_PATH \
  APPLE_SIGNING_IDENTITY=- \
  pnpm tauri build --bundles app --config "$signing_dir/tauri.touch-id.conf.json"

work="$(mktemp -d)"
keychain=""
original_keychains=()
cleanup() {
  if [[ -n "$keychain" ]]; then
    security list-keychains -d user -s "${original_keychains[@]}"
    security delete-keychain "$keychain"
  fi
  rm -rf "$work"
}
trap cleanup EXIT
if [[ -n "${APPLE_CERTIFICATE:-}" ]]; then
  : "${APPLE_CERTIFICATE_PASSWORD:?Set APPLE_CERTIFICATE_PASSWORD}"
  while IFS= read -r entry; do
    original_keychains+=("$(printf '%s' "$entry" | xargs)")
  done < <(security list-keychains -d user)
  keychain="$work/signing.keychain-db"
  password="$(openssl rand -hex 24)"
  printf '%s' "$APPLE_CERTIFICATE" | /usr/bin/base64 -D >"$work/certificate.p12"
  security create-keychain -p "$password" "$keychain"
  security set-keychain-settings -lut 21600 "$keychain"
  security unlock-keychain -p "$password" "$keychain"
  security import "$work/certificate.p12" -k "$keychain" \
    -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
  security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$password" "$keychain"
  security list-keychains -d user -s "$keychain" "${original_keychains[@]}"
fi

# Do not use --deep for signing: the helper must not inherit app entitlements.
codesign --force --timestamp --options runtime --sign "$APPLE_SIGNING_IDENTITY" "$helper"
codesign --force --timestamp --options runtime --sign "$APPLE_SIGNING_IDENTITY" \
  --entitlements "$signing_dir/Hitsu.entitlements" "$app"
codesign --verify --deep --strict --verbose=2 "$app"
codesign -d --entitlements :- "$helper" >"$work/helper-entitlements.plist"
# A plain Rust stdio helper needs no entitlements at all.
if [[ -s "$work/helper-entitlements.plist" ]]; then
  echo "Native host unexpectedly has entitlements" >&2
  exit 1
fi
# Signature verification alone did not catch the original AMFI launch rejection.
"$helper" </dev/null

if [[ "$release" == --release ]]; then
  : "${APPLE_ID:?Set APPLE_ID}"
  : "${APPLE_PASSWORD:?Set APPLE_PASSWORD}"
  ditto -c -k --keepParent "$app" "$work/Hitsu.zip"
  xcrun notarytool submit "$work/Hitsu.zip" --apple-id "$APPLE_ID" \
    --password "$APPLE_PASSWORD" --team-id "$APPLE_TEAM_ID" --wait \
    --output-format json >"$work/notarization.json"
  node -e 'if (require(process.argv[1]).status !== "Accepted") process.exit(1)' \
    "$work/notarization.json"
  xcrun stapler staple "$app"
  xcrun stapler validate "$app"
  spctl --assess --type execute --verbose=2 "$app"
  mkdir "$work/dmg"
  ditto "$app" "$work/dmg/Hitsu.app"
  ln -s /Applications "$work/dmg/Applications"
  version="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$app/Contents/Info.plist")"
  mkdir -p src-tauri/target/release/bundle/dmg
  arch="$(uname -m)"
  [[ "$arch" != arm64 ]] || arch=aarch64
  dmg="src-tauri/target/release/bundle/dmg/Hitsu_${version}_${arch}.dmg"
  rm -f "$dmg"
  hdiutil create -volname Hitsu -srcfolder "$work/dmg" -ov -format UDZO "$dmg"
  codesign --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$dmg"
  xcrun notarytool submit "$dmg" --apple-id "$APPLE_ID" \
    --password "$APPLE_PASSWORD" --team-id "$APPLE_TEAM_ID" --wait \
    --output-format json >"$work/notarization.json"
  node -e 'if (require(process.argv[1]).status !== "Accepted") process.exit(1)' \
    "$work/notarization.json"
  xcrun stapler staple "$dmg"
  xcrun stapler validate "$dmg"
fi
