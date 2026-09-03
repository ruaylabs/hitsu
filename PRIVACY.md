# Hitsu Privacy Policy

**Effective date:** July 18, 2026

Hitsu is a local-first password manager with an optional browser extension and an iOS viewer,
developed by Ruaylabs.

## Desktop application

The Hitsu desktop app stores your vault locally on your device and sends no telemetry.

## Browser extension

When you focus an eligible login field or open the extension, it temporarily processes:

- The active tab's origin, to find logins matching the current site.
- Login titles and usernames returned by the locally installed Hitsu desktop app.
- The username and password of the login you explicitly select to fill.

This information is used solely to fill the login you select. Credentials are matched against the
exact active-site hostname, and nothing leaves your device: the extension communicates only with
the local desktop app through the browser's Native Messaging API. It does not collect browsing
history or persist anything in browser storage.

All executable code ships in the published extension package; no remote code is downloaded or
executed. Use of information received from browser APIs adheres to the Chrome Web Store User Data
Policy, including its Limited Use requirements.

### Permissions

- **Website access (HTTP and HTTPS)** — detect focused login fields and display matching Hitsu
  suggestions. Login metadata is requested only after an eligible field receives focus.
- **activeTab** — read the active page's origin when you open Hitsu from the toolbar.
- **scripting** — run the bundled credential-filling script after you select a login.
- **nativeMessaging** — communicate with the locally installed Hitsu desktop app.

## iOS application

The Hitsu iOS app is a read-only viewer for your KeePass `.kdbx` vault. It runs entirely on
device: it makes no network connections, sends no telemetry, and never writes to your vault.

- Your master password is used only to unlock the vault in memory and is never stored or
  transmitted.
- The vault locks automatically whenever the app leaves the foreground and after a period of
  inactivity while it stays open. You choose the inactivity period (1, 5, or 15 minutes) on the
  lock screen, before unlocking; it cannot be disabled.
- Copied passwords and one-time codes stay on the local clipboard briefly and expire on their
  own.
- Decrypted attachment previews are kept in a protected temporary location and are purged when
  the app locks or restarts.
- The only data the app keeps between launches is a bookmark to the vault file you last opened,
  stored on device.

## Data sharing

Ruaylabs does not sell, rent, or share user data, and Hitsu includes no advertising, analytics, or
tracking services.

## Children's privacy

Hitsu is not directed to children under 13 and does not knowingly collect personal information
from children.

## Changes

Updates are published at this URL with a revised effective date.

## Contact

For privacy questions, contact Ruaylabs at hello@ruaylabs.com.
