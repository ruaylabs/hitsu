import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const fixtures = new URL("browser-extension/protocol/fixtures/", root);
const versions = JSON.parse(await readFile(new URL("versions.json", fixtures), "utf8"));

// Verify every snapshot, even when capturing current source. Fixture updates must
// explicitly record the package version, provenance, and new checksum.
for (const fixture of Object.values(versions)) {
  const bytes = await readFile(new URL(fixture.source, fixtures));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), fixture.sha256);
}

const target = process.argv[2] ?? "current";
assert.ok(["current", "chrome", "firefox"].includes(target), "Unknown protocol source");
const source =
  target === "current"
    ? new URL("browser-extension/background.js", root)
    : new URL(versions[target].source, fixtures);
let listener;
let requests;
const tab = { id: 7, url: "https://accounts.example.com/login" };
globalThis.chrome = {
  runtime: {
    id: "protocol-test",
    lastError: null,
    onMessage: {
      addListener(registered) {
        listener = registered;
      },
    },
    sendNativeMessage(host, message, callback) {
      assert.equal(host, "com.ruaylabs.hitsu.browser");
      // Match the native host's token injection, without a real vault or secrets.
      requests.push({ ...message, token: "protocol-test-token" });
      callback({
        ok: true,
        entries: [{ id: "entry", title: "Example", username: "test" }],
        username: "test",
        password: "synthetic-password",
        otp: "123456",
        remaining: 20,
      });
    },
  },
  tabs: {
    query: async () => [tab],
    get: async () => tab,
    sendMessage: async () => ({ ok: true }),
  },
  scripting: {
    executeScript: async () => [{ frameId: 0, result: "https://accounts.example.com" }],
  },
};
const code = await readFile(source);
await import(`data:text/javascript;base64,${code.toString("base64")}`);
assert.equal(typeof listener, "function", "Background listener did not register");

const cases = [
  ["popup-list", { type: "list-logins" }, false, "listLogins"],
  ["inline-list", { type: "list-logins" }, true, "listLogins"],
  ["popup-fill", { type: "fill-login", id: "entry" }, false, "getCredentials"],
  [
    "inline-fill",
    { type: "fill-login-inline", id: "entry", requestId: "request" },
    true,
    "getCredentials",
  ],
  ["totp", { type: "get-totp", id: "entry" }, false, "getTotp"],
];
const captures = [];
for (const [name, message, inline, type] of cases) {
  requests = [];
  const sender = { id: "protocol-test" };
  if (inline) Object.assign(sender, { tab, frameId: 0, url: tab.url });
  const response = await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`${name}: response timed out`)), 2000);
    const accepted = listener(message, sender, (result) => {
      clearTimeout(timer);
      resolve(result);
    });
    assert.equal(accepted, true, `${name}: listener rejected message`);
  });
  assert.equal(response.ok, true, `${name}: ${JSON.stringify(response)}`);
  assert.equal(requests.length, 1, `${name}: expected exactly one native request`);
  assert.equal(requests[0].type, type);
  assert.equal(requests[0].origin, "https://accounts.example.com");
  // Missing flags are deliberately passed through to the real Rust deserializer.
  if ("inline" in requests[0]) assert.equal(requests[0].inline, inline);
  captures.push({ name, request: requests[0] });
}
console.log(JSON.stringify(captures));
