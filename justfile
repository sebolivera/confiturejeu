set shell := ["bash", "-uc"]

# List the available tasks
[private]
default:
    @just --list

# ---------------------------------------------------------------- run / build

# Run the game with fast-iteration dev features (dynamic linking, hot asset reload)
run:
    cargo run --features dev

# Run the game exactly as it ships (no dev features, release profile)
run-release:
    cargo run --release

# Build a debug binary
build:
    cargo build

# Build the shippable release binary
build-release:
    cargo build --release

# Build the web/wasm bundle into ./dist (needs: cargo binstall wasm-bindgen-cli)
build-web:
    cargo build --profile wasm-release --target wasm32-unknown-unknown
    wasm-bindgen --no-typescript --target web \
        --out-dir ./dist --out-name confiturejeu \
        ./target/wasm32-unknown-unknown/wasm-release/confiturejeu.wasm
    @[ -d assets ] && cp -r assets ./dist/ || true
    @cp web/index.html ./dist/index.html
    @echo "Web build written to ./dist (serve it over http, not file://)"

# Serve the web build locally on http://localhost:8000
serve-web: build-web
    @cd dist && python3 -m http.server 8000

# ------------------------------------------------------------------- quality

# Format all Rust code
format:
    cargo fmt --all

# Verify formatting without rewriting files
format-check:
    cargo fmt --all --check

# Lint with Clippy, denying warnings
clippy:
    cargo clippy --all-targets -- -D warnings

# Type-check the wasm target so web builds don't rot
check-wasm:
    cargo check --target wasm32-unknown-unknown

# Audit licenses, advisories, bans and sources
check-deps:
    cargo deny check

# Find unused dependencies (nightly; installs the toolchain if missing)
check-unused:
    (rustup toolchain install nightly --no-self-update 2>/dev/null || true) && cargo +nightly udeps

# Run the test suite
test:
    cargo nextest run --no-tests=pass

# Run doc tests (nextest does not cover these)
test-doc:
    cargo test --doc

# Build and open the API documentation
doc:
    cargo doc --no-deps --open

# ------------------------------------------------------------------ bundles

# Everything CI runs, in the same order
check-all: format-check clippy check-wasm test test-doc check-deps
    @echo -e "\033[0;32mAll checks passed.\033[0m"

# Fast local loop: fix what can be fixed, then lint
lint: format clippy

# Install the extra cargo tools this justfile expects
install-tools:
    cargo install cargo-binstall --locked || true
    cargo binstall -y cargo-nextest cargo-deny wasm-bindgen-cli

# Install the git hooks (pre-commit / pre-push)
install-hooks:
    @mkdir -p .git/hooks
    @cp .githooks/pre-commit .git/hooks/pre-commit
    @cp .githooks/pre-push .git/hooks/pre-push
    @chmod +x .git/hooks/pre-commit .git/hooks/pre-push
    @echo "Git hooks installed."

# Remove build artifacts
clean:
    cargo clean && rm -rf ./dist

# Update dependencies within semver bounds
update:
    cargo update
