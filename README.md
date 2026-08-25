# confiturejeu

A [Bevy](https://bevyengine.org) playground for my own entertainment.

> Current state: nothing.

## Requirements

- Rust stable (pinned via [`rust-toolchain.toml`](rust-toolchain.toml))
- [`just`](https://github.com/casey/just) as the task runner
- Linux only: `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`

Install the optional cargo tools used by the checks:

```bash
just install-tools
```

## Quick start

```bash
just run          # fast dev build (dynamic linking + asset hot-reload)
just run-release  # exactly what ships
just --list       # every available task
```

The `dev` cargo feature enables dynamic linking, Bevy's dev tools and the asset file watcher.
It should **never** be enabled in release or CI builds.

## Controls

## Development workflow

```bash
just lint        # format + clippy
just test        # nextest suite
just check-all   # everything CI runs, in CI order
```

> `src/main.rs` is currently Bevy's shape-primitives example, kept as a scratch pad. It is not
> rustfmt-formatted and trips the Clippy gate, so `just check-all` will not go green until it is
> replaced with real game code.

Install the git hooks once so mistakes are caught before they reach CI:

```bash
just install-hooks
```

- `pre-commit` runs `format-check` and `clippy`
- `pre-push` runs `test` and `check-wasm`

## Web build

```bash
just serve-web    # builds to ./dist and serves http://localhost:8000
```

`just build-web` alone produces an itch.io-ready `./dist` folder using the size-optimized
`wasm-release` profile.

## Layout

| Path                     | Purpose                                              |
| ------------------------ | ---------------------------------------------------- |
| `src/`                   | Game code                                            |
| `assets/`                | Runtime assets loaded by Bevy                        |
| `web/index.html`         | Shell page for the wasm build                        |
| `justfile`               | Every task; the single source of truth for commands  |
| `deny.toml`              | License / advisory / source policy                   |
| `.githooks/`             | Git hooks installed by `just install-hooks`          |
| `.github/workflows/`     | CI (`ci.yml`) and multi-platform releases            |

## Releasing

Push a `v*` tag. The release workflow builds Linux, Windows, macOS and web bundles and
uploads them as artifacts.

```bash
git tag v0.1.0 && git push origin v0.1.0
```

## License

Code is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

Assets under `assets/` are released under [CC0-1.0](LICENSE-ASSETS) unless a `LICENSE` file in
their subdirectory states otherwise.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in this work shall be dual-licensed as above, without any additional terms or conditions.
