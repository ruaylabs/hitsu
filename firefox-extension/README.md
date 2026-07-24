# Hitsu Firefox Extension

Minimal Manifest V3 extension for Firefox. It lists exact-host login matches from the unlocked
Hitsu desktop app and fills the first username/password form on the current page.

## Install for development

Run the local setup command and follow the browser instructions it prints:

```bash
just firefox-extension-dev
```

This removes stale Firefox packages, rebuilds the extension, and installs the native-messaging
host. The equivalent manual steps are:

1. Build Hitsu, the shared native-messaging host, and the extension:

   ```bash
   cargo build --release --manifest-path src-tauri/Cargo.toml --bin hitsu
   cargo build --release --manifest-path browser-extension/native-host/Cargo.toml
   node scripts/build-browser-extension.mjs firefox
   ```

2. Open `about:debugging#/runtime/this-firefox`, choose **Load Temporary Add-on**, and select
   `package/hitsu-firefox-extension/manifest.json`.
3. Register the native host for the extension ID declared in the manifest:

   ```bash
   ./firefox-extension/install-host.sh
   ```

4. Start the release build of Hitsu, enable **Settings → Features → Browser integration**, and
   unlock a vault. Open an HTTP(S) page whose hostname exactly matches a login URL, then select the
   Hitsu toolbar button.

A release built with `pnpm tauri build` contains the native host as a Tauri sidecar. Enabling
browser integration registers it for the stable Firefox extension ID `hitsu@ruaylabs.com`. The
install script is useful for local development or an alternate extension ID.

After changing files in `browser-extension/`, rebuild with
`node scripts/build-browser-extension.mjs firefox` and reload the temporary extension from
`about:debugging`.

## Security model

- The extension uses `activeTab`, `nativeMessaging`, and `scripting`; it has no persistent host
  permissions and injects the local fill script only after the user selects a login.
- The extension declares that it collects no user data.
- Browser integration is opt-in and off by default. Until enabled, no socket listens and no
  native-messaging host manifest is registered.
- Login metadata is returned only for exact HTTP(S) hostname matches.
- A password is returned only after selecting a matching entry, and trashed entries are excluded.
- Credentials travel directly from the background script to the page content script; the popup
  never receives them.
- The owner-only Unix socket and per-session token use the same security model as the Chromium
  extension. The desktop backend remains the source of truth and refuses requests while locked.

## Current limitations

- macOS and Linux only; Windows support is not planned.
- Exact hostname matching only; related subdomains are intentionally not inferred.
- Fills the first writable password field and nearest preceding username-like field.
- No inline suggestions, save/update prompts, generated-password capture, HTTP-auth support, or
  iframe handling.
