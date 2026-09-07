# CI and Releases

## CI

Every pull request runs (`.github/workflows/ci.yml`):

| Job        | What it does                          |
| ---------- | ------------------------------------- |
| `lint`     | clippy + dprint format check.         |
| `security` | cargo-audit, cargo-deny, zizmor.      |
| `test`     | native nextest suite.                 |
| `coverage` | llvm-cov report uploaded to Codecov.  |
| `build`    | locked build, plain and all-features. |

All actions are pinned by SHA, checkouts drop credentials, and the workflow runs with `contents: read` only.

## Release flow with MonoChange

Releases are planned by [MonoChange](https://github.com/pina-rs/monochange):

1. **Create a changeset** describing the intent (`release:change` devenv script, or a hand-written `.changeset/*.md`):

   ```markdown
   ---
   wallet_standard: minor
   ---

   Add sign in With Solana support
   ```

   The front matter lists packages and their bump type; the body becomes the changelog entry.

2. **Release PR** — when changesets exist on `main`, the `release-pr` workflow runs `monochange run release`, which:
   - computes the next version per package from the changesets,
   - updates `Cargo.toml` versions and `Cargo.lock`,
   - refreshes the changelog,
   - opens (or updates) the `monochange/release/*` pull request.

3. **Publish** — merging the release PR tags and dispatches the `publish` workflow, which publishes every package in dependency order using **trusted publishing** (crates.io OIDC, `environment: publisher`). No registry tokens are stored in the repository.

## Publishing locally

If CI publishing fails or you need to publish from a machine with credentials:

```bash
# prepare versions, changelog and lockfile, then commit locally
release:local

# publish from the local release commit
publish:local
```

`release:local` runs the same monochange release steps as CI (`PrepareRelease` → format → commit), and `publish:local` runs `PublishPackages` with the local cargo credentials.

## Versioning rules

| Changeset type            | Version effect       |
| ------------------------- | -------------------- |
| `feat`                    | minor                |
| `fix`                     | patch                |
| `docs` / `none`           | no release by itself |
| breaking (`!` or `major`) | major                |

`wallet_standard_browser` tracks `wallet_standard` through `parent_bump = "patch"`: any bump to the core crate also bumps the browser crate at least by patch, keeping the pair in lockstep.
