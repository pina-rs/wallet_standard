#![allow(clippy::unused_async)]

use js_sys::Array;
use js_sys::Object;
use js_sys::Reflect;
use wallet_standard_browser::BrowserWallet;
use wallet_standard_browser::BrowserWalletInfo;
use wallet_standard_browser::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::CustomEvent;
use web_sys::CustomEventInit;
use web_sys::window;

// Configure wasm_bindgen_test to run in a browser environment
wasm_bindgen_test_configure!(run_in_browser);

struct TestWallet {
	name: String,
	version: String,
	icon: String,
	chains: Vec<String>,
	features: Vec<String>,
}

// Helper function to create a mock wallet for testing
pub fn create_mock_wallet() -> JsValue {
	// Create a mock wallet object
	let wallet = Object::new();

	// Set basic wallet properties
	Reflect::set(
		&wallet,
		&JsValue::from_str("name"),
		&JsValue::from_str("MockWallet"),
	)
	.unwrap();
	Reflect::set(
		&wallet,
		&JsValue::from_str("version"),
		&JsValue::from_str("1.0.0"),
	)
	.unwrap();
	Reflect::set(&wallet, &JsValue::from_str("icon"), &JsValue::from_str("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIzMiIgaGVpZ2h0PSIzMiIgdmlld0JveD0iMCAwIDMyIDMyIiBmaWxsPSJub25lIj48cmVjdCB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHJ4PSIxNiIgZmlsbD0iIzQxNDE0MSIvPjxwYXRoIGZpbGwtcnVsZT0iZXZlbm9kZCIgY2xpcC1ydWxlPSJldmVub2RkIiBkPSJNMTcuNDI0IDEwLjQ2OEwxNi41OTEgMTMuMDA2TDE5LjIxMSAxMy4wMDZMMTcuNDI0IDEwLjQ2OFpNMTMuNTM0IDEwLjQ2OEwxMi43MDEgMTMuMDA2TDE1LjMyMSAxMy4wMDZMMTMuNTM0IDEwLjQ2OFpNMjEuNjY3IDE3LjE1OUgyMy4xNjdWMTUuNjU5SDIxLjY2N1YxNy4xNTlaTTIxLjY2NyAyMC4xNTlIMjMuMTY3VjE4LjY1OUgyMS42NjdWMjAuMTU5Wk04LjgzNCAxNy4xNTlIMTAuMzM0VjE1LjY1OUg4LjgzNFYxNy4xNTlaTTguODM0IDIwLjE1OUgxMC4zMzRWMTguNjU5SDguODM0VjIwLjE1OVoiIGZpbGw9IndoaXRlIi8+PHBhdGggZD0iTTIzLjE2NyAyMS42NTlIMjEuNjY3VjIzLjE1OUgyMy4xNjdWMjEuNjU5WiIgZmlsbD0id2hpdGUiLz48cGF0aCBkPSJNMTAuMzM0IDIxLjY1OUg4LjgzNFYyMy4xNTlIMTAuMzM0VjIxLjY1OVoiIGZpbGw9IndoaXRlIi8+PC9zdmc+")).unwrap();

	// Set chains
	let chains = Array::new();
	chains.push(&JsValue::from_str("solana:mainnet"));
	chains.push(&JsValue::from_str("solana:devnet"));
	Reflect::set(&wallet, &JsValue::from_str("chains"), &chains).unwrap();

	// Set features
	let features = Object::new();

	// Add standard:connect feature
	let connect_feature = Object::new();
	Reflect::set(
		&connect_feature,
		&JsValue::from_str("version"),
		&JsValue::from_str("1.0.0"),
	)
	.unwrap();
	Reflect::set(
		&features,
		&JsValue::from_str("standard:connect"),
		&connect_feature,
	)
	.unwrap();

	// Add standard:disconnect feature
	let disconnect_feature = Object::new();
	Reflect::set(
		&disconnect_feature,
		&JsValue::from_str("version"),
		&JsValue::from_str("1.0.0"),
	)
	.unwrap();
	Reflect::set(
		&features,
		&JsValue::from_str("standard:disconnect"),
		&disconnect_feature,
	)
	.unwrap();

	// Add solana:signMessage feature
	let sign_message_feature = Object::new();
	Reflect::set(
		&sign_message_feature,
		&JsValue::from_str("version"),
		&JsValue::from_str("1.0.0"),
	)
	.unwrap();
	Reflect::set(
		&features,
		&JsValue::from_str("solana:signMessage"),
		&sign_message_feature,
	)
	.unwrap();

	Reflect::set(&wallet, &JsValue::from_str("features"), &features).unwrap();

	// Set accounts
	let accounts = Array::new();
	Reflect::set(&wallet, &JsValue::from_str("accounts"), &accounts).unwrap();

	wallet.into()
}

// Helper function to register a mock wallet
pub fn register_mock_wallet() -> Result<(), JsValue> {
	let window = window().expect("no global window exists");
	let wallet = create_mock_wallet();

	// Create a custom event to register the wallet
	let event_init = CustomEventInit::new();
	event_init.set_detail(&wallet);

	let event =
		CustomEvent::new_with_event_init_dict("wallet-standard:register-wallet", &event_init)?;

	window.dispatch_event(&event)?;

	Ok(())
}

#[wasm_bindgen_test]
pub async fn test_wallet_creation() {
	// Create a mock wallet
	let wallet_js = create_mock_wallet();

	// Convert to BrowserWalletInfo
	let wallet_info: BrowserWalletInfo = wallet_js.unchecked_into();

	// Verify wallet properties
	assert_eq!(wallet_info.name(), "MockWallet");
	assert_eq!(wallet_info.version(), "1.0.0");
	assert!(wallet_info.icon().starts_with("data:image/svg+xml;base64,"));

	// Verify chains
	let chains = wallet_info.chains();
	assert_eq!(chains.len(), 2);
	assert_eq!(chains[0], "solana:mainnet");
	assert_eq!(chains[1], "solana:devnet");

	// Verify features
	let features = wallet_info.features();
	assert_eq!(features.len(), 3);
	assert!(features.contains(&"standard:connect".to_string()));
	assert!(features.contains(&"standard:disconnect".to_string()));
	assert!(features.contains(&"solana:signMessage".to_string()));

	// Create a BrowserWallet from the wallet info
	let wallet = BrowserWallet::from(wallet_info);

	// Verify wallet properties
	assert_eq!(wallet.name(), "MockWallet");
	assert_eq!(wallet.wallet().version(), "1.0.0");
	assert!(!wallet.connected());
}

#[wasm_bindgen_test]
pub async fn test_get_wallets() {
	// Register a mock wallet
	register_mock_wallet().expect("Failed to register mock wallet");

	// Get all wallets
	let wallets = wallet_standard_browser::get_wallets();

	// Verify that we can get the wallets
	let wallet_list = wallets.get();

	// There should be at least one wallet (our mock wallet)
	assert!(!wallet_list.is_empty());

	// Find our mock wallet
	let mock_wallet = wallet_list
		.iter()
		.find(|w| w.name() == "MockWallet")
		.expect("Mock wallet not found");

	// Verify wallet properties
	assert_eq!(mock_wallet.name(), "MockWallet");
	assert_eq!(mock_wallet.version(), "1.0.0");

	// Create a BrowserWallet from the wallet info
	let wallet = BrowserWallet::from(mock_wallet.clone());

	// Verify wallet properties
	assert_eq!(wallet.name(), "MockWallet");
	assert!(!wallet.connected());
}

#[wasm_bindgen_test]
pub async fn test_wallet_features() {
	// Register a mock wallet
	register_mock_wallet().expect("Failed to register mock wallet");

	// Get all wallets and store in a variable to extend its lifetime
	let wallets = wallet_standard_browser::get_wallets();
	let wallet_list = wallets.get();

	// Find our mock wallet
	let mock_wallet = wallet_list
		.iter()
		.find(|w| w.name() == "MockWallet")
		.expect("Mock wallet not found");

	// Check if the wallet supports standard features
	assert!(mock_wallet.is_standard_compatible());

	// Check specific features
	assert!(mock_wallet.is_feature_supported::<wallet_standard_browser::StandardConnectFeature>());
	assert!(
		mock_wallet.is_feature_supported::<wallet_standard_browser::StandardDisconnectFeature>()
	);

	// If solana feature is enabled, check Solana-specific features
	#[cfg(feature = "solana")]
	{
		assert!(
			mock_wallet.is_feature_supported::<wallet_standard_browser::SolanaSignMessageFeature>()
		);
	}
}
