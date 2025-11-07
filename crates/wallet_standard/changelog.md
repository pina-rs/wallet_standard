# Changelog

## 0.5.1 (2025-11-07)

### Fixes

- Use `solana_transaction_error` to prevent private usage of enum

## 0.5.0 (2025-11-07)

### Breaking Changes

#### Upgrade solana toolchain to 'v3'

Additionally, remove all instances of `solana-sdk` and use the core crates instead. For example, `solana_message` instead of `solana_sdk::message`.

### Fixes

- Remove unused dependency `derive_more`

### Documentation

- Make some improvements to the documentation for the crates.

## [Unreleased]

## [0.4.4](https://github.com/ifiokjr/wallet_standard/compare/0.4.3...0.4.4) - 2025-03-15

### <!-- 5 -->🎨 Styling

- update formatting

## [0.4.2](https://github.com/ifiokjr/wallet_standard/compare/wallet_standard@v0.4.1...wallet_standard@v0.4.2) - 2024-12-13

### <!-- 6 -->🧪 Testing

- add basic tests

### <!-- 7 -->⚙️ Miscellaneous Tasks

- update dependencies and docs
- move `wallet_standard` to new repository

## [0.4.1](https://github.com/ifiokjr/wallet_standard/compare/wallet_standard@v0.4.0...wallet_standard@v0.4.1) - 2024-12-12

### <!-- 7 -->⚙️ Miscellaneous Tasks

- update dependencies and configurations across multiple crates

## [0.4.0](https://github.com/ifiokjr/wallet_standard/compare/wallet_standard@v0.3.0...wallet_standard@v0.4.0) - 2024-10-13

### <!-- 0 -->🎉 Added

- [**breaking**] rename `pubkey` to `WalletSolanaPubkey::solana_pubkey` to prevent clashes
- [**breaking**] rename `sign_message` to `WalletSolanaSignMessage::sign_message_async`

### <!-- 1 -->🐛 Bug Fixes

- update instances of `pubkey` and `sign_message` after rename

## [0.3.0](https://github.com/ifiokjr/wallet_standard/compare/wallet_standard@v0.2.1...wallet_standard@v0.3.0) - 2024-10-12

### <!-- 2 -->🚜 Refactor

- [**breaking**] remove `AsyncSigner`

## [0.2.1](https://github.com/ifiokjr/wallet_standard/compare/wallet_standard@v0.2.0...wallet_standard@v0.2.1) - 2024-10-03

### <!-- 7 -->⚙️ Miscellaneous Tasks

- update formatting

## [0.2.0](https://github.com/ifiokjr/wallet_standard/compare/wallet_standard@v0.1.3...wallet_standard@v0.2.0) - 2024-09-28

### <!-- 0 -->🎉 Added

- [**breaking**] make `signed_transaction` return `VersionedTransaction`

## [0.1.3](https://github.com/ifiokjr/wallet_standard/compare/wallet_standard@v0.1.2...wallet_standard@v0.1.3) - 2024-09-18

### <!-- 3 -->📚 Documentation

- include crate `readme.md`

## [0.1.2](https://github.com/ifiokjr/wallet_standard/compare/wallet_standard@v0.1.1...wallet_standard@v0.1.2) - 2024-09-16

### <!-- 2 -->🚜 Refactor

- `strip_option` methods to `SolanaSignAndSendTransactionProps`

### <!-- 7 -->⚙️ Miscellaneous Tasks

- make crate versioning independent

## [0.1.1](https://github.com/ifiokjr/wallet_standard/compare/wallet_standard@v0.1.0...wallet_standard@v0.1.1) - 2024-09-13

### <!-- 2 -->🚜 Refactor

- remove unused imports

### <!-- 3 -->📚 Documentation

- add `wallet-standard` github repo link
- update crate readme description
