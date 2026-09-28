# wallet_standard browser examples

Two real-world WASM dApps — one per popular Rust frontend framework — that use `wallet_standard_browser` end to end against a local [surfpool](https://github.com/forge-xyz/surfpool) node, plus a Playwright suite that proves the whole flow in a real browser.

```
examples/
├── crates/surfpool-wallet-core   # framework-agnostic core shared by both apps
├── leptos-wallet/                # Leptos 0.8 CSR app (built with trunk)
├── dioxus-wallet/                # Dioxus 0.7 web app (built with trunk)
└── e2e/                          # Playwright 1.56 tests + surfpool orchestration
```

## What the examples demonstrate

Each app embeds a **complete Wallet Standard wallet implemented in Rust** (the "Surfpool Dev Wallet") and registers it into the page from WASM — the same thing a Rust-based wallet extension would do. The app's dApp side then talks to that wallet through `wallet_standard_browser` exactly as it would talk to Phantom or any injected wallet:

| Step                      | Feature exercised                                                          |
| ------------------------- | -------------------------------------------------------------------------- |
| Detect wallet             | `get_wallets()` + `Wallets::get()`                                         |
| Connect / disconnect      | `standard:connect`, `standard:disconnect` (trait impls on `BrowserWallet`) |
| Account change events     | `standard:events`                                                          |
| Airdrop + balance         | surfpool JSON-RPC (`requestAirdrop`, `getBalance`)                         |
| Sign message + verify     | `solana:signMessage`, local ed25519 verification                           |
| Sign, dApp broadcasts     | `solana:signTransaction` + `sendTransaction` + `getSignatureStatuses`      |
| Sign & send in the wallet | `solana:signAndSendTransaction` (the wallet broadcasts)                    |

The dev wallet's key lives inside the WASM heap; only signatures and public data cross the JS boundary. The keypair is a throwaway, deterministic dev key (`4acVT7jfykgHZNunZYEwg1NNCUnvuXwFH292ebEGnN4g`) so tests are reproducible.

## Prerequisites

- Rust with the `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- [`trunk`](https://trunkrs.dev) (`cargo install trunk` or `cargo binstall trunk`)
- Node.js ≥ 20 and npm
- [`surfpool`](https://github.com/forge-xyz/surfpool) on your `PATH`
- `wasm-bindgen-cli` matching the workspace `wasm-bindgen` version (`cargo binstall wasm-bindgen-cli --version 0.2.129`)

## Run the Playwright suite

```bash
cd examples/e2e
npm install
npx playwright test
```

The Playwright config starts everything it needs:

1. `surfpool start --offline` (RPC on `:8899`, WS on `:8900`) behind a small health-check wrapper,
2. `trunk serve` for the Leptos app on `http://127.0.0.1:3021`,
3. `trunk serve` for the Dioxus app on `http://127.0.0.1:3022`.

Each spec then drives the full connect → airdrop → sign message (verified) → sign + send (dApp broadcasts) → signAndSend (wallet broadcasts) → disconnect flow and asserts on `data-testid` hooks. Run it headed with `npm run test:headed`, or inspect a failure with `npm run report`.

> **Note** — build the two apps with **separate target directories**. Sharing one `CARGO_TARGET_DIR` between two concurrent `trunk serve` processes corrupts their wasm-bindgen staging output.

## Manual exploration

```bash
# terminal 1: local chain
surfpool start --offline

# terminal 2: an app
cd examples/leptos-wallet && trunk serve --port 3021
# or: cd examples/dioxus-wallet && trunk serve --port 3022
```

Then open the served URL. The RPC endpoint defaults to `http://127.0.0.1:8899` and can be overridden with `?rpc=<host:port>`.

## Where to look in the code

- `crates/surfpool-wallet-core/src/wallet.rs` — building a Wallet Standard wallet object in Rust: JS feature objects backed by Rust closures, keeping closures alive for the page lifetime, signing transaction wire bytes with `solana-keypair`, and registering with `register_wallet`.
- `crates/surfpool-wallet-core/src/rpc.rs` — a minimal `fetch`-based Solana JSON-RPC client that works from WASM.
- `leptos-wallet/src/main.rs` / `dioxus-wallet/src/main.rs` — the dApp side: discovery, connect, signing flows, broadcasting, and confirmation polling, using the same `wallet_standard_browser` API in two different UI stacks.
