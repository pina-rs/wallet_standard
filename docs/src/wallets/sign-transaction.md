# Signing Transactions

Solana wallets distinguish two flows:

1. **`solana:signTransaction`** — sign a transaction; the app submits it.
2. **`solana:signAndSendTransaction`** — sign and broadcast; the app only receives the signature.

## Signing without sending

`WalletSolanaSignTransaction::sign_transaction` receives `SolanaSignTransactionProps`, which carries either a typed `VersionedTransaction` (feature `solana`) or raw `Vec<u8>` bytes, plus per-transaction `options`:

- `min_context_slot` — the minimum slot the wallet should sign against.
- `signers` — additional partial signers the app supplies.

```rust,ignore
impl WalletSolanaSignTransaction for MyWallet {
	type Output = MySignTransactionOutput;

	async fn sign_transaction(
		&self,
		props: SolanaSignTransactionPropsWithBytes,
	) -> WalletResult<Self::Output> {
		let mut transaction = props.transaction().clone();
		self.keypair
			.try_sign_versioned_transaction(&mut transaction)
			.map_err(WalletError::from)?;
		Ok(MySignTransactionOutput { transaction })
	}
}
```

The output must implement `SolanaSignTransactionOutput`, which exposes the signed transaction bytes. The crate provides a canonical byte conversion via the `signed_transaction_bytes()` helper that snapshot tests pin down.

## Signing and sending

`sign_and_send_transaction` receives the same props plus a `SolanaSignAndSendTransactionMode` (`Encoded` or `Versioned`) and must broadcast the signed transaction to the chain indicated by `props.chain()`:

```rust,ignore
impl WalletSolanaSignAndSendTransaction for MyWallet {
	type Output = MySignAndSendOutput;

	async fn sign_and_send_transaction(
		&self,
		props: SolanaSignAndSendTransactionProps,
	) -> WalletResult<Self::Output> {
		let signature = self.submit_to_rpc(&props).await?;
		Ok(MySignAndSendOutput { signature })
	}
}
```

The output extends `SolanaSignatureOutput`, so the app receives exactly the transaction signature it can use for confirmation lookups.

## Serialization notes

Transactions cross the JS boundary as bytes encoded with `bincode` 1.x — the same wire format the Solana RPC and browser wallets use. With the latest Agave crates the `bincode` cargo feature no longer exists on `solana-message`/`solana-transaction`; the external `bincode` crate handles serialization directly, which is what this repository does (see `sign_transaction.rs` in both crates).
