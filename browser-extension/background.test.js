import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

let activeHttpTab;
let chromeMock;
let listener;
let loginEntries;
let pageMatchesOrigin;

beforeEach(async () => {
  listener = undefined;
  chromeMock = {
    runtime: {
      id: "extension-id",
      lastError: null,
      onMessage: {
        addListener: vi.fn((registered) => {
          listener = registered;
        }),
      },
      sendNativeMessage: vi.fn(),
    },
    scripting: { executeScript: vi.fn() },
    tabs: {
      get: vi.fn(),
      query: vi.fn(),
      sendMessage: vi.fn(),
    },
  };
  vi.stubGlobal("chrome", chromeMock);
  vi.resetModules();
  ({ activeHttpTab, loginEntries, pageMatchesOrigin } = await import("./background.js"));
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("active tab origin validation", () => {
  it("returns only the exact HTTP origin", async () => {
    chromeMock.tabs.query.mockResolvedValue([
      { id: 42, url: "https://accounts.example.com/login?next=%2Fvault" },
    ]);

    await expect(activeHttpTab()).resolves.toEqual({
      id: 42,
      origin: "https://accounts.example.com",
    });
    expect(chromeMock.tabs.query).toHaveBeenCalledWith({
      active: true,
      currentWindow: true,
    });
  });

  it.each([
    "chrome://extensions",
    "file:///tmp/login.html",
    "javascript:alert(1)",
  ])("rejects non-web tab URL %s", async (url) => {
    chromeMock.tabs.query.mockResolvedValue([{ id: 42, url }]);
    await expect(activeHttpTab()).rejects.toThrow("Hitsu can only fill HTTP and HTTPS pages");
  });

  it("rejects a tab without a usable ID or URL", async () => {
    chromeMock.tabs.query.mockResolvedValue([{}]);
    await expect(activeHttpTab()).rejects.toThrow("No active browser tab found");
  });
});

describe("fill-time origin validation", () => {
  it("allows navigation within the exact origin", () => {
    expect(pageMatchesOrigin("https://example.com/account", "https://example.com")).toBe(true);
  });

  it.each([
    "https://example.com.attacker.test/login",
    "http://example.com/login",
    "chrome://example.com",
    "not a URL",
    undefined,
  ])("rejects a changed or invalid page URL %s", (url) => {
    expect(pageMatchesOrigin(url, "https://example.com")).toBe(false);
  });
});

describe("native response validation", () => {
  it("accepts only well-formed login summaries", () => {
    expect(
      loginEntries({ entries: [{ id: "id", title: "Example", username: "ada" }] }),
    ).toHaveLength(1);
    expect(() => loginEntries({ entries: [{ id: "id", title: "Example" }] })).toThrow(
      "Hitsu returned an invalid login list",
    );
  });
});

describe("background integration", () => {
  it("requests login summaries for the exact active-tab origin", async () => {
    chromeMock.tabs.query.mockResolvedValue([
      { id: 7, url: "https://accounts.example.com/login?next=%2Fvault" },
    ]);
    chromeMock.runtime.sendNativeMessage.mockImplementation((_host, _message, callback) => {
      callback({ ok: true, entries: [{ id: "entry", title: "Example", username: "ada" }] });
    });
    const sendResponse = vi.fn();

    expect(listener({ type: "list-logins" }, { id: "extension-id" }, sendResponse)).toBe(true);
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

    listener({ type: "list-logins" }, { id: "extension-id" }, sendResponse);
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
    // Frame discovery: top frame matches, one cross-origin child is excluded
    chromeMock.scripting.executeScript
      .mockResolvedValueOnce([
        { frameId: 0, result: "https://example.com" },
        { frameId: 42, result: "https://accounts.google.com" },
      ])
      .mockResolvedValueOnce(undefined);
    chromeMock.tabs.sendMessage.mockResolvedValue({ ok: true });
    const sendResponse = vi.fn();

    listener({ type: "fill-login", id: "entry" }, { id: "extension-id" }, sendResponse);
    await vi.waitFor(() => expect(sendResponse).toHaveBeenCalledWith({ ok: true }));

    expect(chromeMock.runtime.sendNativeMessage).toHaveBeenCalledWith(
      "com.ruaylabs.hitsu.browser",
      { type: "getCredentials", id: "entry", origin: "https://example.com" },
      expect.any(Function),
    );
    // First call: frame discovery (allFrames)
    expect(chromeMock.scripting.executeScript).toHaveBeenNthCalledWith(1, {
      target: { tabId: 7, allFrames: true },
      func: expect.any(Function),
    });
    // Second call: inject content.js only into same-origin frames
    expect(chromeMock.scripting.executeScript).toHaveBeenNthCalledWith(2, {
      target: { tabId: 7, frameIds: [0] },
      files: ["content.js"],
    });
    // Sends fill message with expectedOrigin to each matching frame
    expect(chromeMock.tabs.sendMessage).toHaveBeenCalledWith(
      7,
      {
        type: "fill-login",
        username: "ada",
        password: "secret",
        expectedOrigin: "https://example.com",
      },
      { frameId: 0 },
    );
  });

  it("injects and fills all same-origin frames including iframes", async () => {
    chromeMock.tabs.query.mockResolvedValue([{ id: 7, url: "https://example.com/login" }]);
    chromeMock.tabs.get.mockResolvedValue({ id: 7, url: "https://example.com/login" });
    chromeMock.runtime.sendNativeMessage.mockImplementation((_host, _message, callback) => {
      callback({ ok: true, username: "ada", password: "secret" });
    });
    // Discovery: frames 0 and 24 are same-origin, frame 99 is cross-origin
    chromeMock.scripting.executeScript
      .mockResolvedValueOnce([
        { frameId: 0, result: "https://example.com" },
        { frameId: 24, result: "https://example.com" },
        { frameId: 99, result: "https://idp.other.test" },
      ])
      .mockResolvedValueOnce(undefined);
    chromeMock.tabs.sendMessage.mockResolvedValue({ ok: true });
    const sendResponse = vi.fn();

    listener({ type: "fill-login", id: "entry" }, { id: "extension-id" }, sendResponse);
    await vi.waitFor(() => expect(sendResponse).toHaveBeenCalledWith({ ok: true }));

    // Frame 99 is cross-origin and must be excluded
    expect(chromeMock.scripting.executeScript).toHaveBeenNthCalledWith(2, {
      target: { tabId: 7, frameIds: [0, 24] },
      files: ["content.js"],
    });
    expect(chromeMock.tabs.sendMessage).toHaveBeenCalledTimes(2);
    expect(chromeMock.tabs.sendMessage).toHaveBeenCalledWith(
      7,
      {
        type: "fill-login",
        username: "ada",
        password: "secret",
        expectedOrigin: "https://example.com",
      },
      { frameId: 0 },
    );
    expect(chromeMock.tabs.sendMessage).toHaveBeenCalledWith(
      7,
      {
        type: "fill-login",
        username: "ada",
        password: "secret",
        expectedOrigin: "https://example.com",
      },
      { frameId: 24 },
    );
  });

  it("reports failure when no frame matches the entry origin", async () => {
    chromeMock.tabs.query.mockResolvedValue([{ id: 7, url: "https://example.com/login" }]);
    chromeMock.tabs.get.mockResolvedValue({ id: 7, url: "https://example.com/login" });
    chromeMock.runtime.sendNativeMessage.mockImplementation((_host, _message, callback) => {
      callback({ ok: true, username: "ada", password: "secret" });
    });
    // All frames are cross-origin (e.g., page is a shell embedding foreign widgets)
    chromeMock.scripting.executeScript.mockResolvedValueOnce([
      { frameId: 0, result: "https://idp.other.test" },
      { frameId: 1, result: "https://cdn.different.test" },
    ]);
    const sendResponse = vi.fn();

    listener({ type: "fill-login", id: "entry" }, { id: "extension-id" }, sendResponse);
    await vi.waitFor(() => expect(sendResponse).toHaveBeenCalled());

    expect(sendResponse).toHaveBeenCalledWith({
      ok: false,
      error: "No matching frames found on this page",
    });
    // Discovery call did happen, but no injection since no frames matched
    expect(chromeMock.scripting.executeScript).toHaveBeenCalledTimes(1);
    expect(chromeMock.scripting.executeScript).toHaveBeenCalledWith({
      target: { tabId: 7, allFrames: true },
      func: expect.any(Function),
    });
  });

  it("succeeds when at least one matching frame fills successfully", async () => {
    chromeMock.tabs.query.mockResolvedValue([{ id: 7, url: "https://example.com/login" }]);
    chromeMock.tabs.get.mockResolvedValue({ id: 7, url: "https://example.com/login" });
    chromeMock.runtime.sendNativeMessage.mockImplementation((_host, _message, callback) => {
      callback({ ok: true, username: "ada", password: "secret" });
    });
    // Discovery: both frames match the entry origin
    chromeMock.scripting.executeScript
      .mockResolvedValueOnce([
        { frameId: 0, result: "https://example.com" },
        { frameId: 7, result: "https://example.com" },
      ])
      .mockResolvedValueOnce(undefined);
    // Frame 0 has a form, frame 7 does not
    chromeMock.tabs.sendMessage
      .mockResolvedValueOnce({ ok: true })
      .mockResolvedValueOnce({ ok: false, error: "No password field found on this page" });
    const sendResponse = vi.fn();

    listener({ type: "fill-login", id: "entry" }, { id: "extension-id" }, sendResponse);
    await vi.waitFor(() => expect(sendResponse).toHaveBeenCalledWith({ ok: true }));
  });

  it("aborts filling if the tab navigates to another origin", async () => {
    chromeMock.tabs.query.mockResolvedValue([{ id: 7, url: "https://example.com/login" }]);
    chromeMock.tabs.get.mockResolvedValue({ id: 7, url: "https://attacker.test/login" });
    chromeMock.runtime.sendNativeMessage.mockImplementation((_host, _message, callback) => {
      callback({ ok: true, username: "ada", password: "secret" });
    });
    const sendResponse = vi.fn();

    listener({ type: "fill-login", id: "entry" }, { id: "extension-id" }, sendResponse);
    await vi.waitFor(() => expect(sendResponse).toHaveBeenCalled());

    expect(sendResponse).toHaveBeenCalledWith({
      ok: false,
      error: "The page changed before Hitsu could fill it",
    });
    expect(chromeMock.scripting.executeScript).not.toHaveBeenCalled();
    expect(chromeMock.tabs.sendMessage).not.toHaveBeenCalled();
  });
});
