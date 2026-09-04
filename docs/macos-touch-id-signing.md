# macOS Touch ID signing

Hitsu stores opted-in vault passwords in the macOS Data Protection Keychain. Apple requires an
app-like bundle with keychain access-group entitlements authorized by an embedded provisioning
profile for this keychain implementation. A Developer ID certificate by itself, or ad-hoc signing,
is not sufficient.

References:

- [Apple TN3137: On Mac keychain APIs and implementations][apple-tn3137]
- [Tauri macOS code signing][tauri-signing]
- [Tauri `bundle.macOS.files` and `entitlements` configuration][tauri-config]

[apple-tn3137]:
  https://developer.apple.com/documentation/technotes/tn3137-on-mac-keychains
[tauri-signing]: https://v2.tauri.app/distribute/sign/macos/
[tauri-config]: https://v2.tauri.app/reference/config/#macconfig

## Apple Developer setup

1. Register the macOS App ID `com.ruaylabs.hitsu` for the release team's account.
2. Create a Developer ID provisioning profile that authorizes that App ID and its default keychain
   access group (`<TEAM_ID>.com.ruaylabs.hitsu`).
3. Ensure the profile uses the same team and Developer ID signing certificate as the release build.

Do not commit a generated entitlements file with a hard-coded team ID. For local builds, the
preparation script writes the entitlements and Tauri config overlay to the ignored
`.touch-id-signing/` directory. CI instead writes them under `$RUNNER_TEMP`.

## Local signed build

```bash
export APPLE_TEAM_ID="YOURTEAMID"
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (YOURTEAMID)"
export APPLE_PROVISIONING_PROFILE_PATH="$HOME/Downloads/embedded.provisionprofile"
just macos-touch-id-build
```

The task prepares the signing overlay, builds the app, verifies its signature and embedded profile,
and launches it. Set `APPLE_PROVISIONING_PROFILE` instead when the profile is already base64
encoded. The normal Tauri notarization environment variables still apply.

Unsigned `tauri dev` builds intentionally report Touch ID as unavailable because they cannot access
the Data Protection Keychain group.

## GitHub Actions

Store the profile as a single-line base64 value in the `APPLE_PROVISIONING_PROFILE` Actions secret:

```bash
base64 < embedded.provisionprofile | tr -d '\n'
```

The release and nightly workflows generate the signing overlay in `$RUNNER_TEMP`, embed the profile
at `Contents/embedded.provisionprofile`, and pass the generated entitlements to Tauri. Existing
`APPLE_TEAM_ID`, certificate, and notarization secrets remain unchanged.

After building, verify the artifact on macOS:

```bash
codesign -d --entitlements :- /Applications/Hitsu.app
security cms -D -i /Applications/Hitsu.app/Contents/embedded.provisionprofile
```
