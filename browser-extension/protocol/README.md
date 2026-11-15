# Browser/desktop protocol compatibility

Run the CI gate locally (Node.js and the desktop Rust build dependencies are required):

```bash
cd src-tauri
cargo test --locked --lib browser_protocol_compatibility -- --ignored --nocapture
```

The test executes the actual background listeners with synthetic browser APIs and captures five
native requests: popup/inline login lists, popup/inline fills, and TOTP. It injects a synthetic
session token as the native host does, then deserializes requests with the live desktop's
`BrowserRequest` definition. It also checks current extension requests against the actual request
definition from desktop 0.7.1, the oldest supported release.

## Pinned compatibility matrix

- Current extension -> current desktop.
- Published Chrome 0.2.1 -> current desktop.
- Published Firefox 0.2.1 -> current desktop.
- Current extension -> desktop 0.7.1.

Snapshots come from published store packages. Chrome 0.2.1 replaces the incompatible 0.2.0
snapshot: its login-list and credential requests include the required `inline` field. Chrome
and Firefox 0.2.1 ship identical background scripts. Do not substitute unpublished local builds
for published snapshots.

`fixtures/versions.json` records versions, source URLs, SHA-256 checksums of the downloaded store
packages, checksums of the extracted background scripts, and the oldest desktop commit. Background
scripts are stored verbatim as `.js.txt` so formatters do not modify the snapshots. The desktop
fixture contains only its serde enum, with module visibility added for the test.

## Updating published snapshots

1. Confirm the new version is publicly available, not merely uploaded or pending review.
2. Download Chrome's CRX or Firefox's signed XPI using the recorded store URLs. For Firefox,
   query `https://addons.mozilla.org/api/v5/addons/addon/hitsu-password-manager/` for the current
   version and download URL. Chrome's update URL returns the currently published package.
3. Compute SHA-256 of the original package; extract `manifest.json` and `background.js` from
   its ZIP payload (CRX has a header before the ZIP).
4. Save the unchanged background script under its browser/version name. Update its source
   checksum, package checksum, version, and download URL in `versions.json`.
5. Run the gate and review the snapshot diff. Never bump the oldest supported desktop merely
   to bypass a failure; dropping support requires an explicit compatibility decision.

The checksum verification runs offline and detects accidental fixture changes. It does not poll
stores for updates; refresh snapshots after each publication. The test checks request wire-format
compatibility, not browser installation, host registration, real vault access, response compatibility,
DOM filling, or release-mode peer verification. Those still require separate smoke tests.
