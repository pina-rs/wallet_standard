---
wallet_standard: feat
wallet_standard_browser: feat
---

# Implement Wallet Standard features 1.5.0

Catch up to `@solana/wallet-standard-features@1.5.0`:

- `solana:signAndSendAllTransactions` (spec 1.3.0): sign and send a whole batch in one wallet prompt, with each transaction settling independently — a refused transaction arrives as a `WalletSettled::Rejected` element instead of failing the batch, so partial success is a first-class result.
- `solana:signOffchainMessage` (spec 1.4.0, SRFC-3): the wallet wraps the app's UTF-8 message in the canonical preamble (domain, version, sorted signers) and signs it, returning the exact preamble bytes so verifiers need no wallet SDK.

Both ship as core traits plus browser bridges with flat wire-format inputs (`Uint8Array` signer keys, numeric `messageVersion`), and are covered by the headless-Chrome boundary suite: the settled-batch parsing is driven end to end against a mock JS wallet, and the offchain input shape is pinned against the spec.
