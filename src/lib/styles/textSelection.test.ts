import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");
const controls = read("./controls.css");
const generator = read("../components/generator/GeneratorPanel.svelte");
const unlock = read("../components/ui/PasswordDialog.svelte");
const settings = read("../components/settings/SettingsView.svelte");
const reset = read("./reset.css");

function declaration(source: string, selector: string) {
  const rules = [...source.matchAll(/([^{}]+)\{([^{}]*)\}/g)];
  return rules
    .filter(([, selectors]) => selectors.split(",").some((item) => item.trim() === selector))
    .map(([, , body]) => body)
    .join("\n");
}

describe("rendered text selection", () => {
  it.each([".control-error", ".save-error", ".generator-error"])(
    "allows selection of %s across components",
    (selector) => {
      expect(declaration(controls, selector)).toContain("user-select: text;");
    },
  );

  it("allows selection of the generated password without selecting its controls", () => {
    expect(declaration(generator, ".generated-pw")).toContain("user-select: text;");
    expect(declaration(generator, ".generator-error")).toContain("user-select: text;");
    expect(declaration(generator, ".generator-actions")).not.toContain("user-select: text;");
  });

  it("allows selection of vault paths in unlock and settings", () => {
    expect(declaration(unlock, ".vault-path")).toContain("user-select: text;");
    expect(declaration(settings, ".vault-path-sub")).toContain("user-select: text;");
  });

  it("keeps selection opt-in rather than enabling it for all app chrome", () => {
    expect(declaration(reset, "body")).toContain("user-select: none;");
    expect(declaration(controls, ".control")).not.toContain("user-select: text;");
  });
});
