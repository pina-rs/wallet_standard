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

/// Check that a Sign In With Solana output matches the input the app sent.
///
/// This follows `verifySignIn` from `@solana/wallet-standard-util`. The
/// signed bytes must parse as a SIWS message, every field the app set must
/// equal the parsed field, and every field the app left unset must be
/// absent. The account address and public key must be the address inside
/// that message, and the Ed25519 signature must verify over those exact
/// bytes. A message that only embeds the expected fields as substrings —
/// inside a different statement, or after a different domain — is rejected.
///
/// # Errors
///
/// Returns [`WalletError::WalletSignIn`] when the message, fields, account,
/// or signature do not match. Returns [`WalletError::WalletPublicKey`] when
/// the account public key is not a Solana public key, and
/// [`WalletError::ParseString`] when the signed bytes are not UTF-8.
pub fn verify_sign_in(
	input: &SolanaSignInInput,
	output: &impl SolanaSignInOutput,
) -> WalletResult<()> {
	let account = output.account();
	// `signed_message` crosses the wasm boundary once and feeds both the
	// text check and the signature check.
	let signed_message = output.signed_message();
	let output_text = str::from_utf8(&signed_message)
		.map_err(|_| WalletError::ParseString("could not parse the signed_message".into()))?;
	let parsed = parse_sign_in_message_text(output_text).ok_or(WalletError::WalletSignIn)?;

	if account.address() != parsed.address {
		return Err(WalletError::WalletSignIn);
	}

	require_supplied(input.domain.as_ref(), &parsed.domain)?;
	require_supplied(input.address.as_ref(), &parsed.address)?;
	require_exact(input.statement.as_ref(), parsed.statement.as_ref())?;
	require_exact(input.uri.as_ref(), parsed.uri.as_ref())?;
	require_exact(input.version.as_ref(), parsed.version.as_ref())?;
	require_exact(input.chain_id.as_ref(), parsed.chain_id.as_ref())?;
	require_exact(input.nonce.as_ref(), parsed.nonce.as_ref())?;
	require_exact(input.issued_at.as_ref(), parsed.issued_at.as_ref())?;
	require_exact(
		input.expiration_time.as_ref(),
		parsed.expiration_time.as_ref(),
	)?;
	require_exact(input.not_before.as_ref(), parsed.not_before.as_ref())?;
	require_exact(input.request_id.as_ref(), parsed.request_id.as_ref())?;
	require_resources(input.resources.as_ref(), parsed.resources.as_ref())?;

	// Rebuild the message the parser accepted. Anything the grammar allowed
	// but `create_sign_in_message_text` would not emit — extra trailing
	// newlines, for example — fails this equality before the signature is
	// even consulted.
	let reconstructed = create_sign_in_message_text(&parsed.to_input())?;

	if reconstructed != output_text {
		return Err(WalletError::WalletSignIn);
	}

	let signature = output.try_signature()?;
	let pubkey =
		Pubkey::try_from(account.public_key()).map_err(|_| WalletError::WalletPublicKey)?;

	if pubkey.to_string() != parsed.address {
		return Err(WalletError::WalletSignIn);
	}

	if signature.verify(pubkey.as_ref(), &signed_message) {
		Ok(())
	} else {
		Err(WalletError::WalletSignIn)
	}
}

/// A SIWS message parsed the way `@solana/wallet-standard-util` parses it.
struct ParsedSignIn {
	domain: String,
	address: String,
	statement: Option<String>,
	uri: Option<String>,
	version: Option<String>,
	chain_id: Option<String>,
	nonce: Option<String>,
	issued_at: Option<String>,
	expiration_time: Option<String>,
	not_before: Option<String>,
	request_id: Option<String>,
	resources: Option<Vec<String>>,
}

impl ParsedSignIn {
	fn to_input(&self) -> SolanaSignInInput {
		SolanaSignInInput {
			domain: Some(self.domain.clone()),
			address: Some(self.address.clone()),
			statement: self.statement.clone(),
			uri: self.uri.clone(),
			version: self.version.clone(),
			chain_id: self.chain_id.clone(),
			nonce: self.nonce.clone(),
			issued_at: self.issued_at.clone(),
			expiration_time: self.expiration_time.clone(),
			not_before: self.not_before.clone(),
			request_id: self.request_id.clone(),
			resources: self.resources.clone(),
		}
	}
}

/// `provided` is optional in the app's input. When the app set it, it must
/// equal the value the wallet put in the message.
fn require_supplied(provided: Option<&String>, parsed: &str) -> WalletResult<()> {
	if let Some(value) = provided
		&& value != parsed
	{
		return Err(WalletError::WalletSignIn);
	}

	Ok(())
}

/// An omitted input field must be omitted from the signed message, and a
/// set field must match exactly. A wallet that invents a statement or a
/// nonce the app did not send fails here.
fn require_exact(provided: Option<&String>, parsed: Option<&String>) -> WalletResult<()> {
	if provided == parsed {
		Ok(())
	} else {
		Err(WalletError::WalletSignIn)
	}
}

fn require_resources(
	provided: Option<&Vec<String>>,
	parsed: Option<&Vec<String>>,
) -> WalletResult<()> {
	if provided == parsed {
		Ok(())
	} else {
		Err(WalletError::WalletSignIn)
	}
}

/// Parse a SIWS message.
///
/// The shape matches the reference regular expression: a domain line, an
/// address line, an optional statement, then the optional fields in order.
/// The statement is the shortest span that still lets the remaining text
/// parse as those fields, which is what the non-greedy reference pattern
/// does.
fn parse_sign_in_message_text(text: &str) -> Option<ParsedSignIn> {
	let (domain_line, rest) = text.split_once('\n')?;
	let suffix = " wants you to sign in with your Solana account:";
	let domain = domain_line.strip_suffix(suffix)?;

	if domain.is_empty() {
		return None;
	}

	let (address, body) = match rest.split_once('\n') {
		Some((address, body)) => (address, body),
		None => (rest, ""),
	};

	if address.is_empty() {
		return None;
	}

	let (statement, fields) = parse_statement_and_fields(body)?;

	Some(ParsedSignIn {
		domain: domain.to_owned(),
		address: address.to_owned(),
		statement,
		uri: fields.uri,
		version: fields.version,
		chain_id: fields.chain_id,
		nonce: fields.nonce,
		issued_at: fields.issued_at,
		expiration_time: fields.expiration_time,
		not_before: fields.not_before,
		request_id: fields.request_id,
		resources: fields.resources,
	})
}

struct SignInFields {
	uri: Option<String>,
	version: Option<String>,
	chain_id: Option<String>,
	nonce: Option<String>,
	issued_at: Option<String>,
	expiration_time: Option<String>,
	not_before: Option<String>,
	request_id: Option<String>,
	resources: Option<Vec<String>>,
}

fn parse_statement_and_fields(body: &str) -> Option<(Option<String>, SignInFields)> {
	if let Some(fields) = finish_fields(body) {
		return Some((None, fields));
	}

	let rest = body.strip_prefix('\n')?;
	let mut index = 0;

	loop {
		if !rest.is_char_boundary(index) {
			index += 1;
			continue;
		}

		let statement = &rest[..index];
		let after_statement = &rest[index..];
		let fields = if after_statement.is_empty() {
			finish_fields("")
		} else {
			after_statement.strip_prefix('\n').and_then(finish_fields)
		};

		if let Some(fields) = fields {
			return Some((Some(statement.to_owned()), fields));
		}

		if index == rest.len() {
			break;
		}

		index += 1;
	}

	None
}

fn finish_fields(input: &str) -> Option<SignInFields> {
	let (fields, rest) = take_fields(input);

	if rest.bytes().all(|byte| byte == b'\n') {
		Some(fields)
	} else {
		None
	}
}

fn take_fields(mut input: &str) -> (SignInFields, &str) {
	let mut fields = SignInFields {
		uri: None,
		version: None,
		chain_id: None,
		nonce: None,
		issued_at: None,
		expiration_time: None,
		not_before: None,
		request_id: None,
		resources: None,
	};

	if let Some((value, rest)) = take_labeled(input, "URI") {
		fields.uri = Some(value);
		input = rest;
	}

	if let Some((value, rest)) = take_labeled(input, "Version") {
		fields.version = Some(value);
		input = rest;
	}

	if let Some((value, rest)) = take_labeled(input, "Chain ID") {
		fields.chain_id = Some(value);
		input = rest;
	}

	if let Some((value, rest)) = take_labeled(input, "Nonce") {
		fields.nonce = Some(value);
		input = rest;
	}

	if let Some((value, rest)) = take_labeled(input, "Issued At") {
		fields.issued_at = Some(value);
		input = rest;
	}

	if let Some((value, rest)) = take_labeled(input, "Expiration Time") {
		fields.expiration_time = Some(value);
		input = rest;
	}

	if let Some((value, rest)) = take_labeled(input, "Not Before") {
		fields.not_before = Some(value);
		input = rest;
	}

	if let Some((value, rest)) = take_labeled(input, "Request ID") {
		fields.request_id = Some(value);
		input = rest;
	}

	if let Some((resources, rest)) = take_resources(input) {
		fields.resources = Some(resources);
		input = rest;
	}

	(fields, input)
}

fn take_labeled<'a>(input: &'a str, label: &str) -> Option<(String, &'a str)> {
	let prefix = format!("\n{label}: ");
	let rest = input.strip_prefix(&prefix)?;
	let value_len = rest.find('\n').unwrap_or(rest.len());
	let value = &rest[..value_len];

	if value.is_empty() {
		return None;
	}

	Some((value.to_owned(), &rest[value_len..]))
}

fn take_resources(input: &str) -> Option<(Vec<String>, &str)> {
	let mut rest = input.strip_prefix("\nResources:")?;
	let mut resources = Vec::new();

	while let Some(after_marker) = rest.strip_prefix("\n- ") {
		let value_len = after_marker.find('\n').unwrap_or(after_marker.len());
		let value = &after_marker[..value_len];

		if value.is_empty() {
			break;
		}

		resources.push(value.to_owned());
		rest = &after_marker[value_len..];
	}

	Some((resources, rest))
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
	fn verify_sign_in_rejects_a_message_that_only_embeds_the_fields() {
		let input = input();
		let address = input.address.clone().unwrap();
		// The real domain is not the first line. A substring search still
		// finds every required field later in the text.
		let message = format!(
			"evil.example wants you to sign in with your Solana account:\n{address}\n\n\
			 Transfer everything. Also: example.com wants you to sign in with your Solana account:\n{address}\n\n\
			 Accept the terms of service.\n\n\
			 URI: https://example.com\n\
			 Version: 1\n\
			 Chain ID: solana:mainnet\n\
			 Nonce: nonce-1\n\
			 Issued At: 2026-10-16T00:00:00Z\n\
			 Resources:\n\
			 - https://example.com/terms\n\
			 - https://example.com/avatar.png"
		);

		assert!(verify_sign_in(&input, &signed_output_with_message(&input, message)).is_err());
	}

	#[test]
	fn verify_sign_in_rejects_a_statement_with_a_smuggled_prefix() {
		let input = input();
		let address = input.address.clone().unwrap();
		let message = format!(
			"example.com wants you to sign in with your Solana account:\n{address}\n\n\
			 I authorize a transfer. Accept the terms of service.\n\n\
			 URI: https://example.com\n\
			 Version: 1\n\
			 Chain ID: solana:mainnet\n\
			 Nonce: nonce-1\n\
			 Issued At: 2026-10-16T00:00:00Z\n\
			 Resources:\n\
			 - https://example.com/terms\n\
			 - https://example.com/avatar.png"
		);

		assert!(verify_sign_in(&input, &signed_output_with_message(&input, message)).is_err());
	}

	#[test]
	fn verify_sign_in_rejects_a_field_the_app_did_not_send() {
		let mut constrained = input();
		constrained.issued_at = None;
		let message = create_sign_in_message_text(&input()).unwrap();

		assert!(
			verify_sign_in(
				&constrained,
				&signed_output_with_message(&constrained, message)
			)
			.is_err()
		);
	}

	#[test]
	fn verify_sign_in_accepts_an_omitted_address_when_the_account_matches() {
		let full = input();
		let output = signed_output(&full);
		let mut partial = full.clone();
		partial.address = None;

		verify_sign_in(&partial, &output).unwrap();
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
