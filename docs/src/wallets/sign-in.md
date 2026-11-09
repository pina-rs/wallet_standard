# Sign in With Solana

`solana:signIn` implements [Sign in With Solana](https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/6-siws.md) (SIWS). It combines message signing with account selection: the wallet renders the human-readable EIP-4361-style statement, the user approves it, and the app receives both the signed message and the account that signed it.

## Implementing `WalletSolanaSignIn`

```rust,ignore
impl WalletSolanaSignIn for MyWallet {
	type Output = MySignInOutput;

	async fn sign_in(&self, input: SolanaSignInInput) -> WalletResult<Self::Output> {
		// 1. Render the SIWS message from the input fields.
		// 2. Ask the user to approve.
		// 3. Sign the message with the selected account's keypair.
		// 4. Return the account, signature and signed message.
	}
}
```

## The input

`SolanaSignInInput` mirrors the EIP-4361 fields, all optional:

| Field                                                                   | Purpose                                                             |
| ----------------------------------------------------------------------- | ------------------------------------------------------------------- |
| `domain`                                                                | The domain requesting sign-in. If omitted the wallet determines it. |
| `address`                                                               | The account address to sign in with. If omitted the user picks.     |
| `statement`                                                             | Human-readable statement the user approves.                         |
| `uri`                                                                   | URI the app will redirect to.                                       |
| `version`, `chain_id`, `nonce`                                          | EIP-4361 metadata for replay protection.                            |
| `issued_at`, `expiration_time`, `not_before`, `request_id`, `resources` | Standard SIWS extensions.                                           |

The input serializes to `camelCase` JSON, matching the TypeScript reference implementation.

## The output

`SolanaSignInOutput` extends both `SolanaSignatureOutput` and `SolanaSignMessageOutput`, and adds:

```rust,ignore
pub trait SolanaSignInOutput: SolanaSignatureOutput + SolanaSignMessageOutput {
	type Account: WalletAccountInfo;

	/// Account that was signed in.
	/// The address of the account may be different from the provided input
	/// Address.
	fn account(&self) -> Self::Account;
}
```

The returned account may differ from the requested `address` — the user can always choose a different account, so apps must use `output.account()` rather than echoing their input.

## Verifying on the backend

Call `verify_sign_in`. It parses the signed message with the same rules as `@solana/wallet-standard-util`:

- the bytes must be a SIWS message, not a free-form string that happens to contain the field names
- each field the app set must match the parsed field, and a field the app left unset must not appear
- the account address and public key must be the address inside the message
- the Ed25519 signature must verify over those exact bytes

Searching the signed text for `URI:` or `Resources:` is not enough. A wallet can embed those strings inside a different statement, or after a different domain line, and still satisfy a substring check. `verify_sign_in` rejects that message. Domain and address may be omitted from the input; the wallet fills them in, and verification then checks the account against the message.
