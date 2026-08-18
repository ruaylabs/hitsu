import { describe, expect, it } from "vitest";
import { createKdfPrompt } from "./kdfPrompt.svelte";

const needsUpgrade = { path: "/vaults/weak.kdbx", kdfNeedsUpgrade: true };
const healthy = { path: "/vaults/healthy.kdbx", kdfNeedsUpgrade: false };
const anotherWeak = { path: "/vaults/weak2.kdbx", kdfNeedsUpgrade: true };

describe("kdf prompt", () => {
  it("shows the prompt for an unlocked vault that needs an upgrade", () => {
    const prompt = createKdfPrompt();

    prompt.update(needsUpgrade, false, []);

    expect(prompt.visible).toBe(true);
  });

  it("does not show the prompt while locked or after dismissal is persisted", () => {
    const prompt = createKdfPrompt();

    prompt.update(needsUpgrade, true, []);
    expect(prompt.visible).toBe(false);

    prompt.update(needsUpgrade, false, ["/vaults/weak.kdbx"]);
    expect(prompt.visible).toBe(false);
  });

  it("dismisses the prompt when switching to a healthy vault", () => {
    const prompt = createKdfPrompt();
    prompt.update(needsUpgrade, false, []);
    expect(prompt.visible).toBe(true);

    prompt.update(healthy, false, []);

    expect(prompt.visible).toBe(false);
  });

  it("shows the prompt again for a different vault that needs an upgrade", () => {
    const prompt = createKdfPrompt();
    prompt.update(needsUpgrade, false, []);
    prompt.update(healthy, false, []);

    prompt.update(anotherWeak, false, []);

    expect(prompt.visible).toBe(true);
  });

  it("re-shows an Esc-dismissed prompt after a lock/unlock cycle, but not a persisted 'Later'", () => {
    const prompt = createKdfPrompt();
    prompt.update(needsUpgrade, false, []);
    // Esc/close only hides for now; the effect re-runs when lock state changes.
    prompt.dismiss();

    prompt.update(needsUpgrade, true, []);
    prompt.update(needsUpgrade, false, []);
    expect(prompt.visible).toBe(true);

    // "Later" persists the dismissal, so the prompt stays gone for this vault.
    prompt.update(needsUpgrade, false, ["/vaults/weak.kdbx"]);
    expect(prompt.visible).toBe(false);
  });
});
