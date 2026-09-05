# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Project tooling: `justfile` task runner, pinned Rust toolchain, `rustfmt` and Clippy
  configuration, `cargo-deny` policy, git hooks, CI and multi-platform release workflows.
- Web (wasm) build pipeline with a size-optimized `wasm-release` profile.
- `dev` cargo feature bundling dynamic linking, Bevy dev tools and asset hot-reload.
- Basic physic system using `Avian3D`.
- Menus
- Level visualization: pure `level_to_svg` dumps (`generate --svg`) and a gizmo-based
  top-down `level_viewer` binary (`just viewer`) with pan/zoom, floor switching and reseed.

### Fixed


### Changed


### Fixed


[Unreleased]: https://github.com/sebastianMindee/confiturejeu/compare/HEAD
