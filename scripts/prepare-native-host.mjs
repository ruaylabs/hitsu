import { execFileSync } from "node:child_process";
import { chmodSync, copyFileSync, mkdirSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifest = path.join(root, "chrome-extension", "native-host", "Cargo.toml");

function rustHostTriple() {
  const output = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const host = output.match(/^host: (.+)$/m)?.[1];
  if (!host) throw new Error("Could not determine the Rust host target triple");
  return host;
}

const target = process.env.KAGI_TARGET_TRIPLE ?? process.env.CARGO_BUILD_TARGET ?? rustHostTriple();
const executable = process.platform === "win32" ? "kagi-native-host.exe" : "kagi-native-host";

execFileSync("cargo", ["build", "--release", "--manifest-path", manifest, "--target", target], {
  cwd: root,
  stdio: "inherit",
});

const source = path.join(
  root,
  "chrome-extension",
  "native-host",
  "target",
  target,
  "release",
  executable,
);
const destinationDirectory = path.join(root, "src-tauri", "binaries");
const destination = path.join(
  destinationDirectory,
  process.platform === "win32" ? `kagi-native-host-${target}.exe` : `kagi-native-host-${target}`,
);

mkdirSync(destinationDirectory, { recursive: true });
copyFileSync(source, destination);
if (process.platform !== "win32") chmodSync(destination, 0o755);
console.log(`Prepared Tauri sidecar: ${path.relative(root, destination)}`);
