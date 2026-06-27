# Format all source files
format: format-js format-rust

# Format JS/TS/Svelte files
format-js:
    biome format --fix .

# Format Rust code
format-rust:
    cd src-tauri && cargo fmt

# Check formatting
format-check:
    biome check .

# Lint
lint:
    biome lint .

# Run all checks
check: format-check lint

# Install + build frontend
build:
    pnpm install
    pnpm build

# Run dev server
dev:
    pnpm dev

# Run Tauri dev
tauri-dev:
    pnpm tauri dev
