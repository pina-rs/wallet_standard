# Security

This repository treats supply-chain security as a first-class gate. Every change to dependencies or CI passes through automated checks.

## Dependency policy (`cargo-deny`)

`deny.toml` enforces:

- **Licenses** — an explicit allow-list (`MIT`, `Apache-2.0`, `BSD-*`, `ISC`, `MPL-2.0`, `Unicode-3.0`, `Zlib`, `Unlicense`, ...). Anything outside the list fails the build.
- **Bans** — duplicate crate versions are surfaced as warnings so accidental version splits (two `getrandom`s, three `rand_core`s) stay visible.
- **Sources** — crates may only come from crates.io; git sources must be explicitly allow-listed.

Run it with `devenv shell -c security:deny`.

## Advisory scanning (`cargo-audit`)

`security:audit` checks `Cargo.lock` against the RustSec advisory database. Ignored advisories must carry a comment explaining why they are safe to ignore.

## Workflow hardening (`zizmor`)

`security:zizmor` audits every GitHub Actions workflow and composite action. The repository policy (`.github/zizmor.yml`) is zero findings:

- every action pinned by commit SHA,
- `persist-credentials: false` on every checkout,
- workflow-level permissions limited to `contents: read`,
- Dependabot updates behind a 7-day cooldown.

## Secret scanning (`gitleaks`)

The pre-commit git hook scans staged changes for leaked secrets (`gitleaks protect --staged`). CI never holds registry tokens; publishing uses trusted publishing (OIDC), see [CI and Releases](./ci-and-releases.md).

## Cryptographic behavior

- Message and transaction signing delegate to the Agave `solana-signer` / `solana-keypair` crates (Ed25519 via `ed25519-dalek`).
- The `experimental:encrypt`/`experimental:decrypt` features implement the Wallet Standard's x25519-xsalsa20-poly1305 authenticated encryption scheme.
- No key material ever crosses the JS boundary through this crate: the browser bridge only passes signatures and public data.
