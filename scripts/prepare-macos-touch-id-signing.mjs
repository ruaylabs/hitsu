import { chmodSync, copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const teamId = process.env.APPLE_TEAM_ID?.trim();
const profileBase64 = process.env.APPLE_PROVISIONING_PROFILE?.replace(/\s/g, "");
const profilePath = process.env.APPLE_PROVISIONING_PROFILE_PATH?.trim();
const outputDir = resolve(process.argv[2] ?? ".touch-id-signing");
const bundleId = "com.ruaylabs.hitsu";

if (!teamId || !/^[A-Z0-9]{10}$/.test(teamId)) {
  throw new Error("APPLE_TEAM_ID must be a 10-character Apple Developer Team ID");
}
if (!profileBase64 && !profilePath) {
  throw new Error("Set APPLE_PROVISIONING_PROFILE (base64) or APPLE_PROVISIONING_PROFILE_PATH");
}

mkdirSync(outputDir, { recursive: true, mode: 0o700 });
const entitlementsPath = resolve(outputDir, "Hitsu.entitlements");
const embeddedProfilePath = resolve(outputDir, "embedded.provisionprofile");
const configPath = resolve(outputDir, "tauri.touch-id.conf.json");
const applicationIdentifier = `${teamId}.${bundleId}`;

if (profilePath) {
  copyFileSync(resolve(profilePath), embeddedProfilePath);
} else {
  const profile = Buffer.from(profileBase64, "base64");
  if (profile.length === 0) throw new Error("APPLE_PROVISIONING_PROFILE is not valid base64");
  writeFileSync(embeddedProfilePath, profile, { mode: 0o600 });
}
chmodSync(embeddedProfilePath, 0o600);

writeFileSync(
  entitlementsPath,
  `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.application-identifier</key>
  <string>${applicationIdentifier}</string>
  <key>com.apple.developer.team-identifier</key>
  <string>${teamId}</string>
  <key>keychain-access-groups</key>
  <array>
    <string>${applicationIdentifier}</string>
  </array>
</dict>
</plist>
`,
  { mode: 0o600 },
);

writeFileSync(
  configPath,
  `${JSON.stringify(
    {
      bundle: {
        macOS: {
          entitlements: entitlementsPath,
          files: {
            "embedded.provisionprofile": embeddedProfilePath,
          },
        },
      },
    },
    null,
    2,
  )}\n`,
  { mode: 0o600 },
);

console.log(`Prepared Touch ID signing config: ${configPath}`);
