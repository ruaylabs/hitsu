# Format all source files
format: format-js format-rust

# Format JS/TS/Svelte files
format-js:
    pnpm exec biome format --fix .

# Format Rust code
format-rust:
    cd src-tauri && cargo fmt
    cd browser-extension/native-host && cargo fmt

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

# Run dev server
dev:
    pnpm dev

# Run Tauri dev
tauri-dev:
    pnpm tauri dev
