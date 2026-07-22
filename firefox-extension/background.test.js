import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

let chromeMock;
let listener;

beforeEach(async () => {
  listener = undefined;
  chromeMock = {
    runtime: {
      id: "hitsu@ruaylabs.com",
      lastError: null,
      onMessage: {
        addListener: vi.fn((registered) => {
          listener = registered;
        }),
      },
      sendNativeMessage: vi.fn(),
    },
    scripting: { executeScript: vi.fn().mockResolvedValue(undefined) },
    tabs: {
      get: vi.fn(),
      query: vi.fn(),
      sendMessage: vi.fn(),
    },
  };
  vi.stubGlobal("chrome", chromeMock);
  vi.resetModules();
  await import("./background.js");
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("Firefox background integration", () => {
  it("requests login summaries for the exact active-tab origin", async () => {
    chromeMock.tabs.query.mockResolvedValue([
      { id: 7, url: "https://accounts.example.com/login?next=%2Fvault" },
    ]);
    chromeMock.runtime.sendNativeMessage.mockImplementation((_host, _message, callback) => {
      callback({ ok: true, entries: [{ id: "entry", title: "Example", username: "ada" }] });
    });
    const sendResponse = vi.fn();

    expect(listener({ type: "list-logins" }, { id: "hitsu@ruaylabs.com" }, sendResponse)).toBe(
      true,
    );
    await vi.waitFor(() => expect(sendResponse).toHaveBeenCalled());

    expect(chromeMock.runtime.sendNativeMessage).toHaveBeenCalledWith(
      "com.ruaylabs.hitsu.browser",
      { type: "listLogins", origin: "https://accounts.example.com" },
      expect.any(Function),
    );
    expect(sendResponse).toHaveBeenCalledWith({
      ok: true,
      entries: [{ id: "entry", title: "Example", username: "ada" }],
    });
  });

  it("rejects non-HTTP pages before contacting the native host", async () => {
    chromeMock.tabs.query.mockResolvedValue([{ id: 7, url: "about:logins" }]);
    const sendResponse = vi.fn();

    listener({ type: "list-logins" }, { id: "hitsu@ruaylabs.com" }, sendResponse);
    await vi.waitFor(() => expect(sendResponse).toHaveBeenCalled());

    expect(chromeMock.runtime.sendNativeMessage).not.toHaveBeenCalled();
    expect(sendResponse).toHaveBeenCalledWith({
      ok: false,
      error: "Hitsu can only fill HTTP and HTTPS pages",
    });
  });

  it("fills credentials only after rechecking the page origin", async () => {
    chromeMock.tabs.query.mockResolvedValue([{ id: 7, url: "https://example.com/login" }]);
    chromeMock.tabs.get.mockResolvedValue({ id: 7, url: "https://example.com/account" });
    chromeMock.runtime.sendNativeMessage.mockImplementation((_host, _message, callback) => {
      callback({ ok: true, username: "ada", password: "secret" });
    });
    chromeMock.tabs.sendMessage.mockResolvedValue({ ok: true });
    const sendResponse = vi.fn();

    listener({ type: "fill-login", id: "entry" }, { id: "hitsu@ruaylabs.com" }, sendResponse);
    await vi.waitFor(() => expect(sendResponse).toHaveBeenCalledWith({ ok: true }));

    expect(chromeMock.runtime.sendNativeMessage).toHaveBeenCalledWith(
      "com.ruaylabs.hitsu.browser",
      { type: "getCredentials", id: "entry", origin: "https://example.com" },
      expect.any(Function),
    );
    expect(chromeMock.scripting.executeScript).toHaveBeenCalledWith({
      target: { tabId: 7 },
      files: ["content.js"],
    });
    expect(chromeMock.tabs.sendMessage).toHaveBeenCalledWith(7, {
      type: "fill-login",
      username: "ada",
      password: "secret",
    });
  });

  it("aborts filling if the tab navigates to another origin", async () => {
    chromeMock.tabs.query.mockResolvedValue([{ id: 7, url: "https://example.com/login" }]);
    chromeMock.tabs.get.mockResolvedValue({ id: 7, url: "https://attacker.test/login" });
    chromeMock.runtime.sendNativeMessage.mockImplementation((_host, _message, callback) => {
      callback({ ok: true, username: "ada", password: "secret" });
    });
    const sendResponse = vi.fn();

    listener({ type: "fill-login", id: "entry" }, { id: "hitsu@ruaylabs.com" }, sendResponse);
    await vi.waitFor(() => expect(sendResponse).toHaveBeenCalled());

    expect(sendResponse).toHaveBeenCalledWith({
      ok: false,
      error: "The page changed before Hitsu could fill it",
    });
    expect(chromeMock.scripting.executeScript).not.toHaveBeenCalled();
    expect(chromeMock.tabs.sendMessage).not.toHaveBeenCalled();
  });
});
