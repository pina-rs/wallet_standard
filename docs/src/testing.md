# Testing

## Native tests

The core crate's snapshot tests pin the wire shapes (signatures, signed transaction bytes) that must not drift:

```bash
cargo test_wallet_standard
```

The alias runs `cargo nextest` with the `solana` feature. Snapshots live next to the tests and are reviewed with `cargo insta`.

## Browser tests

`wallet_standard_browser` runs its test suite as `wasm-bindgen-test` in headless Chrome, driven by chromedriver:

```bash
test:browser
```

The browser tests cover the round trip a dApp depends on: wallets registered into the page are discovered through the bundled `@wallet-standard/app` registry, `standard:connect` resolves and attaches the account, and — most importantly — every input struct is asserted against the exact JavaScript shape that crosses the wasm boundary (`Uint8Array` bytes, flat wire-format keys). A regression in that serialization fails here instead of silently in downstream dApps.

## End-to-end examples

The Leptos and Dioxus example apps under `examples/` are compiled WASM bundles that embed a Rust Wallet Standard wallet and drive every Solana feature against a local [surfpool](https://github.com/solana-foundation/surfpool) node. The Playwright suite boots surfpool plus both dev servers and clicks through connect, airdrop, `signMessage` (verified locally), `signTransaction` + `sendTransaction`, and `signAndSendTransaction`:

```bash
cd examples/e2e
npm install
npx playwright test
```

This is the only layer that exercises the bundled output in a real browser, so bundling and glue defects surface here rather than in users' apps.

## Environment details

- `WASM_BINDGEN_TEST_WEBDRIVER_JSON` points at `setup/webdriver.json` (headless Chrome capabilities).
- `wasm-bindgen-test-runner` comes from `cargo bin` (pinned to the exact `wasm-bindgen` version in `[workspace.metadata.bin]`); chromedriver must be on `PATH` and match the installed Chrome major version — CI installs the matching build for the runner's Chrome.
- `validator:run` / `validator:bg` / `validator:kill` manage a local Solana test validator on `127.0.0.1:8899` for manual experiments.

## Coverage

```bash
coverage:all
```

produces `codecov.json` via `cargo llvm-cov` and the CI coverage job uploads it to Codecov.
