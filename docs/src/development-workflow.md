# Development Workflow

This repository uses [devenv](https://devenv.sh) for a hermetic development environment. `direnv` loads it automatically.

## Environment

```bash
devenv shell   # or: direnv allow
```

The environment provides the pinned Rust toolchain (from `rust-toolchain.toml` via rustup), agave (for the test validator), mdbook, and all lint/security tooling (cargo-deny, cargo-audit, zizmor, gitleaks) sourced through `ifiokjr/nixpkgs` where upstream packaging lags.

## Daily commands

| Command          | Description                                                |
| ---------------- | ---------------------------------------------------------- |
| `build:all`      | Build all crates with all features.                        |
| `test:all`       | Run the native test suite.                                 |
| `test:validator` | Start a local validator and run the browser tests.         |
| `lint:all`       | clippy + dprint format check.                              |
| `fix:all`        | Apply clippy and dprint fixes.                             |
| `update:deps`    | `cargo update` + `devenv update` + refresh the bundled JS. |
| `security:all`   | cargo-deny + cargo-audit + zizmor.                         |
| `build:docs`     | Build this book (`mdbook build docs`).                     |

## Formatting

`dprint` formats everything (Rust via rustfmt nightly, TOML, Markdown, Nix, shell). The pre-commit hook formats changed files, and `lint:format` enforces it in CI.

## Git hooks

The devenv git-hooks integration installs:

- `gitleaks protect --staged` — secret scanning,
- `dprint fmt` — formatting,
- `nixfmt` — Nix file formatting.

## Conventions

- Branch names follow Conventional Commit prefixes: `feat/`, `fix/`, `build/`, `ci/`, `docs/`, `chore/`, `test/`.
- Work happens on feature branches with pull requests; `main` is protected by CI.
- Commits follow Conventional Commits; release automation derives versions from them (see [CI and Releases](./ci-and-releases.md)).
