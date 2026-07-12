import assert from "node:assert/strict";
import { $, browser } from "@wdio/globals";

const { after, describe, it } = globalThis;
const password = "e2e-master-password";
const vaultPath = process.env.KAGI_E2E_VAULT;

async function button(label) {
  return $(`button=${label}`);
}

async function unlock() {
  const input = await $("#master-pw");
  await input.waitForDisplayed();
  await input.setValue(password);
  await (await button("Unlock")).click();
  await $('[aria-label="Add entry"]').waitForDisplayed();
}

describe("standalone vault smoke test", () => {
  after(async () => {
    if (vaultPath) {
      await browser.execute(async () => {
        await window.__TAURI_INTERNALS__?.invoke("vault_lock");
      });
    }
  });

  it("creates, locks, reopens, and deletes a password entry", async () => {
    assert.ok(vaultPath, "KAGI_E2E_VAULT must be set by the WDIO configuration");

    const setupError = await browser.execute(
      async (path, masterPassword) => {
        try {
          const invoke = window.__TAURI_INTERNALS__.invoke;
          await invoke("vault_create", {
            path,
            password: masterPassword,
            name: "E2E vault",
          });
          await invoke("prefs_set_last_vault", { path });
          return null;
        } catch (error) {
          return String(error);
        }
      },
      vaultPath,
      password,
    );
    assert.equal(setupError, null);

    await browser.refresh();
    await unlock();

    await $('[aria-label="Add entry"]').click();
    await (await button("Password")).click();

    const title = await $('input[placeholder="Title"]');
    await title.waitForDisplayed();
    await title.setValue("E2E password");
    await $('input[placeholder="Password"]').setValue("generated-secret");
    await $('input[placeholder="URL"]').setValue("https://example.com");
    await (await button("Save")).click();
    await $("h1=E2E password").waitForDisplayed();

    await $('[aria-label="Lock vault"]').click();
    await unlock();
    await $("h1=E2E password").waitForDisplayed();

    await browser.refresh();
    await unlock();
    await $("h1=E2E password").waitForDisplayed();

    await $('[aria-label="Edit entry"]').click();
    await (await button("Delete")).click();
    const confirmDelete = await $('//div[@role="dialog"]//button[normalize-space(.)="Delete"]');
    await confirmDelete.waitForDisplayed();
    await confirmDelete.click();

    await $("p=No entries yet").waitForDisplayed();
  });
});
