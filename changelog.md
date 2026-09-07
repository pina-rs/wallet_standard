# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0](https://github.com/pina-rs/wallet_standard/releases/tag/v0.6.0) (2026-09-07)

Grouped release for `core`.

### Breaking Changes

#### Update Solana dependencies to Agave 4.x

_Packages:_ _wallet_standard_, _wallet_standard_browser_

Update the Solana dependencies to the latest Agave 4.x crates, raise the MSRV to 1.89.0, and unify both crates on a single workspace version.

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #44](https://github.com/pina-rs/wallet_standard/pull/44)

### Notes

#### Migrate release automation to monochange

_Packages:_ _wallet_standard_, _wallet_standard_browser_

Replace knope with monochange: a release PR flow on every push to main, crates.io trusted publishing through the `publisher` environment, and `release:local` / `publish:local` devenv scripts as a manual fallback.

_Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #44](https://github.com/pina-rs/wallet_standard/pull/44)
