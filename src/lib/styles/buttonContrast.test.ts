import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");
const tokens = read("./tokens.css");
const button = read("../components/ui/Button.svelte");
const blocks = [...tokens.matchAll(/([^{}]+)\{([^{}]*)\}/g)];

function declarations(body: string) {
  return Object.fromEntries([...body.matchAll(/(--[\w-]+):\s*([^;]+);/g)].map((m) => [m[1], m[2]]));
}

function luminance(hex: string, brightness: number) {
  const channels = hex.match(/^#([\da-f]{2})([\da-f]{2})([\da-f]{2})$/i);
  if (!channels) throw new Error(`Expected an opaque hex color, received ${hex}`);
  const linear = channels.slice(1).map((channel) => {
    const value = (Number.parseInt(channel, 16) / 255) * brightness;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
}

function rule(selector: string) {
  const start = button.indexOf(`${selector} {`);
  expect(start).toBeGreaterThanOrEqual(0);
  return button.slice(start, button.indexOf("}", start));
}

describe("filled button contrast", () => {
  const base = declarations(blocks.find((m) => m[1].trim() === ":root")?.[2] ?? "");
  for (const theme of ["light", "explicit dark", "system dark"]) {
    const selector =
      theme === "explicit dark" ? ':root[data-theme="dark"]' : ":root:not([data-theme])";
    const overrides =
      theme === "light"
        ? {}
        : declarations(blocks.find((m) => m[1].trim() === selector)?.[2] ?? "");
    const colors = { ...base, ...overrides };

    for (const variant of ["primary", "danger"]) {
      for (const state of ["default", "focus", "hover", "pressed", "hover-pressed"]) {
        it(`${theme} ${variant} ${state} meets 4.5:1`, () => {
          const background =
            colors[`--button-${variant}-${state.includes("hover") ? "hover" : "bg"}`];
          const brightness = state.includes("pressed") ? 0.92 : 1;
          const label = luminance(colors["--text-on-accent"], brightness);
          const fill = luminance(background, brightness);
          expect(
            (Math.max(label, fill) + 0.05) / (Math.min(label, fill) + 0.05),
          ).toBeGreaterThanOrEqual(4.5);
        });
      }
    }
  }

  it("uses opaque fill tokens without fading labels on hover", () => {
    for (const variant of ["primary", "danger"]) {
      const normal = rule(`.button-${variant}`);
      expect(normal).toContain("color: var(--text-on-accent)");
      expect(normal).toContain(`background: var(--button-${variant}-bg)`);
      const hover = rule(`.button-${variant}:hover:not(:disabled)`);
      expect(hover).toContain(`background: var(--button-${variant}-hover)`);
      expect(hover).not.toMatch(/opacity|filter/);
    }
    expect(rule(".button:active:not(:disabled)")).toContain("filter: brightness(0.92)");
    expect(button).toContain(".button.is-active:not(.button-primary):not(.button-danger)");
  });
});
