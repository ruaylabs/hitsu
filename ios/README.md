# HitsuPasswordManager for iOS

A minimal, read-only SwiftUI client for Hitsu/KeePass `.kdbx` databases.

## Open it

1. Open `HitsuPasswordManager.xcodeproj` in Xcode 16 or later.
2. Select an iOS 18 simulator or device.
3. For device builds, copy `Config/Signing.local.xcconfig.example` to
   `Config/Signing.local.xcconfig` and replace `YOUR_TEAM_ID` with your Apple Developer team ID.
4. Build and run.
5. Tap **Open from Files or iCloud Drive**, choose a `.kdbx` file, and enter its
   master password.

The system Files picker provides access to iCloud Drive. The app requests a
security-scoped read URL, reads the database, and never writes to that URL. No
CloudKit container or app-specific iCloud entitlement is required for this
user-selected document flow.

## Scope

- KDBX 4.0, 4.1, and 3.1 read support comes from
  [KDBXKit](https://github.com/shadone/KDBXKit) `1.3.0` via Swift Package Manager.
  The desktop Hitsu writer must emit standard KDBX output for stock KDBXKit compatibility.
- The app lists entries, searches titles, usernames, URLs, folders, tags, notes,
  and unprotected custom or typed fields, and displays details on demand. A Recent
  tab shows the 20 most recently modified
  active entries. Expiration dates are shown in entry details and due entries
  are flagged in lists. Entry links open only HTTP(S) URLs. Entries in the recycle bin
  stay out of the Favorites, Recent,
  and Categories lists and are shown read-only under a Trash item at the end of
  Categories.
- Passwords and protected custom fields are revealed only after tapping a
  reveal button.
- Copied passwords stay on the local clipboard and expire after 30 seconds.
- KeePassXC `otp` and legacy KeePass TOTP fields generate codes with a live expiry indicator; copied
  codes expire from the local clipboard when the code does.
- The vault locks automatically when the app leaves the foreground, and a privacy shield
  covers the app switcher snapshot.
- There are intentionally no save, edit, delete, import, or write-back paths.

KDBXKit is BSD-2-Clause licensed. Review its license and dependency licenses
before distributing the app.
