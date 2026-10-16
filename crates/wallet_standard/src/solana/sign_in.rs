use std::fmt::Write;

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use solana_pubkey::Pubkey;
use typed_builder::TypedBuilder;

use super::SolanaSignMessageOutput;
use crate::SolanaSignatureOutput;
use crate::WalletAccountInfo;
use crate::WalletError;
use crate::WalletResult;

/// Feature identifier for Sign In With Solana.
///
/// Based on EIP-4361 / CAIP-122: the app states who it is and what it
/// wants, and the wallet returns a signed statement, giving login
/// semantics without exposing an account before the user consents.
pub const SOLANA_SIGN_IN: &str = "solana:signIn";

/// The result of a Sign In With Solana attempt: the account that signed,
/// the statement that was signed, and its signature.
pub trait SolanaSignInOutput: SolanaSignatureOutput + SolanaSignMessageOutput {
	/// The account type the wallet hands back, which may differ from the
	/// account the app addressed.
	type Account: WalletAccountInfo;
	/// Account that was signed in.
	/// The address of the account may be different from the provided input
	/// Address.
	fn account(&self) -> Self::Account;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// The app's side of a Sign In With Solana statement.
///
/// Every optional field is one the wallet is trusted to infer, so minimal
/// inputs still produce a well-formed signed statement.
pub struct SolanaSignInInput {
	/// Optional EIP-4361 Domain.
	/// If not provided, the wallet must determine the Domain to include in the
	/// message.
	#[builder(default, setter(into, strip_option))]
	pub domain: Option<String>,
	/// Optional EIP-4361 Address.
	/// If not provided, the wallet must determine the Address to include in the
	/// message.
	#[builder(default, setter(into, strip_option))]
	pub address: Option<String>,
	/// Optional EIP-4361 Statement.
	/// If not provided, the wallet must not include Statement in the message.
	#[builder(default, setter(into, strip_option))]
	pub statement: Option<String>,
	/// Optional EIP-4361 URI.
	/// If not provided, the wallet must not include URI in the message.
	#[builder(default, setter(into, strip_option))]
	pub uri: Option<String>,
	/// Optional EIP-4361 Version.
	/// If not provided, the wallet must not include Version in the message.
	#[builder(default, setter(into, strip_option))]
	pub version: Option<String>,
	/// Optional EIP-4361 Chain ID.
	/// If not provided, the wallet must not include Chain ID in the message.
	#[builder(default, setter(into, strip_option))]
	pub chain_id: Option<String>,
	/// Optional EIP-4361 Nonce.
	/// If not provided, the wallet must not include Nonce in the message.
	#[builder(default, setter(into, strip_option))]
	pub nonce: Option<String>,
	/// Optional EIP-4361 Issued At.
	/// If not provided, the wallet must not include Issued At in the message.
	#[builder(default, setter(into, strip_option))]
	pub issued_at: Option<String>,
	/// Optional EIP-4361 Expiration Time.
	/// If not provided, the wallet must not include Expiration Time in the
	/// message.
	#[builder(default, setter(into, strip_option))]
	pub expiration_time: Option<String>,
	/// Optional EIP-4361 Not Before.
	/// If not provided, the wallet must not include Not Before in the message.
	#[builder(default, setter(into, strip_option))]
	pub not_before: Option<String>,
	/// Optional EIP-4361 Request ID.
	/// If not provided, the wallet must not include Request ID in the message.
	#[builder(default, setter(into, strip_option))]
	pub request_id: Option<String>,
	/// Optional EIP-4361 Resources.
	/// If not provided, the wallet must not include Resources in the message.
	#[builder(default, setter(into, strip_option))]
	pub resources: Option<Vec<String>>,
}

#[async_trait(?Send)]
/// Performs Sign In With Solana through the wallet.
///
/// Apps use this instead of connect-then-signMessage so they never see an
/// address before the user has approved the login statement itself.
pub trait WalletSolanaSignIn {
	/// The wallet-specific output carrying the signed statement.
	type Output: SolanaSignInOutput;

	/// Request one signed login statement.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::WalletSignIn`] when the wallet rejects
	/// the attempt.
	async fn sign_in(&self, input: SolanaSignInInput) -> WalletResult<Self::Output>;
	/// Request several signed statements in one round trip.
	///
	/// # Errors
	///
	/// Fails with [`WalletError::InvalidArguments`] for an empty batch.
	async fn sign_in_many(&self, inputs: Vec<SolanaSignInInput>)
	-> WalletResult<Vec<Self::Output>>;
}

/// Check that the input and output of the sign in are valid.
pub fn verify_sign_in(
	input: &SolanaSignInInput,
	output: &impl SolanaSignInOutput,
) -> WalletResult<()> {
	let account = output.account();

	let Some(input_address) = input.address.as_ref() else {
		return Err(WalletError::WalletSignIn);
	};

	if account.address().as_str() != input_address {
		return Err(WalletError::WalletSignIn);
	}

	// `signed_message` crosses the wasm boundary once and feeds both the
	// text check and the signature check.
	let signed_message = output.signed_message();

	verify_output_text(input, &signed_message)?;

	let signature = output.try_signature()?;
	let bytes = account.public_key();
	let pubkey = Pubkey::try_from(bytes).map_err(|_| WalletError::WalletPublicKey)?;

	if pubkey.to_string().as_str() != input_address {
		return Err(WalletError::WalletSignIn);
	}

	if signature.verify(&pubkey.to_bytes(), &signed_message) {
		Ok(())
	} else {
		Err(WalletError::WalletSignIn)
	}
}

#[allow(unused_assignments)]
fn verify_output_text(input: &SolanaSignInInput, signed_message: &[u8]) -> WalletResult<()> {
	let input_text = create_sign_in_message_text(input)?;
	let output_text = str::from_utf8(signed_message)
		.map_err(|_| WalletError::ParseString("could not parse the signed_message".into()))?;

	if input_text == output_text {
		return Ok(());
	}

	let domain = input
		.domain
		.as_ref()
		.ok_or(WalletError::WalletSignInFields("Domain is required".into()))?;
	let address = input
		.address
		.as_ref()
		.ok_or(WalletError::WalletSignInFields(
			"Address is required".into(),
		))?;

	let Some((_, output_text)) = output_text
		.split_once(format!("{domain} wants you to sign in with your Solana account:").as_str())
	else {
		return Err(WalletError::WalletSignIn);
	};

	let Some((_, mut output_text)) = output_text.split_once(address) else {
		return Err(WalletError::WalletSignIn);
	};

	if let Some(ref statement) = input.statement {
		let Some((_, remaining)) = output_text.split_once(statement) else {
			return Err(WalletError::WalletSignIn);
		};

		output_text = remaining;
	}

	// Use macros for concise field appending
	macro_rules! confirm_field_exists {
		($field:expr, $name:literal) => {
			if let Some(ref value) = $field {
				let Some((_, remaining)) =
					output_text.split_once(format!("{}: {}", $name, value).as_str())
				else {
					return Err(WalletError::WalletSignIn);
				};

				output_text = remaining;
			}
		};
	}

	confirm_field_exists!(input.uri, "URI");
	confirm_field_exists!(input.version, "Version");
	confirm_field_exists!(input.chain_id, "Chain ID");
	confirm_field_exists!(input.nonce, "Nonce");
	confirm_field_exists!(input.issued_at, "Issued At");
	confirm_field_exists!(input.expiration_time, "Expiration Time");
	confirm_field_exists!(input.not_before, "Not Before");
	confirm_field_exists!(input.request_id, "Request ID");

	if let Some(ref resources) = input.resources {
		let Some((_, remaining)) = output_text.split_once("Resources:") else {
			return Err(WalletError::WalletSignIn);
		};

		output_text = remaining;

		for resource in resources {
			let Some((_, remaining)) = output_text.split_once(format!("- {resource}").as_str())
			else {
				return Err(WalletError::WalletSignIn);
			};

			output_text = remaining;
		}
	}

	Ok(())
}

/// ```markup
/// ${domain} wants you to sign in with your Solana account:
/// ${address}
///
/// ${statement}
///
/// URI: ${uri}
/// Version: ${version}
/// Chain ID: ${chain}
/// Nonce: ${nonce}
/// Issued At: ${issued_at}
/// Expiration Time: ${expiration_time}
/// Not Before: ${not_before}
/// Request ID: ${request_id}
/// Resources:
/// - ${resources[0]}
/// - ${resources[1]}
/// ...
/// - ${resources[n]}
/// ```
pub fn create_sign_in_message_text(input: &SolanaSignInInput) -> WalletResult<String> {
	let mut message = String::with_capacity(256); // Pre-allocate for efficiency
	let domain = input
		.domain
		.as_ref()
		.ok_or(WalletError::WalletSignInFields("Domain is required".into()))?;
	let address = input
		.address
		.as_ref()
		.ok_or(WalletError::WalletSignInFields(
			"Address is required".into(),
		))?;

	write!(
		&mut message,
		"{domain} wants you to sign in with your Solana account:\n{address}"
	)?;

	if let Some(ref statement) = input.statement {
		write!(&mut message, "\n\n{statement}")?;
	}

	let mut fields = Vec::with_capacity(10); // Estimate number of fields for allocation

	// Use macros for concise field appending
	macro_rules! push_field {
		($field:expr, $name:literal) => {
			if let Some(ref value) = $field {
				fields.push(format!("{}: {}", $name, value));
			}
		};
	}

	push_field!(input.uri, "URI");
	push_field!(input.version, "Version");
	push_field!(input.chain_id, "Chain ID");
	push_field!(input.nonce, "Nonce");
	push_field!(input.issued_at, "Issued At");
	push_field!(input.expiration_time, "Expiration Time");
	push_field!(input.not_before, "Not Before");
	push_field!(input.request_id, "Request ID");

	if let Some(ref resources) = input.resources {
		fields.push("Resources:".to_string());

		for resource in resources {
			fields.push(format!("- {resource}"));
		}
	}

	if !fields.is_empty() {
		write!(&mut message, "\n\n{}", fields.join("\n"))?;
	}

	Ok(message)
}

#[cfg(test)]
mod tests {
	use solana_keypair::Keypair;
	use solana_signature::Signature;
	use solana_signer::Signer;

	use super::*;

	struct TestAccount {
		address: String,
		public_key: Vec<u8>,
	}

	impl WalletAccountInfo for TestAccount {
		fn address(&self) -> String {
			self.address.clone()
		}

		fn public_key(&self) -> Vec<u8> {
			self.public_key.clone()
		}

		fn chains(&self) -> Vec<String> {
			Vec::new()
		}

		fn features(&self) -> Vec<String> {
			Vec::new()
		}

		fn label(&self) -> Option<String> {
			None
		}

		fn icon(&self) -> Option<String> {
			None
		}
	}

	struct TestSignInOutput {
		account: TestAccount,
		signed_message: Vec<u8>,
		signature: Signature,
	}

	impl SolanaSignatureOutput for TestSignInOutput {
		fn try_signature(&self) -> WalletResult<Signature> {
			Ok(self.signature)
		}

		fn signature(&self) -> Signature {
			self.signature
		}
	}

	impl SolanaSignMessageOutput for TestSignInOutput {
		fn signed_message(&self) -> Vec<u8> {
			self.signed_message.clone()
		}

		fn signature_type(&self) -> Option<String> {
			None
		}
	}

	impl SolanaSignInOutput for TestSignInOutput {
		type Account = TestAccount;

		fn account(&self) -> Self::Account {
			TestAccount {
				address: self.account.address.clone(),
				public_key: self.account.public_key.clone(),
			}
		}
	}

	fn keypair() -> Keypair {
		Keypair::new_from_array([7; 32])
	}

	fn input() -> SolanaSignInInput {
		let address = keypair().pubkey().to_string();

		SolanaSignInInput::builder()
			.domain("example.com")
			.address(address)
			.statement("Accept the terms of service.")
			.uri("https://example.com")
			.version("1")
			.chain_id("solana:mainnet")
			.nonce("nonce-1")
			.issued_at("2026-10-16T00:00:00Z")
			.resources(vec![
				"https://example.com/terms".to_string(),
				"https://example.com/avatar.png".to_string(),
			])
			.build()
	}

	fn signed_output(input: &SolanaSignInInput) -> TestSignInOutput {
		signed_output_with_message(input, create_sign_in_message_text(input).unwrap())
	}

	fn signed_output_with_message(input: &SolanaSignInInput, message: String) -> TestSignInOutput {
		let keypair = keypair();
		let signature = Signer::try_sign_message(&keypair, message.as_bytes()).unwrap();

		TestSignInOutput {
			account: TestAccount {
				address: input.address.clone().unwrap(),
				public_key: keypair.pubkey().to_bytes().to_vec(),
			},
			signed_message: message.into_bytes(),
			signature,
		}
	}

	#[test]
	fn verify_sign_in_accepts_a_signed_statement_with_resources() {
		let input = input();

		verify_sign_in(&input, &signed_output(&input)).unwrap();
	}

	#[test]
	fn verify_sign_in_rejects_a_statement_omitting_the_resources() {
		let input = input();
		// The wallet signed a statement that leaves the resources block out.
		let message = create_sign_in_message_text(
			&SolanaSignInInput::builder()
				.domain(input.domain.clone().unwrap())
				.address(input.address.clone().unwrap())
				.statement(input.statement.clone().unwrap())
				.uri(input.uri.clone().unwrap())
				.version(input.version.clone().unwrap())
				.chain_id(input.chain_id.clone().unwrap())
				.nonce(input.nonce.clone().unwrap())
				.issued_at(input.issued_at.clone().unwrap())
				.build(),
		)
		.unwrap();

		assert!(verify_sign_in(&input, &signed_output_with_message(&input, message)).is_err());
	}

	#[test]
	fn verify_sign_in_rejects_a_statement_missing_one_resource() {
		let input = input();
		let message = create_sign_in_message_text(
			&SolanaSignInInput::builder()
				.domain(input.domain.clone().unwrap())
				.address(input.address.clone().unwrap())
				.statement(input.statement.clone().unwrap())
				.uri(input.uri.clone().unwrap())
				.version(input.version.clone().unwrap())
				.chain_id(input.chain_id.clone().unwrap())
				.nonce(input.nonce.clone().unwrap())
				.issued_at(input.issued_at.clone().unwrap())
				.resources(vec!["https://example.com/terms".to_string()])
				.build(),
		)
		.unwrap();

		assert!(verify_sign_in(&input, &signed_output_with_message(&input, message)).is_err());
	}

	#[test]
	fn verify_sign_in_rejects_a_bad_signature() {
		let input = input();
		let mut output = signed_output(&input);

		output.signed_message[0] ^= 1;

		assert!(verify_sign_in(&input, &output).is_err());
	}

	#[test]
	fn create_sign_in_message_text_lists_every_field_in_order() {
		let text = create_sign_in_message_text(&input()).unwrap();
		let expected = format!(
			"example.com wants you to sign in with your Solana account:\n{}\
			\n\nAccept the terms of service.\
			\n\nURI: https://example.com\
			\nVersion: 1\
			\nChain ID: solana:mainnet\
			\nNonce: nonce-1\
			\nIssued At: 2026-10-16T00:00:00Z\
			\nResources:\n- https://example.com/terms\n- https://example.com/avatar.png",
			keypair().pubkey()
		);

		assert_eq!(text, expected);
	}
}
