import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

let listener;

beforeEach(async () => {
  listener = undefined;
  document.body.innerHTML = "";
  globalThis.hitsuContentScriptLoaded = false;
  vi.stubGlobal("chrome", {
    runtime: {
      id: "extension-id",
      onMessage: {
        addListener: vi.fn((registered) => {
          listener = registered;
        }),
      },
    },
  });
  vi.resetModules();
  await import("./content.js");
});

afterEach(() => {
  vi.unstubAllGlobals();
  document.body.innerHTML = "";
});

describe("login filling", () => {
  it("fills the matching username and password fields and dispatches form events", () => {
    document.body.innerHTML = `
      <form>
        <input name="account_email" type="email">
        <input name="password" type="password">
      </form>
    `;
    const username = document.querySelector('input[type="email"]');
    const password = document.querySelector('input[type="password"]');
    const usernameInput = vi.fn();
    const passwordChange = vi.fn();
    username.addEventListener("input", usernameInput);
    password.addEventListener("change", passwordChange);
    const sendResponse = vi.fn();

    expect(
      listener(
        { type: "fill-login", username: "ada@example.com", password: "correct horse" },
        { id: "extension-id" },
        sendResponse,
      ),
    ).toBe(false);

    expect(username.value).toBe("ada@example.com");
    expect(password.value).toBe("correct horse");
    expect(usernameInput).toHaveBeenCalledOnce();
    expect(passwordChange).toHaveBeenCalledOnce();
    expect(document.activeElement).toBe(password);
    expect(sendResponse).toHaveBeenCalledWith({ ok: true });
  });

  it("uses the nearest text field before the password as a fallback", () => {
    document.body.innerHTML = `
      <form>
        <input name="search" type="text">
        <input name="identifier" type="text">
        <input type="password">
      </form>
    `;
    const fields = document.querySelectorAll("input");

    listener(
      { type: "fill-login", username: "ada", password: "secret" },
      { id: "extension-id" },
      vi.fn(),
    );

    expect(fields[0].value).toBe("");
    expect(fields[1].value).toBe("ada");
    expect(fields[2].value).toBe("secret");
  });

  it("rejects messages not sent by the extension itself", () => {
    document.body.innerHTML = '<input type="password">';
    const password = document.querySelector("input");
    const sendResponse = vi.fn();

    expect(
      listener(
        { type: "fill-login", username: "ada", password: "secret" },
        { id: "other-extension" },
        sendResponse,
      ),
    ).toBe(false);
    expect(password.value).toBe("");
    expect(sendResponse).not.toHaveBeenCalled();
  });

  it("reports when the page has no writable password field", () => {
    document.body.innerHTML = '<input type="password" readonly>';
    const sendResponse = vi.fn();

    listener(
      { type: "fill-login", username: "ada", password: "secret" },
      { id: "extension-id" },
      sendResponse,
    );

    expect(sendResponse).toHaveBeenCalledWith({
      ok: false,
      error: "No password field found on this page",
    });
  });
});
