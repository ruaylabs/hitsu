import { openUrl } from "@tauri-apps/plugin-opener";

/** Open only HTTP(S) URLs, defaulting bare hostnames to HTTPS. */
export function openHttpUrl(rawUrl: string): void {
  const url = rawUrl.includes("://") ? rawUrl : `https://${rawUrl}`;
  try {
    const parsed = new URL(url);
    if (parsed.protocol === "http:" || parsed.protocol === "https:") {
      void openUrl(url);
      return;
    }
    console.warn("Blocked opening URL with disallowed scheme:", parsed.protocol);
  } catch {
    console.warn("Blocked opening invalid URL:", url);
  }
}
