# Agent notes

Project context for AI coding agents working in this repository.

## What this is

A solo Bevy (Rust) game jam playground. Prototypes live here; the tooling is set up so a
promising prototype can be polished into a portfolio piece without restructuring.

## Ground rules

- **`src/main.rs` is currently a scratch pad** (Bevy's shape-primitives example). It is not
  formatted or lint-clean and is expected to be replaced wholesale. Do not refactor it; do not
  treat its style as a project convention.
- **`just` is the entry point.** Never invent ad-hoc cargo invocations when a recipe exists.
  Run `just --list` first. Add new commands as recipes rather than documenting raw commands.
- **Run `just check-all` before declaring work done.** It mirrors CI exactly.
- Clippy runs with `pedantic` + `nursery` and `-D warnings`. Fix lints rather than
  sprinkling `#[allow]`; if an allow is genuinely right, add it to `[lints.clippy]` in
  `Cargo.toml` with a comment so the exception is project-wide and reviewable.
- `unsafe_code` is denied and `missing_docs` warns. Public items need doc comments.
- The `dev` cargo feature (dynamic linking, hot reload) is for local iteration only. It must
  never be enabled in release builds, CI jobs, or committed defaults.
- Keep the wasm target compiling. Anything native-only goes behind
  `#[cfg(not(target_arch = "wasm32"))]`, and `just check-wasm` must pass.
- Update `CHANGELOG.md` under `## [Unreleased]` for player-visible changes.

## Layout

- `src/` — game code
- `assets/` — runtime assets loaded by Bevy
- `web/index.html` — shell page for the wasm bundle
- `.github/workflows/ci.yml` — format, clippy, cross-platform tests, wasm check, cargo-deny
- `.github/workflows/release.yml` — tag-triggered native + web bundles

## Adding dependencies

New crates must pass `cargo deny check`. If a license is legitimately needed but missing,
add it to the `allow` list in `deny.toml` with a comment explaining why.
