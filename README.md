# Hitsu

Native desktop password manager built with Svelte 5 + Tauri 2 + Rust.

## Supported platforms

Hitsu officially supports macOS and Linux desktop systems only. Windows and mobile platforms are not
currently supported. Release builds and CI target macOS and Linux.

## Installation

### macOS with Homebrew

```bash
brew install --cask ruaylabs/tap/hitsu
```

The cask is maintained in the [Ruaylabs Homebrew tap](https://github.com/ruaylabs/homebrew-tap).

## Features

### Vault management

- **Open** existing KeePass databases (`.kdbx`)
- **Create** new KDBX4 vaults
- **Change** master password
- **Lock** vault (drops decrypted data and zeroizes master key)
- **Idle lock** (configurable timeout; default 5 min)
- **Lock on sleep** (detects page hidden for >30 s)
- **Lock with the OS session** via systemd-logind on Linux and NSWorkspace on macOS
- Last vault path remembered; re-opened automatically on next launch

### Entries (login, password, note, identity, card)

- **Create** entries of five types
- **Edit** all fields (type-specific editors)
- **Delete safely** to a KeePass-compatible Recycle Bin
- **Restore** trashed entries to their previous group or delete them permanently
- **Empty Recycle Bin** from Settings with a deletion-count confirmation
- **Favorites** — star/unstar, filter sidebar, ⌘⇧F toggle
- **Tags** with autocomplete suggestions
- **History** tracking via KeePass entry history
- **Entry expiration dates** stored in native KDBX metadata with due indicators
- **Attachments** — add, export, and remove files
- **Custom fields** — add arbitrary protected or unprotected key-value fields to any entry
- Field clearing (blanking a field removes it from KDBX)

### Security

- **Two-tier API** — summaries in list, full details only when selected
- **Protected fields** stored encrypted in KDBX (Password, card.number, card.cvv, card.pin)
- **Master key zeroized** by `DatabaseKey` on vault lock/drop
- **Secret DTOs zeroized** on drop; sensitive fields are redacted from debug output
- **Atomic writes** (tmp file → fsync → rename → dir-fsync) — no partial writes
- **External-modification detection** prevents overwriting a vault changed by another app
- **Verified password/KDF changes** use a temporary backup and re-open the saved vault
- **Argon2id defaults** for new vaults (64 MiB, 2 iterations, 4 lanes)
- **KDF validation and upgrade prompt** for vaults below the recommended 64 MiB
- **Constant-time** master password comparison (`subtle::ConstantTimeEq`)
- **Process hardening** disables core dumps and blocks debugger attachment in release builds
- **Privacy screen** conceals the app whenever its window loses focus
- **CSP** enabled — `default-src 'self'`, scripts locked to `'self'`, and
  `style-src-elem 'self'` so no injected style element can load

### Clipboard

- **Auto-clear** for sensitive values (password, CVV, TOTP) with configurable timeout
- **Smart clear** — only clears if clipboard still contains the secret (doesn't clobber user's subsequent copy)
- **Countdown** visible in status bar

### TOTP

- Reads KeePassXC `otp` field (`otpauth://` URI)
- Falls back to legacy `TOTP Seed` + `TOTP Settings`
- Live countdown ring with auto-refresh
- Copy with auto-clear

### Password generator

- Configurable length (8–100), character classes
- Exclude lookalikes (`il1Lo0O`)
- Live preview

### Search & navigation

- Real-time search across notes, custom fields, identity, card, and other non-protected fields
- Sidebar filters (All, Favorites, Recycle Bin, by type, by tag)
- Optional nested KDBX folder navigation and entry moving (disabled by default)
- Arrow-key navigation with Home/End jumps
- Keyboard shortcuts (⌘N new entry, ⌘⌫ delete, ⌘F search, ⌘⇧F favorites, ⌘, settings)

### Browser integration (developer preview)

- **Opt-in, off by default** — enable it in Settings → Features; nothing listens
  and no native-messaging host is registered until you do
- Minimal Manifest V3 extension for Chrome, Chromium, Brave, and Edge — [install it from the Chrome
  Web Store](https://chromewebstore.google.com/detail/hitsu-password-manager/pkickpkkbgpaffpdloplecfleckoopjc)
- Firefox Manifest V3 extension with the same popup autofill workflow
- Exact-host login lookup and popup-initiated username/password filling
- Native Messaging bridge to the unlocked desktop app on macOS and Linux
- Owner-only local IPC, gated by a per-session token; trashed entries and
  non-matching origins are rejected
- See the [Chromium](chrome-extension/README.md) and [Firefox](firefox-extension/README.md)
  development installation guides

### Appearance

- Light and dark mode (follows OS preference)

## Formats supported

| Format  | Read | Write                  |
|---------|------|------------------------|
| KDBX4.1 | ✅   | ✅                     |
| KDBX4.0 | ✅   | ✅ (upgraded on save)  |
| KDBX3   | ❌   | ❌                     |
| KDB     | ❌   | ❌                     |

KDBX4.0 vaults are upgraded to KDBX4.1 on first save. Hitsu currently requires an
Argon2/Argon2id KDF; legacy AES-KDF vaults must be upgraded in KeePassXC first.
Written vaults remain compatible with KeePassXC and KeePass 2.x.

## Tech stack

|Layer   |Technology                                                     |
|--------|---------------------------------------------------------------|
|Frontend|Svelte 5 (runes), TypeScript                                   |
|Desktop |Tauri 2                                                        |
|Backend |Rust                                                           |
|KDBX    |[keepass](https://github.com/sseemayer/keepass-rs) 0.13        |
|TOTP    |[totp-lite](https://crates.io/crates/totp-lite) (SHA-1/256/512)|
|Icons   |[Tabler Icons](https://tabler.io/icons)                        |
|Font    |OS default (system-ui fallback)                                |

## Development

```bash
# Install dependencies
pnpm install

# Run in dev mode
pnpm tauri dev

# Run checks and tests
pnpm check
pnpm test
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml

# Run the desktop end-to-end smoke test (requires tauri-driver)
pnpm test:e2e
```

### Project structure

```
src/              # Svelte frontend
  lib/
    bridge/       # Tauri IPC wrappers
    stores/       # Svelte 5 $state stores
    components/   # UI components
    utils/        # OTP, time formatting
chrome-extension/  # Chromium extension and shared native host
firefox-extension/ # Firefox extension and native-host installer
src-tauri/         # Rust backend
  src/
    commands/     # Tauri command handlers
    models/       # Entry, VaultMeta, etc.
    vault/        # Atomic write
    state.rs      # AppState (open vaults)
    prefs.rs      # User preferences
  tests/          # Integration tests
```

## Security considerations

- Decrypted vault data lives in process memory while unlocked; `vault_lock` drops the
  `Database` and zeroizes the master key buffer, but heap memory is not explicitly
  scrubbed after drop.
- Hitsu validates Argon2/Argon2id parameters on open and rejects AES-KDF vaults.
  It does not currently expose cipher or KDF configuration controls.
- Memory locking (`mlock`/`mprotect`) is not implemented — sensitive pages may be
  written to swap. Use encrypted swap or full-disk encryption.
- The CSP keeps `style-src-attr 'unsafe-inline'` (plus `style-src 'unsafe-inline'`
  as a fallback for older WebKit): Svelte's dynamic `style=` bindings require it.
  Style *elements* are restricted to `'self'`, and with `script-src 'self'` the
  residual style-injection risk is low.
- The browser integration's local socket lives in `$XDG_RUNTIME_DIR` (per-user,
  mode 0700) when available, so other users can never pre-bind its predictable
  path; it falls back to the per-user temp dir on macOS. The socket is still
  reachable by any process running as the same OS user. A per-session token (written to an
  owner-only file, injected by the native host, and verified in constant time) stops naive
  enumeration, but it is not a hard boundary: a same-user process can also read the token
  file. The desktop app remains the primary trust boundary; the browser bridge
  is a developer preview and is disabled by default (Settings → Features).
