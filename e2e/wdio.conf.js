import { spawn } from "node:child_process";
import { rm } from "node:fs/promises";
import net from "node:net";
import path from "node:path";

const runId = `${process.pid}`;
const configHome = path.join("/tmp", `kagi-e2e-config-${runId}`);
const vaultPath = path.join("/tmp", `kagi-e2e-vault-${runId}.kdbx`);
const application = path.resolve(process.env.KAGI_E2E_APP ?? "src-tauri/target/debug/kagi");
let driver;

process.env.KAGI_E2E_VAULT = vaultPath;

function waitForDriver(timeoutMs = 15_000) {
  const startedAt = Date.now();
  return new Promise((resolve, reject) => {
    const connect = () => {
      const socket = net.createConnection({ host: "127.0.0.1", port: 4444 });
      socket.once("connect", () => {
        socket.destroy();
        resolve();
      });
      socket.once("error", () => {
        socket.destroy();
        if (Date.now() - startedAt >= timeoutMs) {
          reject(new Error("tauri-driver did not start within 15 seconds"));
        } else {
          setTimeout(connect, 100);
        }
      });
    };
    connect();
  });
}

async function cleanup() {
  await Promise.all([
    rm(configHome, { recursive: true, force: true }),
    rm(vaultPath, { force: true }),
  ]);
}

export const config = {
  runner: "local",
  specs: ["./specs/**/*.e2e.js"],
  maxInstances: 1,
  hostname: "127.0.0.1",
  port: 4444,
  path: "/",
  logLevel: "warn",
  // Vault writes run Argon2 and can exceed WebdriverIO's 5-second default on
  // shared CI runners. Keep element waits below Mocha's overall timeout while
  // allowing the native save to complete before asserting on the detail view.
  waitforTimeout: 20_000,
  waitforInterval: 100,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": { application },
    },
  ],
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    timeout: 60_000,
  },
  async onPrepare() {
    await cleanup();
    driver = spawn(process.env.TAURI_DRIVER ?? "tauri-driver", [], {
      env: { ...process.env, XDG_CONFIG_HOME: configHome },
      stdio: ["ignore", "inherit", "inherit"],
    });
    await waitForDriver();
  },
  async onComplete() {
    driver?.kill();
    await cleanup();
  },
};
