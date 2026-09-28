//! Browser tests for the `wallet_standard_browser` bridge.
//!
//! These run under `wasm-bindgen-test` in a real browser:
//!
//! ```sh
//! cargo test --package wallet_standard_browser --all-features \
//!   --target wasm32-unknown-unknown
//! ```

#![allow(clippy::unused_async)]

use js_sys::Array;
use js_sys::Function;
use js_sys::Object;
use js_sys::Reflect;
use js_sys::Uint8Array;
use wallet_standard::Wallet;
use wallet_standard::WalletAccountInfo;
use wallet_standard::WalletInfo;
use wallet_standard::WalletStandardConnect;
use wallet_standard_browser::BrowserWallet;
use wallet_standard_browser::BrowserWalletAccountInfo;
use wallet_standard_browser::BrowserWalletAccountInfoProps;
use wallet_standard_browser::BrowserWalletInfo;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn js_str(value: &str) -> JsValue {
	JsValue::from_str(value)
}

fn set(target: &Object, key: &str, value: impl Into<JsValue>) {
	Reflect::set(target.as_ref(), &js_str(key), &value.into())
		.expect("setting plain properties never fails");
}

/// A feature object backed by real JavaScript, so feature lookups and method
/// calls exercise the same bridge a dApp uses against injected wallets.
fn mock_feature(version: &str, methods: &[(&str, &str)]) -> Object {
	let feature = Object::new();
	set(&feature, "version", js_str(version));
	for (name, body) in methods {
		set(&feature, name, Function::new_no_args(body));
	}
	feature
}

/// A mock wallet whose `standard:connect` resolves a single account.
fn mock_wallet(name: &str, features: &[(&str, Object)]) -> BrowserWalletInfo {
	let wallet = Object::new();
	set(&wallet, "version", js_str("1.0.0"));
	set(&wallet, "name", js_str(name));
	set(
		&wallet,
		"icon",
		js_str("data:image/svg+xml;base64,PHN2Zy8+"),
	);

	let chains = Array::new();
	chains.push(&js_str("solana:mainnet"));
	chains.push(&js_str("solana:devnet"));
	set(&wallet, "chains", chains);

	let features_object = Object::new();
	for (feature_name, feature) in features {
		set(&features_object, feature_name, feature.clone());
	}
	set(&wallet, "features", features_object);
	set(&wallet, "accounts", Array::new());

	wallet.unchecked_into()
}

fn connect_feature() -> Object {
	mock_feature(
		"1.0.0",
		&[(
			"connect",
			r#"return Promise.resolve({
				accounts: [{
					address: "4acVT7jfykgHZNunZYEwg1NNCUnvuXwFH292ebEGnN4g",
					publicKey: new Uint8Array(32).fill(7),
					chains: ["solana:mainnet"],
					features: ["solana:signMessage"],
					label: "Mock",
					icon: "data:image/svg+xml;base64,PHN2Zy8+",
				}],
			});"#,
		)],
	)
}

#[wasm_bindgen_test]
pub fn wallet_info_reads_injected_wallet() {
	let info = mock_wallet(
		"InfoMock",
		&[
			("standard:connect", connect_feature()),
			(
				"standard:disconnect",
				mock_feature("1.0.0", &[("disconnect", "return Promise.resolve();")]),
			),
			(
				"standard:events",
				mock_feature("1.0.0", &[("on", "return function () {};")]),
			),
			(
				"solana:signMessage",
				mock_feature(
					"1.0.0",
					&[(
						"signMessage",
						"return Promise.resolve([{
							signedMessage: new Uint8Array([1, 2, 3]),
							signature: new Uint8Array(64).fill(9),
						}]);",
					)],
				),
			),
		],
	);

	assert_eq!(info.name(), "InfoMock");
	assert_eq!(info.version(), "1.0.0");
	assert_eq!(info.chains(), vec!["solana:mainnet", "solana:devnet"]);
	assert!(info.features().contains(&"standard:connect".to_string()));
	assert!(info.features().contains(&"solana:signMessage".to_string()));
	assert!(info.is_standard_compatible());

	let wallet = BrowserWallet::from(info);
	assert_eq!(wallet.name(), "InfoMock");
	assert!(!wallet.connected());
}

#[wasm_bindgen_test]
pub fn try_register_and_discover_wallet() {
	let info = mock_wallet("RegistryMock", &[("standard:connect", connect_feature())]);

	let dispose = wallet_standard_browser::get_wallets()
		.try_register(&[info])
		.expect("registering a well-formed wallet succeeds");
	assert!(
		wallet_standard_browser::get_wallets()
			.get()
			.iter()
			.any(|wallet| wallet.name() == "RegistryMock")
	);
	dispose();
}

#[wasm_bindgen_test]
pub async fn connect_updates_the_attached_account() {
	let info = mock_wallet("ConnectMock", &[("standard:connect", connect_feature())]);
	let mut wallet = BrowserWallet::from(info);

	let accounts = wallet
		.connect_with_options(wallet_standard::StandardConnectInput::default())
		.await
		.expect("connect resolves");
	assert_eq!(accounts.len(), 1);
	assert_eq!(
		accounts[0].address(),
		"4acVT7jfykgHZNunZYEwg1NNCUnvuXwFH292ebEGnN4g"
	);

	// The connect trait implementation must attach the account and remember
	// it, not recurse into itself.
	assert_eq!(
		wallet
			.wallet_account
			.as_ref()
			.expect("connect attaches the account")
			.address(),
		"4acVT7jfykgHZNunZYEwg1NNCUnvuXwFH292ebEGnN4g"
	);
	assert!(wallet.connected());
}

// --- Boundary-shape regression tests ---------------------------------------
//
// These pin the exact JavaScript shape that crosses the wasm boundary. The
// signing features silently produced `{}` (and before that `{"Ok":[{}]}`)
// because `#[serde(flatten)]` is not supported by `serde-wasm-bindgen`; every
// input struct gets a shape test so a regression fails loudly here instead of
// silently in dApps.

fn mock_account() -> BrowserWalletAccountInfo {
	BrowserWalletAccountInfo::try_new(
		&BrowserWalletAccountInfoProps::builder()
			.address("4acVT7jfykgHZNunZYEwg1NNCUnvuXwFH292ebEGnN4g")
			.public_key(vec![0u8; 32])
			.build(),
	)
	.expect("mock account props are valid")
}

fn shape_of(value: &JsValue) -> Vec<String> {
	Object::keys(
		value
			.dyn_ref::<Object>()
			.expect("shape targets are objects"),
	)
	.iter()
	.filter_map(|key| key.as_string())
	.collect()
}

#[cfg(feature = "solana")]
#[wasm_bindgen_test]
pub fn sign_message_input_serializes_to_the_wire_format() {
	let input = wallet_standard_browser::SolanaSignMessageInput::builder()
		.account(mock_account())
		.message(vec![1, 2, 3])
		.build();

	let value = serde_wasm_bindgen::to_value(&vec![input]).expect("serializes");
	let array: Array = value.dyn_into().expect("a batch serializes to an array");
	assert_eq!(array.length(), 1);

	let element = array.get(0);
	assert_eq!(
		shape_of(&element),
		vec!["account".to_string(), "message".to_string()]
	);
	assert!(
		Reflect::get(&element, &js_str("message"))
			.expect("message key exists")
			.is_instance_of::<Uint8Array>(),
		"message bytes must cross as a Uint8Array, not a JSON array"
	);
}

#[cfg(feature = "solana")]
#[wasm_bindgen_test]
pub fn sign_transaction_input_serializes_to_the_wire_format() {
	let message = solana_message::Message::new(&[], None);
	let transaction = solana_transaction::versioned::VersionedTransaction::from(
		solana_transaction::Transaction::new_unsigned(message),
	);

	let input = wallet_standard_browser::SolanaSignTransactionInput::builder()
		.account(mock_account())
		.transaction(bincode::serialize(&transaction).expect("wire bytes"))
		.chain(Some("solana:devnet".to_string()))
		.build();

	let value = serde_wasm_bindgen::to_value(&vec![input]).expect("serializes");
	let array: Array = value.dyn_into().expect("a batch serializes to an array");
	assert_eq!(array.length(), 1);

	let element = array.get(0);
	assert_eq!(
		shape_of(&element),
		vec![
			"account".to_string(),
			"transaction".to_string(),
			"chain".to_string(),
			"options".to_string(),
		]
	);
	assert!(
		Reflect::get(&element, &js_str("transaction"))
			.expect("transaction key exists")
			.is_instance_of::<Uint8Array>(),
		"transaction bytes must cross as a Uint8Array"
	);
}

#[cfg(feature = "solana")]
#[wasm_bindgen_test]
pub fn sign_and_send_input_serializes_to_the_wire_format() {
	let message = solana_message::Message::new(&[], None);
	let transaction = solana_transaction::versioned::VersionedTransaction::from(
		solana_transaction::Transaction::new_unsigned(message),
	);

	let input = wallet_standard_browser::SolanaSignAndSendTransactionInput::builder()
		.account(mock_account())
		.transaction(bincode::serialize(&transaction).expect("wire bytes"))
		.build();

	let value = serde_wasm_bindgen::to_value(&vec![input]).expect("serializes");
	let array: Array = value.dyn_into().expect("a batch serializes to an array");
	assert_eq!(array.length(), 1);

	let element = array.get(0);
	assert_eq!(
		shape_of(&element),
		vec![
			"account".to_string(),
			"transaction".to_string(),
			"chain".to_string(),
			"options".to_string(),
		]
	);
	assert!(
		Reflect::get(&element, &js_str("transaction"))
			.expect("transaction key exists")
			.is_instance_of::<Uint8Array>()
	);
}

#[wasm_bindgen_test]
pub fn decrypt_props_serialize_ciphertext_as_one_word() {
	let props = wallet_standard::ExperimentalDecryptProps::builder()
		.cipher("x25519-xsalsa20-poly1305")
		.public_key(vec![0u8; 32])
		.cipher_text(vec![7u8; 24])
		.nonce(vec![1u8; 24])
		.build();

	let value = serde_wasm_bindgen::to_value(&props).expect("serializes");
	assert!(
		Reflect::get(&value, &js_str("ciphertext")).is_ok(),
		"the wallet-standard experimental spec spells the field `ciphertext`"
	);
	let camel_case = Reflect::get(&value, &js_str("cipherText"));
	assert!(
		!camel_case.is_ok_and(|v| !v.is_undefined()),
		"the camelCase spelling must not be produced"
	);
}

#[wasm_bindgen_test]
pub fn encrypt_output_reads_the_spec_field_names() {
	let output = Object::new();
	set(
		&output,
		"ciphertext",
		Uint8Array::from(&[3u8, 1u8, 4u8][..]),
	);
	set(&output, "nonce", Uint8Array::from(&[9u8; 24][..]));

	let output: wallet_standard_browser::BrowserExperimentalEncryptOutput = output.unchecked_into();
	use wallet_standard::ExperimentalEncryptOutput;

	assert_eq!(output.cipher_text(), vec![3u8, 1u8, 4u8]);
	assert_eq!(output.nonce(), vec![9u8; 24]);
}

#[wasm_bindgen_test]
pub fn js_rejections_keep_their_error_message() {
	let error = wallet_standard::WalletError::from(JsValue::from(js_sys::Error::new("boom")));
	assert_eq!(error, wallet_standard::WalletError::Js("boom".to_string()));

	let plain = wallet_standard::WalletError::from(js_str("plain rejection"));
	assert_eq!(
		plain,
		wallet_standard::WalletError::Js("plain rejection".to_string())
	);
}
