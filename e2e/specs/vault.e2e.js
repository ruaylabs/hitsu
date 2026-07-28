import assert from "node:assert/strict";
import { $, browser } from "@wdio/globals";

const { after, describe, it } = globalThis;
const password = "e2e-master-password";
const vaultPath = process.env.HITSU_E2E_VAULT;

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

async function saveEntry(expectedTitle) {
  const save = await button("Save");
  await save.waitForEnabled();
  await save.click();
  await browser.waitUntil(
    async () => {
      const state = await browser.execute((title) => {
        const heading = [...document.querySelectorAll("h1")].some(
          (element) => element.textContent?.trim() === title,
        );
        const error = document.querySelector(".save-error")?.textContent?.trim() ?? "";
        return { heading, error };
      }, expectedTitle);
      if (state.error) throw new Error(`Entry save failed: ${state.error}`);
      return state.heading;
    },
    {
      timeout: 45_000,
      interval: 200,
      timeoutMsg: `Entry save did not finish for "${expectedTitle}"`,
    },
  );
}

describe("standalone vault lifecycle", () => {
  after(async () => {
    if (vaultPath) {
      await browser.execute(async () => {
        await window.__TAURI_INTERNALS__?.invoke("vault_lock");
      });
    }
  });

  it("unlocks, creates, edits, locks, reopens, and deletes an entry", async () => {
    assert.ok(vaultPath, "HITSU_E2E_VAULT must be set by the WDIO configuration");

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
    const passwordType = await $('[role="option"][aria-label="Password"]');
    await passwordType.waitForDisplayed();
    await passwordType.click();

    const title = await $('input[placeholder="Title"]');
    await title.waitForDisplayed();
    await title.setValue("E2E password");
    await $('input[placeholder="Password"]').setValue("generated-secret");
    await $('input[placeholder="URL"]').setValue("https://example.com");
    await saveEntry("E2E password");

    await $('[aria-label="Edit entry"]').click();
    const editedTitle = await $('input[placeholder="Title"]');
    await editedTitle.waitForDisplayed();
    await editedTitle.setValue("E2E password updated");
    await $('input[placeholder="URL"]').setValue("https://example.org/updated");
    await saveEntry("E2E password updated");

    await $('[aria-label="Lock vault"]').click();
    await unlock();
    await $("h1=E2E password updated").waitForDisplayed();

    await browser.refresh();
    await unlock();
    await $("h1=E2E password updated").waitForDisplayed();

    await $('[aria-label="Edit entry"]').click();
    await (await button("Delete")).click();

    await (await button("Undo")).waitForDisplayed();
    await $("p=No entries yet").waitForDisplayed();
    await (await button("Recycle Bin 1")).click();
    await $("h1=E2E password updated").waitForDisplayed();
  });
});
