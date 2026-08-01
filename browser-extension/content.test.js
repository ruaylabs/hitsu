import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

let listener;

beforeEach(async () => {
  globalThis.hitsuContentScriptCleanup?.();
  listener = undefined;
  document.body.innerHTML = "";
  globalThis.hitsuContentScriptLoaded = false;
  vi.stubGlobal("chrome", {
    runtime: {
      id: "extension-id",
      lastError: undefined,
      sendMessage: vi.fn(),
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
  globalThis.hitsuContentScriptCleanup?.();
  vi.useRealTimers();
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
  it("offers secure login suggestions and requests the keyboard selection", async () => {
    document.body.innerHTML = `
      <form>
        <input name="email" type="email" autocomplete="username">
        <input name="password" type="password" autocomplete="current-password">
      </form>
    `;
    const [username, password] = document.querySelectorAll("input");
    mockVisibility(username);
    mockVisibility(password);
    chrome.runtime.sendMessage.mockImplementation((message, callback) => {
      if (message.type === "list-logins") {
        callback({
          ok: true,
          entries: [{ id: "entry-1", title: "Example", username: "ada@example.com" }],
        });
      } else if (message.type === "fill-login-inline") {
        callback({ ok: true });
      }
    });

    username.focus();

    await vi.waitFor(() =>
      expect(document.querySelector("hitsu-login-suggestions")).not.toBeNull(),
    );
    const host = document.querySelector("hitsu-login-suggestions");
    expect(host.shadowRoot).toBeNull();
    expect(host.textContent).toBe("");
    expect(host.style.width).toBe("max-content");
    expect(host.style.maxWidth).toBe("calc(100vw - 16px)");
    expect(chrome.runtime.sendMessage).toHaveBeenCalledWith(
      { type: "list-logins" },
      expect.any(Function),
    );

    username.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    username.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

    expect(chrome.runtime.sendMessage).toHaveBeenCalledWith(
      {
        type: "fill-login-inline",
        id: "entry-1",
        requestId: expect.any(String),
      },
      expect.any(Function),
    );
  });

  it("fills only the form where an inline suggestion was selected", async () => {
    document.body.innerHTML = `
      <form id="first-login">
        <input name="first-email" type="email" autocomplete="username">
        <input name="first-password" type="password" autocomplete="current-password">
      </form>
      <form id="second-login">
        <input name="second-email" type="email" autocomplete="username">
        <input name="second-password" type="password" autocomplete="current-password">
      </form>
    `;
    const fields = document.querySelectorAll("input");
    for (const field of fields) mockVisibility(field);
    let inlineMessage;
    let inlineCallback;
    chrome.runtime.sendMessage.mockImplementation((message, callback) => {
      if (message.type === "list-logins") {
        callback({
          ok: true,
          entries: [{ id: "entry-1", title: "Example", username: "ada@example.com" }],
        });
      } else if (message.type === "fill-login-inline") {
        inlineMessage = message;
        inlineCallback = callback;
      }
    });

    fields[2].focus();
    await vi.waitFor(() =>
      expect(document.querySelector("hitsu-login-suggestions")).not.toBeNull(),
    );
    fields[2].dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    fields[2].dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    expect(inlineMessage.requestId).toEqual(expect.any(String));
    const sendResponse = vi.fn();

    expect(
      listener(
        {
          type: "fill-login",
          username: "ada@example.com",
          password: "secret",
          inlineRequestId: inlineMessage.requestId,
        },
        { id: "extension-id" },
        sendResponse,
      ),
    ).toBe(false);
    inlineCallback({ ok: true });

    expect(fields[0].value).toBe("");
    expect(fields[1].value).toBe("");
    expect(fields[2].value).toBe("ada@example.com");
    expect(fields[3].value).toBe("secret");
    expect(document.activeElement).toBe(fields[3]);
    expect(sendResponse).toHaveBeenCalledWith({ ok: true });
  });

  it("keeps suggestions closed when autofill focuses the password field", async () => {
    document.body.innerHTML = `
      <form>
        <input name="email" type="email" autocomplete="username">
        <input name="password" type="password" autocomplete="current-password">
      </form>
    `;
    const [username, password] = document.querySelectorAll("input");
    mockVisibility(username);
    mockVisibility(password);
    chrome.runtime.sendMessage.mockImplementation((message, callback) => {
      if (message.type === "list-logins") {
        callback({
          ok: true,
          entries: [{ id: "entry-1", title: "Example", username: "ada@example.com" }],
        });
      }
    });

    username.focus();
    await vi.waitFor(() =>
      expect(document.querySelector("hitsu-login-suggestions")).not.toBeNull(),
    );
    const host = document.querySelector("hitsu-login-suggestions");
    chrome.runtime.sendMessage.mockClear();

    expect(
      listener(
        {
          type: "fill-login",
          username: "ada@example.com",
          password: "secret",
        },
        { id: "extension-id" },
        vi.fn(),
      ),
    ).toBe(false);

    expect(document.activeElement).toBe(password);
    expect(host.style.display).toBe("none");
    expect(chrome.runtime.sendMessage).not.toHaveBeenCalled();
  });

  it("does not offer suggestions on signup password fields", () => {
    document.body.innerHTML = `
      <form>
        <input name="email" type="email">
        <input name="password" type="password" autocomplete="new-password">
      </form>
    `;
    const fields = document.querySelectorAll("input");
    for (const field of fields) mockVisibility(field);

    fields[1].focus();

    expect(chrome.runtime.sendMessage).not.toHaveBeenCalled();
  });

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

  it("fills login fields inside nested open shadow roots", () => {
    document.body.innerHTML = "<login-shell></login-shell>";
    const shell = document.querySelector("login-shell").attachShadow({ mode: "open" });
    shell.innerHTML = "<login-form></login-form>";
    const loginForm = shell.querySelector("login-form").attachShadow({ mode: "open" });
    loginForm.innerHTML = `
      <form>
        <input name="username" type="email" autocomplete="username">
        <input name="password" type="password" autocomplete="current-password">
      </form>
    `;
    const username = loginForm.querySelector('[name="username"]');
    const password = loginForm.querySelector('[name="password"]');
    mockVisibility(password);
    const sendResponse = vi.fn();

    listener(
      { type: "fill-login", username: "ada@example.com", password: "secret" },
      { id: "extension-id" },
      sendResponse,
    );

    expect(username.value).toBe("ada@example.com");
    expect(password.value).toBe("secret");
    expect(loginForm.activeElement).toBe(password);
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

  it("fills a login form rendered after the fill request", async () => {
    const sendResponse = vi.fn();

    expect(
      listener(
        { type: "fill-login", username: "ada@example.com", password: "secret" },
        { id: "extension-id" },
        sendResponse,
      ),
    ).toBe(true);
    expect(sendResponse).not.toHaveBeenCalled();

    document.body.innerHTML = `
      <form>
        <input name="username" type="email" autocomplete="username">
        <input name="password" type="password" autocomplete="current-password">
      </form>
    `;
    const username = document.querySelector('[name="username"]');
    const password = document.querySelector('[name="password"]');
    mockVisibility(password);

    await vi.waitFor(() => expect(sendResponse).toHaveBeenCalledWith({ ok: true }));
    expect(username.value).toBe("ada@example.com");
    expect(password.value).toBe("secret");
  });

  it("does not treat an unrelated search input as a username step", async () => {
    vi.useFakeTimers();
    document.body.innerHTML = '<input name="q" type="search">';
    const search = document.querySelector("input");
    mockVisibility(search);
    const sendResponse = vi.fn();

    expect(
      listener(
        { type: "fill-login", username: "ada", password: "secret" },
        { id: "extension-id" },
        sendResponse,
      ),
    ).toBe(true);
    await vi.advanceTimersByTimeAsync(5_000);

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

  it("immediately refuses a signup form with new-password fields", () => {
    document.body.innerHTML = `
      <form id="signup">
        <input name="email" type="email">
        <input name="new-password" type="password" autocomplete="new-password">
        <input name="confirm-password" type="password" autocomplete="new-password">
      </form>
    `;
    const fields = document.querySelectorAll("input");
    for (const f of fields) mockVisibility(f);
    const sendResponse = vi.fn();

    expect(
      listener(
        { type: "fill-login", username: "ada", password: "secret" },
        { id: "extension-id" },
        sendResponse,
      ),
    ).toBe(false);

    expect(fields[0].value).toBe("");
    expect(fields[1].value).toBe("");
    expect(fields[2].value).toBe("");
    expect(sendResponse).toHaveBeenCalledWith({
      ok: false,
      error: "Hitsu does not fill signup forms",
    });
  });

  it("refuses a single new-password field", () => {
    document.body.innerHTML = `
      <form>
        <input name="email" type="email">
        <input name="password" type="password" autocomplete="new-password">
      </form>
    `;
    const fields = document.querySelectorAll("input");
    for (const field of fields) mockVisibility(field);
    const sendResponse = vi.fn();

    expect(
      listener(
        { type: "fill-login", username: "ada", password: "secret" },
        { id: "extension-id" },
        sendResponse,
      ),
    ).toBe(false);

    expect(fields[0].value).toBe("");
    expect(fields[1].value).toBe("");
    expect(sendResponse).toHaveBeenCalledWith({
      ok: false,
      error: "Hitsu does not fill signup forms",
    });
  });

  it("immediately refuses ambiguous password fields", () => {
    document.body.innerHTML = `
      <form>
        <input name="username" type="text">
        <input name="password-one" type="password">
        <input name="password-two" type="password">
      </form>
    `;
    const fields = document.querySelectorAll("input");
    for (const field of fields) mockVisibility(field);
    const sendResponse = vi.fn();

    expect(
      listener(
        { type: "fill-login", username: "ada", password: "secret" },
        { id: "extension-id" },
        sendResponse,
      ),
    ).toBe(false);

    expect([...fields].every((field) => field.value === "")).toBe(true);
    expect(sendResponse).toHaveBeenCalledWith({
      ok: false,
      error: "Hitsu could not determine which password field to fill",
    });
  });

  it("picks current-password among multiple password fields in a change-password form", () => {
    document.body.innerHTML = `
      <form id="change-password">
        <input name="username" type="text" autocomplete="username">
        <input name="current" type="password" autocomplete="current-password">
        <input name="new1" type="password" autocomplete="new-password">
        <input name="new2" type="password" autocomplete="new-password">
      </form>
    `;
    const fields = document.querySelectorAll("input");
    for (const f of fields) mockVisibility(f);
    const sendResponse = vi.fn();

    listener(
      { type: "fill-login", username: "ada", password: "secret" },
      { id: "extension-id" },
      sendResponse,
    );

    // Only the current-password field gets filled
    expect(fields[0].value).toBe("ada");
    expect(fields[1].value).toBe("secret");
    expect(fields[2].value).toBe("");
    expect(fields[3].value).toBe("");
    expect(sendResponse).toHaveBeenCalledWith({ ok: true });
  });

  it("fills a lone password field with no autocomplete hint", () => {
    document.body.innerHTML = `
      <form>
        <input name="username" type="text">
        <input name="password" type="password">
      </form>
    `;
    const fields = document.querySelectorAll("input");
    for (const f of fields) mockVisibility(f);
    const sendResponse = vi.fn();

    listener(
      { type: "fill-login", username: "ada", password: "secret" },
      { id: "extension-id" },
      sendResponse,
    );

    expect(fields[0].value).toBe("ada");
    expect(fields[1].value).toBe("secret");
    expect(sendResponse).toHaveBeenCalledWith({ ok: true });
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

  it("silently ignores a fill message whose expectedOrigin does not match", () => {
    document.body.innerHTML = `
      <form>
        <input name="username" type="email">
        <input name="password" type="password">
      </form>
    `;
    const password = document.querySelector('input[type="password"]');
    mockVisibility(password);
    const sendResponse = vi.fn();

    expect(
      listener(
        {
          type: "fill-login",
          username: "ada",
          password: "secret",
          expectedOrigin: "https://example.com",
        },
        { id: "extension-id" },
        sendResponse,
      ),
    ).toBe(false);

    // Fields must remain untouched
    expect(document.querySelector('[name="username"]').value).toBe("");
    expect(password.value).toBe("");
    expect(sendResponse).not.toHaveBeenCalled();
  });

  it("fills when expectedOrigin matches the current frame origin", () => {
    // content.js tests run with about:blank origin; no expectedOrigin means
    // the check is skipped (backward-compatible with older callers).
    document.body.innerHTML = `
      <form>
        <input name="username" type="email">
        <input name="password" type="password">
      </form>
    `;
    const password = document.querySelector('input[type="password"]');
    mockVisibility(password);
    const sendResponse = vi.fn();

    // Omit expectedOrigin to simulate a pre-iframe-support caller
    expect(
      listener(
        { type: "fill-login", username: "ada", password: "secret" },
        { id: "extension-id" },
        sendResponse,
      ),
    ).toBe(false);

    expect(document.querySelector('[name="username"]').value).toBe("ada");
    expect(password.value).toBe("secret");
    expect(sendResponse).toHaveBeenCalledWith({ ok: true });
  });

  it("reports when no writable password field appears before the retry deadline", async () => {
    vi.useFakeTimers();
    document.body.innerHTML = '<input type="password" readonly>';
    const sendResponse = vi.fn();

    expect(
      listener(
        { type: "fill-login", username: "ada", password: "secret" },
        { id: "extension-id" },
        sendResponse,
      ),
    ).toBe(true);
    await vi.advanceTimersByTimeAsync(5_000);

    expect(sendResponse).toHaveBeenCalledWith({
      ok: false,
      error: "No password field found on this page",
    });
  });
});
