# Hitsu Chrome Extension

Minimal Manifest V3 extension for Chrome, Chromium, Brave, and Edge. It lists exact-host login
matches from the unlocked Hitsu desktop app and fills the first username/password form on the current
page.

## Install for development

1. Build Hitsu and its native-messaging host:

   ```bash
   cargo build --release --manifest-path src-tauri/Cargo.toml --bin hitsu
   cargo build --release --manifest-path chrome-extension/native-host/Cargo.toml
   ```

2. Open `chrome://extensions`, enable **Developer mode**, choose **Load unpacked**, and select this
   `chrome-extension` directory.
3. Copy the extension ID shown by Chrome.
4. Register the native host with that unpacked-development ID:

   ```bash
   ./chrome-extension/install-host.sh abcdefghijklmnopqrstuvwxyzabcdef
   ```

   Running the script without an ID registers the permanent Web Store extension instead.

5. Start and unlock the release build of Hitsu. Open an HTTP(S) page whose hostname exactly matches a
   login URL, then select the Hitsu toolbar button.

A release built with `pnpm tauri build` contains the native host as a Tauri sidecar. On first launch,
the release app automatically registers that host for the permanent Web Store extension in Chrome,
Chromium, Brave, and Edge. The script remains useful for unpacked development extensions whose IDs
differ from the store ID.

Reload the extension from `chrome://extensions` after changing its source files.

## Security model

- The extension uses `activeTab`, `nativeMessaging`, and `scripting`; it has no persistent host
  permissions and injects the local fill script only after the user selects a login.
- Hitsu exposes an owner-only Unix socket available while the desktop app is running.
- The Unix account is the local trust boundary: another process running as the same user can connect
  directly to the socket while the vault is unlocked. Native Messaging restricts browser access to
  the configured extension ID, but it does not authenticate same-user processes to the socket.
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
- A store archive can be created with `just extension-zip`; publishing and desktop release
  distribution remain separate manual steps.
