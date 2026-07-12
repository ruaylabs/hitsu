# Kagi Chrome Extension

Minimal Manifest V3 extension for Chrome, Chromium, Brave, and Edge. It lists exact-host login
matches from the unlocked Kagi desktop app and fills the first username/password form on the current
page.

## Install for development

1. Build Kagi and its native-messaging host:

   ```bash
   cargo build --release --manifest-path src-tauri/Cargo.toml --bin kagi
   cargo build --release --manifest-path chrome-extension/native-host/Cargo.toml
   ```

2. Open `chrome://extensions`, enable **Developer mode**, choose **Load unpacked**, and select this
   `chrome-extension` directory.
3. Copy the extension ID shown by Chrome.
4. Register the native host, replacing the example ID:

   ```bash
   ./chrome-extension/install-host.sh abcdefghijklmnopqrstuvwxyzabcdef
   ```

5. Start and unlock the release build of Kagi. Open an HTTP(S) page whose hostname exactly matches a
   login URL, then select the Kagi toolbar button.

A release built with `pnpm tauri build` contains the native host as a Tauri sidecar. The script above
remains useful for an unpacked development extension until automatic production registration is
implemented.

Reload the extension from `chrome://extensions` after changing its source files.

For production packaging, see [`BUNDLING.md`](BUNDLING.md).

## Security model

- The extension asks only for `activeTab` and `nativeMessaging` permissions.
- Kagi exposes an owner-only Unix socket available while the desktop app is running.
- Login metadata is returned only for exact hostname matches.
- A password is returned only after selecting a matching entry.
- Trashed entries are never exposed.
- Credentials are sent directly from the extension service worker to the page content script; the
  popup does not receive them.
- The desktop backend remains the source of truth and refuses requests while locked.

## Current limitations

- macOS and Linux only; the local IPC transport does not yet support Windows.
- Exact hostname matching only; related subdomains are intentionally not inferred.
- Fills the first visible password field and nearest preceding username-like field.
- No inline suggestions, save/update prompts, generated-password capture, HTTP-auth support, or
  iframe handling.
- Developer-mode installation only. Store packaging and a stable signed extension ID are not
  configured.
