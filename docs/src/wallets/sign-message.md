# Signing Messages

`solana:signMessage` signs arbitrary byte payloads with the account's keypair. Apps use it for authentication challenges and off-chain attestations; Sign in With Solana builds on the same primitive.

## Implementing `WalletSolanaSignMessage`

The trait returns any type implementing `SolanaSignMessageOutput`, which itself extends `SolanaSignatureOutput`:

```rust,ignore
use solana_signature::Signature;
use solana_signer::Signer;
use wallet_standard::prelude::*;

impl WalletSolanaSignMessage for MyWallet {
	type Output = MySignMessageOutput;

	async fn sign_message(&self, props: SolanaSignMessageProps) -> WalletResult<Self::Output> {
		// props.chain() identifies the chain to sign for,
		// props.auth_state() carries session/auth context.
		let signature = self
			.keypair
			.try_sign_message(&props.data())
			.map_err(WalletError::from)?;

		Ok(MySignMessageOutput {
			signature,
			signed_message: props.data(),
		})
	}
}
```

`SolanaSignMessageProps` exposes:

- `data()` — the raw message bytes to sign.
- `chain()` — the chain the message relates to (e.g. `solana:mainnet`).
- `auth_state()` — optional authentication context carried by the app.

## Consuming from an app

App code works against the output trait, so it never depends on your concrete output type:

```rust,ignore
let wallet: impl WalletSolanaSignMessage = /* ... */;
let output = wallet
    .sign_message(SolanaSignMessageProps::builder()
        .data(b"hello world".to_vec())
        .build())
    .await?;

let signature = output.signature();
let signed_bytes = output.signed_message();
```

The `signature()` value is a `Signature` (from the Agave `solana-signature` crate) which verifies against the account's public key.

## Returning richer outputs

Your `MySignMessageOutput` can add anything your wallet needs — display metadata, the raw signed message, signing method tags — as long as it implements the two output traits. The repository's snapshot tests pin the exact JSON shape that crosses the JS boundary for the reference implementations, so a wallet returning the recommended shape interoperates with the full Wallet Standard ecosystem.
