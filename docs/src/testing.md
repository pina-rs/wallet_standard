# Testing

## Native tests

The core crate's snapshot tests pin the wire shapes (signatures, signed transaction bytes) that must not drift:

```bash
cargo test_wallet_standard
```

The alias runs `cargo nextest` with the `solana` feature. Snapshots live next to the tests and are reviewed with `cargo insta`.

## Browser tests

`wallet_standard_browser` runs its test suite as `wasm-bindgen-test` against real browsers, driven by webdriver. The `test:validator` devenv script starts a local Solana test validator (agave from nixpkgs), waits for the RPC port, then runs the wasm tests through chromedriver:

```bash
# start validator + run chrome tests
test:validator
```

The browser tests exercise the full round trip: a Rust wallet registered into the page, discovered by the JS reference implementation, and used to sign transactions against the local validator.

## Environment details

- `WASM_BINDGEN_TEST_WEBDRIVER_JSON` points at `setup/webdriver.json`.
- `wasm-bindgen-test-runner` comes from `cargo bin` (pinned to the exact `wasm-bindgen` version in `[workspace.metadata.bin]`).
- The validator binds `127.0.0.1:8899`; `validator:kill` releases the port.

## Coverage

```bash
coverage:all
```

produces `codecov.json` via `cargo llvm-cov` and the CI coverage job uploads it to Codecov.
