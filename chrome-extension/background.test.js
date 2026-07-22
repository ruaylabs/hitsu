import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";

const chromeMock = {
  runtime: {
    id: "extension-id",
    onMessage: { addListener: vi.fn() },
  },
  tabs: {
    query: vi.fn(),
  },
};

let activeHttpTab;
let pageMatchesOrigin;
let loginEntries;

beforeAll(async () => {
  vi.stubGlobal("chrome", chromeMock);
  ({ activeHttpTab, pageMatchesOrigin, loginEntries } = await import("./background.js"));
});

beforeEach(() => {
  chromeMock.tabs.query.mockReset();
});

afterAll(() => {
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
