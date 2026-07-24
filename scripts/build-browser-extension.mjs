import { copyFileSync, mkdirSync, rmSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const browser = process.argv[2];
const builds = {
  chrome: {
    configDirectory: "chrome-extension",
    outputDirectory: "hitsu-chrome-extension",
    files: ["background.js", "content.js", "popup.html", "popup.js", "popup.css"],
  },
  firefox: {
    configDirectory: "firefox-extension",
    outputDirectory: "hitsu-firefox-extension",
    files: [
      "background.js",
      "background-loader.js",
      "content.js",
      "popup.html",
      "popup.js",
      "popup.css",
    ],
  },
};

const build = builds[browser];
if (!build) {
  throw new Error("Usage: node scripts/build-browser-extension.mjs <chrome|firefox>");
}

const output = path.join(root, "package", build.outputDirectory);
rmSync(output, { recursive: true, force: true });
mkdirSync(output, { recursive: true });

copyFileSync(
  path.join(root, build.configDirectory, "manifest.json"),
  path.join(output, "manifest.json"),
);
for (const file of build.files) {
  copyFileSync(path.join(root, "browser-extension", file), path.join(output, file));
}

console.log(`Built ${browser} extension in ${path.relative(root, output)}`);
