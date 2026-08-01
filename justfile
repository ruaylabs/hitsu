# Format all source files
format: format-js format-rust format-html

# Format JS/TS/Svelte files
format-js:
    pnpm exec biome format --fix .

# Format Rust code
format-rust:
    cd src-tauri && cargo fmt
    cd browser-extension/native-host && cargo fmt

# Format HTML views
format-html:
    cd browser-extension-test-site && uvx djlint==1.43.1 --profile=golang --indent 2 --reformat templates/

# Check formatting
format-check:
    pnpm exec biome check .

# Lint
lint:
    pnpm exec biome lint .

# Run all checks
check: format-check lint

# Install + build frontend
build:
    pnpm install
    pnpm build

# Package the Chrome extension for store upload
chrome-extension-zip:
    rm -f package/hitsu-chrome-extension.zip
    node scripts/build-browser-extension.mjs chrome
    cd package/hitsu-chrome-extension && zip -q ../hitsu-chrome-extension.zip *
    unzip -l package/hitsu-chrome-extension.zip

# Prepare a clean Chrome extension build for local testing
chrome-extension-dev:
    rm -rf package/hitsu-extension package/hitsu-extension.zip package/hitsu-chrome-extension package/hitsu-chrome-extension.zip
    node scripts/build-browser-extension.mjs chrome
    @echo ""
    @echo "Test in Chrome:"
    @echo "  1. Open chrome://extensions and enable Developer mode"
    @echo "  2. Remove any existing unpacked Hitsu extension"
    @echo "  3. Click 'Load unpacked' and select:"
    @printf "     %s/package/hitsu-chrome-extension\n" "$PWD"
    @echo "  4. Copy the extension ID shown by Chrome and register the native host:"
    @echo "     ./chrome-extension/install-host.sh <EXTENSION_ID>"
    @echo "  5. Start Hitsu, enable Settings > Features > Browser integration, and unlock a vault"
    @echo "  6. Open an HTTP(S) login page and click the Hitsu toolbar icon"

# Package the Firefox extension for signing or distribution
firefox-extension-zip:
    rm -f package/hitsu-firefox-extension.zip
    node scripts/build-browser-extension.mjs firefox
    cd package/hitsu-firefox-extension && zip -q ../hitsu-firefox-extension.zip *
    unzip -l package/hitsu-firefox-extension.zip

# Build both store archives and run the same Firefox validation used by CI
browser-extension-validate: chrome-extension-zip firefox-extension-zip
    pnpm exec web-ext lint --warnings-as-errors --source-dir package/hitsu-firefox-extension

# Prepare a clean Firefox extension build for local testing
firefox-extension-dev:
    rm -rf package/hitsu-firefox-extension package/hitsu-firefox-extension.zip
    node scripts/build-browser-extension.mjs firefox
    ./firefox-extension/install-host.sh
    @echo ""
    @echo "Test in Firefox:"
    @echo "  1. Open about:debugging#/runtime/this-firefox"
    @echo "  2. Remove any existing temporary Hitsu extension"
    @echo "  3. Click 'Load Temporary Add-on...' and select:"
    @printf "     %s/package/hitsu-firefox-extension/manifest.json\n" "$PWD"
    @echo "  4. Start Hitsu, enable Settings > Features > Browser integration, and unlock a vault"
    @echo "  5. Open an HTTP(S) login page and click the Hitsu toolbar icon"

# Run the browser-extension scenario site
browser-extension-test-site:
    go run browser-extension-test-site/main.go

# Run dev server
dev:
    pnpm dev

# Run Tauri dev
tauri-dev:
    pnpm tauri dev

# Read current desktop app version from tauri.conf.json
version:
    @grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*"\([0-9.]*\)".*/\1/'

# Bump both browser-extension manifests, build publish archives, and commit the change
# Usage: just bump-browser-extension-version 0.2.0
bump-browser-extension-version version:
    @printf '%s\n' "{{ version }}" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || (echo "Version must use x.y.z format" >&2; exit 2)
    sed -i'' -e 's/"version": "[0-9.]*"/"version": "{{ version }}"/' chrome-extension/manifest.json firefox-extension/manifest.json
    @node -e 'for (const file of process.argv.slice(2)) { const manifest = require("./" + file); if (manifest.version !== process.argv[1]) throw new Error(file + " was not updated"); }' "{{ version }}" chrome-extension/manifest.json firefox-extension/manifest.json
    git add chrome-extension/manifest.json firefox-extension/manifest.json
    if ! git diff --cached --quiet; then git commit -m "chore(browser-extension): bump v{{ version }}"; fi
    just browser-extension-validate
    @echo ""
    @echo "Upload package/hitsu-chrome-extension.zip to the Chrome Web Store."
    @echo "Upload package/hitsu-firefox-extension.zip to Mozilla Add-ons."

# Bump desktop app version in all config files and create a tag
# Usage: just bump-version 0.2.0
bump-version version:
    sed -i'' -e 's/"version": "[0-9.]*"/"version": "{{ version }}"/' src-tauri/tauri.conf.json
    sed -i'' -e 's/"version": "[0-9.]*"/"version": "{{ version }}"/' package.json
    sed -i'' -e 's/^version = "[0-9.]*"/version = "{{ version }}"/' src-tauri/Cargo.toml
    cd src-tauri && cargo update --workspace
    git add src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock package.json pnpm-lock.yaml
    if ! git diff --cached --quiet; then git commit -m "chore: bump v{{ version }}"; fi
    git tag -a "v{{ version }}" -m "v{{ version }}"
