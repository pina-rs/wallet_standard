---
wallet_standard: patch
wallet_standard_browser: patch
---

# Verify sign-in resources and fix broken doctests

`verify_sign_in` now checks the `Resources:` block of a signed Sign In With Solana statement instead of skipping it, so a wallet that drops a requested resource can no longer pass verification. The signed message is read from the output once and shared by the text check and the signature check instead of crossing the wasm boundary twice, and `create_sign_in_message_text` propagates write failures like the rest of the function rather than `unwrap`ping.

Nine doctests that failed to compile are fixed: the readme's Sign In With Solana example imported `Keypair` from the wrong crate and referenced an undefined wallet, its error-handling example called an undefined function, and the seven value illustrations in `types.rs` are now marked `rust,ignore` like the other illustrative snippets. The "occured" typo in the deserialization error message is corrected, and the empty version markers in the root readme are filled in so `mdt check` passes again.

Wallet discovery reads the JavaScript `features` object once per compatibility check instead of once per probed feature, and the dependency lockfile plus the pinned cargo tools (cargo-dylint 6.2.0, cargo-nextest 0.9.148, cargo-semver-checks 0.51.0, wasm-bindgen-cli 0.2.129) are refreshed to their latest releases.
