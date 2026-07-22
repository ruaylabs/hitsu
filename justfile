# Format all source files
format: format-js format-rust

# Format JS/TS/Svelte files
format-js:
    pnpm exec biome format --fix .

# Format Rust code
format-rust:
    cd src-tauri && cargo fmt
    cd chrome-extension/native-host && cargo fmt

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
extension-zip:
    mkdir -p package
    rm -f package/hitsu-extension.zip
    cd chrome-extension && zip -q ../package/hitsu-extension.zip manifest.json background.js content.js popup.html popup.js popup.css
    unzip -l package/hitsu-extension.zip

# Run dev server
dev:
    pnpm dev

# Run Tauri dev
tauri-dev:
    pnpm tauri dev
