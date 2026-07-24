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

function mockVisibility(
  input,
  { checkVisibility = true, offsetParent = true, bounds = true } = {},
) {
  input.checkVisibility = vi.fn(() => checkVisibility);
  Object.defineProperty(input, "offsetParent", {
    configurable: true,
    value: offsetParent ? document.body : null,
  });
  input.getBoundingClientRect = vi.fn(() => ({
    width: bounds ? 200 : 0,
    height: bounds ? 30 : 0,
  }));
}

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
    mockVisibility(password);
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
    mockVisibility(fields[2]);

    listener(
      { type: "fill-login", username: "ada", password: "secret" },
      { id: "extension-id" },
      vi.fn(),
    );

    expect(fields[0].value).toBe("");
    expect(fields[1].value).toBe("ada");
    expect(fields[2].value).toBe("secret");
  });

  it("fills a Google-style username step and waits for its password step", async () => {
    document.body.innerHTML = `
      <form id="gaia_loginform">
        <input
          id="identifierId"
          name="identifier"
          type="email"
          autocomplete="username"
          aria-label="Email or phone"
        >
      </form>
    `;
    const username = document.querySelector("#identifierId");
    mockVisibility(username);
    const usernameInput = vi.fn();
    username.addEventListener("input", usernameInput);
    const sendResponse = vi.fn();

    listener(
      { type: "fill-login", username: "ada@example.com", password: "secret" },
      { id: "extension-id" },
      sendResponse,
    );

    expect(username.value).toBe("ada@example.com");
    expect(usernameInput).toHaveBeenCalledOnce();
    expect(document.activeElement).toBe(username);
    expect(sendResponse).toHaveBeenCalledWith({ ok: true, usernameOnly: true });

    document.querySelector("#gaia_loginform").innerHTML = `
      <input name="password" type="password" autocomplete="current-password">
    `;
    const password = document.querySelector('input[type="password"]');
    mockVisibility(password);

    await vi.waitFor(() => expect(password.value).toBe("secret"));
    expect(document.activeElement).toBe(password);
  });

  it("does not treat an unrelated search input as a username step", () => {
    document.body.innerHTML = '<input name="q" type="search">';
    const search = document.querySelector("input");
    mockVisibility(search);
    const sendResponse = vi.fn();

    listener(
      { type: "fill-login", username: "ada", password: "secret" },
      { id: "extension-id" },
      sendResponse,
    );

    expect(search.value).toBe("");
    expect(sendResponse).toHaveBeenCalledWith({
      ok: false,
      error: "No password field found on this page",
    });
  });

  it.each([
    ["checkVisibility", { checkVisibility: false }],
    ["offsetParent", { offsetParent: false }],
    ["bounding rectangle", { bounds: false }],
  ])("skips a password field hidden by %s", (_check, hiddenState) => {
    document.body.innerHTML = `
      <form id="hidden-form">
        <input name="hidden-user" type="email">
        <input name="hidden-password" type="password">
      </form>
      <form id="login-form">
        <input name="login-user" type="email">
        <input name="login-password" type="password">
      </form>
    `;
    const passwords = document.querySelectorAll('input[type="password"]');
    mockVisibility(passwords[0], hiddenState);
    mockVisibility(passwords[1]);

    listener(
      { type: "fill-login", username: "ada@example.com", password: "secret" },
      { id: "extension-id" },
      vi.fn(),
    );

    expect(document.querySelector('[name="hidden-user"]').value).toBe("");
    expect(passwords[0].value).toBe("");
    expect(document.querySelector('[name="login-user"]').value).toBe("ada@example.com");
    expect(passwords[1].value).toBe("secret");
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
