//! The "Surfpool Dev Wallet": a complete Wallet Standard wallet implemented in
//! Rust and registered into the page from WASM.
//!
//! This mirrors what a real Rust-based browser wallet (e.g. a wallet extension
//! written in Rust) would do:
//!
//! 1. Build the JS wallet object with `standard:connect`,
//!    `standard:disconnect`, `standard:events`, `solana:signMessage`,
//!    `solana:signTransaction` and `solana:signAndSendTransaction` features
//!    backed by Rust closures.
//! 2. Call [`register_wallet`] so every Wallet Standard app on the page can
//!    discover it.
//! 3. Keep the private key inside the WASM heap; only signatures cross the JS
//!    boundary.

use std::cell::RefCell;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use js_sys::Array;
use js_sys::Function;
use js_sys::Object;
use js_sys::Promise;
use js_sys::Reflect;
use js_sys::Uint8Array;
use solana_keypair::Keypair;
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use wallet_standard_browser::BrowserWallet;
use wallet_standard_browser::BrowserWalletAccountInfo;
use wallet_standard_browser::BrowserWalletInfo;
use wallet_standard_browser::register_wallet;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;

use crate::tx;

pub const DEV_WALLET_NAME: &str = "Surfpool Dev Wallet";
pub const DEV_WALLET_VERSION: &str = "1.0.0";

const DEV_WALLET_ICON: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIzMiIgaGVpZ2h0PSIzMiIgdmlld0JveD0iMCAwIDMyIDMyIj48cmVjdCB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHJ4PSI2IiBmaWxsPSIjMmQzMDRmIi8+PHBhdGggZD0iTTYgMTJoMjBNNiAxNGgyME02IDE2aDIwTTYgMThoMjAiIHN0cm9rZT0iIzNhZGZmZiIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiLz48L3N2Zz4=";

/// Throwaway keypair seed. It is only ever funded on a local surfpool
/// instance and exists so the Playwright suite is fully deterministic.
#[rustfmt::skip]
const DEV_WALLET_SEED: [u8; 32] = [
	218, 142, 230, 74, 239, 167, 140, 243, 207, 148, 107, 50, 35, 37, 148, 103,
	175, 206, 218, 127, 153, 77, 191, 218, 28, 69, 70, 33, 226, 188, 199, 93,
];

const ACCOUNT_FEATURES: [&str; 6] = [
	"standard:connect",
	"standard:disconnect",
	"standard:events",
	"solana:signMessage",
	"solana:signTransaction",
	"solana:signAndSendTransaction",
];

const CHAINS: [&str; 2] = ["solana:mainnet", "solana:devnet"];

thread_local! {
	static KEYPAIR: Keypair = Keypair::new_from_array(DEV_WALLET_SEED);
	static RPC_URL: RefCell<String> = RefCell::new(crate::DEFAULT_RPC_URL.to_string());
	static CONNECTED: RefCell<bool> = const { RefCell::new(false) };
	static CHANGE_LISTENER: RefCell<Option<Function>> = const { RefCell::new(None) };
	/// Feature closures must live as long as the page.
	static KEEP_ALIVE: RefCell<Vec<JsValue>> = const { RefCell::new(Vec::new()) };
}

#[must_use]
pub fn dev_pubkey() -> solana_pubkey::Pubkey {
	KEYPAIR.with(Keypair::pubkey)
}

#[must_use]
pub fn dev_address() -> String {
	dev_pubkey().to_string()
}

fn js_str(value: &str) -> JsValue {
	JsValue::from_str(value)
}

fn set(target: &Object, key: &str, value: impl Into<JsValue>) {
	Reflect::set(target.as_ref(), &js_str(key), &value.into())
		.expect("setting plain properties never fails");
}

fn js_to_bytes(value: &JsValue) -> Vec<u8> {
	if let Ok(array) = value.clone().dyn_into::<Uint8Array>() {
		return array.to_vec();
	}

	if let Ok(array) = value.clone().dyn_into::<Array>() {
		return array
			.iter()
			.filter_map(|item| item.as_f64().map(|number| number as u8))
			.collect();
	}

	Vec::new()
}

fn bytes_to_js(bytes: &[u8]) -> JsValue {
	Uint8Array::from(bytes).into()
}

fn account_object() -> Object {
	let account = Object::new();
	set(&account, "address", js_str(&dev_address()));
	set(&account, "publicKey", bytes_to_js(&dev_pubkey().to_bytes()));
	set(
		&account,
		"chains",
		Array::from_iter(CHAINS.iter().copied().map(js_str)),
	);
	set(
		&account,
		"features",
		Array::from_iter(ACCOUNT_FEATURES.iter().copied().map(js_str)),
	);
	set(&account, "label", js_str("Surfpool Dev Account"));
	set(&account, "icon", js_str(DEV_WALLET_ICON));
	account
}

fn accounts_array() -> Array {
	let accounts = Array::new();

	if CONNECTED.with(|connected| *connected.borrow()) {
		accounts.push(&account_object().into());
	}

	accounts
}

fn emit_change() {
	let Some(listener) = CHANGE_LISTENER.with(|listener| listener.borrow().clone()) else {
		return;
	};
	let properties = Object::new();
	set(&properties, "accounts", accounts_array());
	let _ = listener.call1(&JsValue::NULL, &properties.into());
}

fn sign_message_outputs(inputs: &[JsValue]) -> Result<JsValue, String> {
	let outputs = Array::new();

	for input in inputs {
		let message = Reflect::get(input, &js_str("message")).unwrap_or(JsValue::UNDEFINED);
		let message = js_to_bytes(&message);

		if message.is_empty() {
			return Err("`message` must be non-empty bytes".to_string());
		}

		let signature = KEYPAIR.with(|keypair| keypair.sign_message(&message));
		let output = Object::new();

		set(&output, "signedMessage", bytes_to_js(&message));
		set(&output, "signature", bytes_to_js(signature.as_ref()));
		outputs.push(&output.into());
	}

	Ok(outputs.into())
}

fn sign_wire_transaction(bytes: &[u8]) -> Result<VersionedTransaction, String> {
	let mut transaction = tx::deserialize_wire_transaction(bytes)
		.map_err(|error| format!("wallet could not parse transaction: {error}"))?;

	// The signature preimage for a legacy transaction is the bincode-encoded
	// message itself; the serde format of `VersionedMessage::Legacy` matches
	// the classic wire format.
	let preimage = match &transaction.message {
		solana_message::VersionedMessage::Legacy(message) => {
			bincode::serialize(message)
				.map_err(|error| format!("wallet could not serialize message: {error}"))?
		}
		_ => return Err("demo wallet only signs legacy transactions".to_string()),
	};

	let signature = KEYPAIR.with(|keypair| keypair.sign_message(&preimage));
	let signer_pubkey = dev_pubkey();
	let index = transaction
		.message
		.static_account_keys()
		.iter()
		.position(|key| *key == signer_pubkey)
		.ok_or_else(|| "signer is not a required signer of the transaction".to_string())?;

	if index >= transaction.signatures.len() {
		transaction
			.signatures
			.resize(index + 1, Signature::default());
	}

	transaction.signatures[index] = signature;
	Ok(transaction)
}

fn sign_transaction_outputs(inputs: &[JsValue]) -> Result<JsValue, String> {
	let outputs = Array::new();

	for input in inputs {
		let transaction = Reflect::get(input, &js_str("transaction")).unwrap_or(JsValue::UNDEFINED);
		let bytes = js_to_bytes(&transaction);
		let signed = sign_wire_transaction(&bytes)?;
		let output = Object::new();
		set(
			&output,
			"signedTransaction",
			bytes_to_js(&tx::serialize_wire_transaction(&signed)),
		);
		outputs.push(&output.into());
	}

	Ok(outputs.into())
}

fn sign_and_send_outputs(inputs: &[JsValue]) -> Result<JsValue, String> {
	let outputs = Array::new();

	for input in inputs {
		let transaction = Reflect::get(input, &js_str("transaction")).unwrap_or(JsValue::UNDEFINED);
		let bytes = js_to_bytes(&transaction);
		let signed = sign_wire_transaction(&bytes)?;

		// The wallet owns broadcasting for this feature. Real wallets would
		// consult their own node config; here we send to the surfpool RPC.
		let wire = tx::serialize_wire_transaction(&signed);
		let encoded = BASE64.encode(&wire);
		let rpc_url = RPC_URL.with(|url| url.borrow().clone());
		let client = crate::rpc::SurfpoolClient::new(rpc_url);

		wasm_bindgen_futures::spawn_local(async move {
			// Nothing awaits the send here; the dApp polls signature status.
			let _ = client.send_raw_base64(&encoded).await;
		});

		// Optimistically report the expected signature: surfpool derives the
		// first signature deterministically from the signed transaction.
		let signature = signed.signatures.first().cloned().unwrap_or_default();
		let output = Object::new();
		set(&output, "signature", bytes_to_js(signature.as_ref()));
		outputs.push(&output.into());
	}

	Ok(outputs.into())
}

fn keep<T>(closure: Closure<T>) -> JsValue
where
	T: ?Sized + wasm_bindgen::closure::WasmClosure + 'static,
{
	let value = closure.into_js_value();
	KEEP_ALIVE.with(|keep_alive| keep_alive.borrow_mut().push(value.clone()));
	value
}

macro_rules! variadic_closure {
	($body:expr) => {{
		let handler = move |first: JsValue, second: JsValue, third: JsValue| {
			let inputs: Vec<JsValue> = [first, second, third]
				.into_iter()
				.filter(|value| !value.is_undefined())
				.collect();
			Promise::new(&mut |resolve, reject| {
				let inputs = inputs.clone();
				wasm_bindgen_futures::spawn_local(async move {
					match $body(&inputs) {
						Ok(value) => {
							let _ = resolve.call1(&JsValue::NULL, &value);
						}
						Err(message) => {
							let _ = reject.call1(&JsValue::NULL, &js_str(&message));
						}
					}
				});
			})
		};
		let closure: Closure<dyn FnMut(JsValue, JsValue, JsValue) -> Promise> =
			Closure::new(handler);
		closure
	}};
}

/// Builds the wallet JS object with every feature backed by Rust closures and
/// registers it on the page.
///
/// # Errors
///
/// Returns the underlying JS error if registration fails.
pub fn register_dev_wallet(rpc_url: String) -> Result<(), JsValue> {
	RPC_URL.with(|url| *url.borrow_mut() = rpc_url);

	let features = Object::new();

	// standard:connect
	let connect = Object::new();
	set(&connect, "version", js_str("1.0.0"));
	let connect_fn = variadic_closure!(|_inputs: &[JsValue]| {
		CONNECTED.with(|connected| *connected.borrow_mut() = true);
		let output = Object::new();
		set(&output, "accounts", accounts_array());
		emit_change();
		Ok::<_, String>(output.into())
	});
	set(&connect, "connect", keep(connect_fn));
	set(&features, "standard:connect", connect);

	// standard:disconnect
	let disconnect = Object::new();
	set(&disconnect, "version", js_str("1.0.0"));
	let disconnect_fn: Closure<dyn FnMut() -> Promise> = Closure::new(move || {
		CONNECTED.with(|connected| *connected.borrow_mut() = false);
		emit_change();
		Promise::resolve(&JsValue::UNDEFINED)
	});
	set(&disconnect, "disconnect", keep(disconnect_fn));
	set(&features, "standard:disconnect", disconnect);

	// standard:events
	let events = Object::new();
	set(&events, "version", js_str("1.0.0"));
	let on_fn: Closure<dyn FnMut(String, Function) -> Function> =
		Closure::new(move |event: String, listener: Function| {
			if event == "change" {
				CHANGE_LISTENER.with(|slot| *slot.borrow_mut() = Some(listener));
			}

			// Unsubscribe stub; the dev wallet has exactly one listener slot.
			let off: Closure<dyn FnMut()> = Closure::new(|| {
				CHANGE_LISTENER.with(|slot| *slot.borrow_mut() = None);
			});
			off.into_js_value().unchecked_into::<Function>()
		});
	set(&events, "on", keep(on_fn));
	set(&features, "standard:events", events);

	// solana:signMessage
	let sign_message = Object::new();
	set(&sign_message, "version", js_str("1.0.0"));
	let sign_message_fn = variadic_closure!(|inputs: &[JsValue]| sign_message_outputs(inputs));
	set(&sign_message, "signMessage", keep(sign_message_fn));
	set(&features, "solana:signMessage", sign_message);

	// solana:signTransaction
	let sign_transaction = Object::new();
	set(&sign_transaction, "version", js_str("1.0.0"));
	set(
		&sign_transaction,
		"supportedTransactionVersions",
		Array::from_iter([js_str("legacy"), JsValue::from(0)]),
	);
	let sign_transaction_fn =
		variadic_closure!(|inputs: &[JsValue]| sign_transaction_outputs(inputs));
	set(
		&sign_transaction,
		"signTransaction",
		keep(sign_transaction_fn),
	);
	set(&features, "solana:signTransaction", sign_transaction);

	// solana:signAndSendTransaction
	let sign_and_send = Object::new();
	set(&sign_and_send, "version", js_str("1.0.0"));
	set(
		&sign_and_send,
		"supportedTransactionVersions",
		Array::from_iter([js_str("legacy"), JsValue::from(0)]),
	);
	let sign_and_send_fn = variadic_closure!(|inputs: &[JsValue]| sign_and_send_outputs(inputs));
	set(
		&sign_and_send,
		"signAndSendTransaction",
		keep(sign_and_send_fn),
	);
	set(&features, "solana:signAndSendTransaction", sign_and_send);

	// The wallet object itself.
	let wallet = Object::new();
	set(&wallet, "version", js_str(DEV_WALLET_VERSION));
	set(&wallet, "name", js_str(DEV_WALLET_NAME));
	set(&wallet, "icon", js_str(DEV_WALLET_ICON));
	set(
		&wallet,
		"chains",
		Array::from_iter(CHAINS.iter().copied().map(js_str)),
	);
	set(&wallet, "features", features);
	set(&wallet, "accounts", Array::new());

	register_wallet(wallet.unchecked_ref::<BrowserWalletInfo>())
}

/// Discovers the registered dev wallet and wraps it in the dApp-side
/// `BrowserWallet` handle.
#[must_use]
pub fn discover_dev_wallet() -> Option<BrowserWallet> {
	let info = wallet_standard_browser::get_wallets()
		.get()
		.into_iter()
		.find(|wallet: &BrowserWalletInfo| wallet._name() == DEV_WALLET_NAME)?;
	Some(BrowserWallet::builder().wallet(info).build())
}

/// Connect the dev wallet through the standard connect feature and return the
/// authorized account.
pub async fn connect_dev_wallet(
	wallet: &mut BrowserWallet,
) -> wallet_standard::WalletResult<BrowserWalletAccountInfo> {
	use wallet_standard::WalletStandardConnect;

	let accounts = wallet
		.connect_with_options(wallet_standard::StandardConnectInput::builder().build())
		.await?;
	accounts
		.first()
		.cloned()
		.ok_or(wallet_standard::WalletError::WalletConnection)
}

/// Sign a message through the standard `solana:signMessage` feature and verify
/// the returned signature locally.
pub async fn sign_and_verify_message(
	wallet: &BrowserWallet,
	message: &[u8],
) -> wallet_standard::WalletResult<(Signature, bool)> {
	use wallet_standard::SolanaSignMessageOutput;
	use wallet_standard::SolanaSignatureOutput;
	use wallet_standard::WalletAccountInfo;
	use wallet_standard::WalletSolanaSignMessage;

	let output = wallet.sign_message_async(message.to_vec()).await?;
	let signature = output.try_signature()?;
	let signed_message = output.signed_message();
	let account = wallet
		.wallet_account
		.clone()
		.ok_or(wallet_standard::WalletError::WalletAccount)?;
	let verified = signature.verify(&account.public_key(), &signed_message);
	Ok((signature, verified))
}
