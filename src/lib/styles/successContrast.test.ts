import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");
const tokens = read("./tokens.css");
const blocks = [...tokens.matchAll(/([^{}]+)\{([^{}]*)\}/g)];

function declarations(selector: string) {
  const block = blocks.find((match) => match[1].trim() === selector);
  expect(block).toBeDefined();
  return Object.fromEntries(
    [...(block?.[2] ?? "").matchAll(/(--[\w-]+):\s*([^;]+);/g)].map((m) => [m[1], m[2]]),
  );
}

function luminance(hex: string) {
  const channels = hex.match(/^#([\da-f]{2})([\da-f]{2})([\da-f]{2})$/i);
  if (!channels) throw new Error(`Expected an opaque hex color, received ${hex}`);
  const linear = channels.slice(1).map((channel) => {
    const value = Number.parseInt(channel, 16) / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
}

describe("success text contrast", () => {
  const base = declarations(":root");
  for (const selector of [":root", ':root[data-theme="dark"]', ":root:not([data-theme])"]) {
    const colors = { ...base, ...declarations(selector) };
    for (const surface of ["--surface-0", "--surface-1", "--surface-2"]) {
      it(`${selector} meets 4.5:1 on ${surface}`, () => {
        const text = luminance(colors["--success-text"]);
        const background = luminance(colors[surface]);
        expect(
          (Math.max(text, background) + 0.05) / (Math.min(text, background) + 0.05),
        ).toBeGreaterThanOrEqual(4.5);
      });
    }
  }

  it("uses the text token for import status and the TOTP success icon, not indicator fills", () => {
    const settings = read("../components/settings/SettingsView.svelte");
    const totp = read("../components/ui/TotpSetupDialog.svelte");
    expect(settings).toMatch(/\.import-summary-heading\s*\{[^}]*color: var\(--success-text\)/);
    expect(totp).toMatch(
      /\.image-picker--success :global\(\.ti\)\s*\{[^}]*color: var\(--success-text\)/,
    );
    expect(totp).toMatch(/\.image-picker--success\s*\{[^}]*border-color: var\(--success\)/);
    expect(base["--success"]).toBe("#1d9e75");
  });
});
