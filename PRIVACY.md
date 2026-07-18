# Hitsu Privacy Policy

**Effective date:** July 18, 2026

Hitsu is a local-first desktop password manager with an optional browser extension, developed by
Ruaylabs.

## Desktop application

The Hitsu desktop app stores your vault locally on your device and sends no telemetry.

## Browser extension

When you open the extension, it temporarily processes:

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

- **activeTab** — read the active page's origin when you open Hitsu.
- **scripting** — inject the bundled credential-filling script after you select a login.
- **nativeMessaging** — communicate with the locally installed Hitsu desktop app.

The extension does not request persistent access to all websites.

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
