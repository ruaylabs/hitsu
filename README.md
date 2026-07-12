# Kagi

Native desktop password manager built with Svelte 5 + Tauri 2 + Rust.

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
- **Favorites** — star/unstar, filter sidebar, ⌘⇧F toggle
- **Tags** with autocomplete suggestions
- **History** tracking via KeePass entry history
- **Attachments** — add, export, and remove files
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
- **CSP** enabled — `default-src 'self'`

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

- Configurable length (8–128), character classes
- Exclude lookalikes (`il1Lo0O`)
- Live preview

### Search & navigation

- Real-time search (title, username, URL, tags)
- Sidebar filters (All, Favorites, Recycle Bin, by type, by tag)
- Arrow-key navigation with Home/End jumps
- Keyboard shortcuts (⌘N new entry, ⌘⌫ delete, ⌘F search, ⌘⇧F favorites, ⌘, settings)

### Browser integration (developer preview)

- Minimal Manifest V3 extension for Chrome, Chromium, Brave, and Edge
- Exact-host login lookup and popup-initiated username/password filling
- Native Messaging bridge to the unlocked desktop app on macOS and Linux
- Owner-only local IPC; trashed entries and non-matching origins are rejected
- See [`chrome-extension/README.md`](chrome-extension/README.md) for development installation

### Appearance

- Light and dark mode (follows OS preference)

## Formats supported

| Format  | Read | Write                  |
|---------|------|------------------------|
| KDBX4.1 | ✅   | ✅                     |
| KDBX4.0 | ✅   | ✅ (upgraded on save)  |
| KDBX3   | ❌   | ❌                     |
| KDB     | ❌   | ❌                     |

KDBX4.0 vaults are upgraded to KDBX4.1 on first save. Kagi currently requires an
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
chrome-extension/ # Chromium extension and native-host installer
src-tauri/        # Rust backend
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
- Kagi validates Argon2/Argon2id parameters on open and rejects AES-KDF vaults.
  It does not currently expose cipher or KDF configuration controls.
- Memory locking (`mlock`/`mprotect`) is not implemented — sensitive pages may be
  written to swap. Use encrypted swap or full-disk encryption.
