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
- Last vault path remembered; re-opened automatically on next launch

### Entries (login, note, identity, card)

- **Create** entries of four types
- **Edit** all fields (type-specific editors)
- **Delete** with confirmation
- **Favorites** — star/unstar, filter sidebar, ⌘⇧F toggle
- **Tags** with autocomplete suggestions
- **History** tracking via KeePass entry history
- Field clearing (blanking a field removes it from KDBX)

### Security

- **Two-tier API** — summaries in list, full details only when selected
- **Protected fields** stored encrypted in KDBX (Password, card.number, card.cvv, card.pin)
- **Master key zeroized** on vault lock/drop (`Zeroizing<Vec<u8>>`)
- **Atomic writes** (tmp file → fsync → rename → dir-fsync) — no partial writes
- **Constant-time** master password comparison (`subtle::ConstantTimeEq`)
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
- Sidebar filters (All, Favorites, by type, by tag)
- Keyboard shortcuts (⌘K, ⌘, for settings, ⌘⇧F for favorite, ⌘F to focus search)

### Appearance

- Light and dark mode (follows OS preference)

## Formats supported

|Format |Read|Write               |
|-------|----|--------------------|
|KDBX4.1|✅   |✅                   |
|KDBX4.0|✅   |✅ (upgraded on save)|
|KDBX3  |✅   |✅ (upgraded on save)|
|KDB    |✅   |✅ (upgraded on save)|

Vaults opened in non-KDBX4 formats are silently upgraded to KDBX4.1 on first save.
This is fully compatible with KeePassXC and KeePass 2.x.

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
cargo tauri dev    # from src-tauri/

# Run checks
cargo check --manifest-path src-tauri/Cargo.toml
cargo test  --manifest-path src-tauri/Cargo.toml
pnpm check
```

### Project structure

```
src/              # Svelte frontend
  lib/
    bridge/       # Tauri IPC wrappers
    stores/       # Svelte 5 $state stores
    components/   # UI components
    utils/        # OTP, time formatting
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
- Argon2 KDF parameters and encryption cipher are **not validated** on open (see
  `keepass-rust-bridge-guide.md` for guidance).
- Memory locking (`mlock`/`mprotect`) is not implemented — sensitive pages may be
  written to swap.
