#![allow(unsafe_code)]

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use js_sys::Array;
use js_sys::Function;
use js_sys::Object;
use js_sys::Reflect;
use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;
use wallet_standard::WalletAccountInfo;
use wallet_standard::WalletError;
use wallet_standard::WalletInfo;
use wallet_standard::WalletResult;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::FeatureFromJs;
use crate::StandardConnectFeature;
use crate::StandardDisconnectFeature;
use crate::StandardEventsFeature;

#[wasm_bindgen(module = "/js/wallet.js")]
extern "C" {
	/// Register a {@link "@wallet-standard/base".Wallet} as a Standard Wallet
	/// with the app.
	///
	/// This dispatches a {@link
	/// "@wallet-standard/base".WindowRegisterWalletEvent} to notify the app
	/// that the Wallet is ready to be registered.
	///
	/// This also adds a listener for {@link
	/// "@wallet-standard/base".WindowAppReadyEvent} to listen for a
	/// notification from the app that the app is ready to register the Wallet.
	///
	/// This combination of event dispatch and listener guarantees that the
	/// Wallet will be registered synchronously as soon as the app is ready
	/// whether the Wallet loads before or after the app.
	///
	/// @param wallet Wallet to register.
	///
	/// @group Wallet
	#[allow(unsafe_code)]
	#[wasm_bindgen(js_name = registerWallet, catch)]
	pub fn register_wallet(wallet: &BrowserWalletInfo) -> Result<(), JsValue>;
}

#[wasm_bindgen(module = "/js/app.js")]
extern "C" {
	#[derive(Clone, Debug)]
	pub type BrowserWalletInfo;
	/// {@link `WalletVersion` | Version} of the Wallet Standard implemented by
	/// the Wallet.
	///
	/// Must be read-only, static, and canonically defined by the Wallet
	/// Standard.
	#[wasm_bindgen(getter, method, js_name = version)]
	pub fn _version(this: &BrowserWalletInfo) -> String;
	/// Name of the Wallet. This may be displayed by the app.
	///
	/// Must be read-only, static, descriptive, unique, and canonically defined
	/// by the wallet extension or application.
	#[wasm_bindgen(getter, method, js_name = name)]
	pub fn _name(this: &BrowserWalletInfo) -> String;
	/// {@link `WalletIcon` | Icon} of the Wallet. This may be displayed by the
	/// app.
	///
	/// Must be read-only, static, and canonically defined by the wallet
	/// extension or application.
	#[wasm_bindgen(getter, method, js_name = icon)]
	pub fn _icon(this: &BrowserWalletInfo) -> String;
	/// Chains supported by the Wallet.
	///
	/// A **chain** is an {@link `IdentifierString`} which identifies a
	/// blockchain in a canonical, human-readable format. [CAIP-2](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md) chain IDs are compatible with this,
	/// but are not required to be used.
	///
	/// Each blockchain should define its own **chains** by extension of the
	/// Wallet Standard, using its own namespace. The `standard` and
	/// `experimental` namespaces are reserved by the Wallet Standard.
	///
	/// The {@link "@wallet-standard/features".EventsFeature | `standard:events`
	/// feature} should be used to notify the app if the value changes.
	#[wasm_bindgen(getter, method, js_name = chains)]
	pub fn _chains(this: &BrowserWalletInfo) -> Vec<String>;
	/// Features supported by the Wallet.
	///
	/// A **feature name** is an {@link `IdentifierString`} which identifies a
	/// **feature** in a canonical, human-readable format.
	///
	/// Each blockchain should define its own features by extension of the
	/// Wallet Standard.
	///
	/// The `standard` and `experimental` namespaces are reserved by the Wallet
	/// Standard.
	///
	/// A **feature** may have any type. It may be a single method or value, or
	/// a collection of them.
	///
	/// A **conventional feature** has the following structure:
	///
	/// ```ts
	///  export type ExperimentalEncryptFeature = {
	///      // Name of the feature.
	///      'experimental:encrypt': {
	///          // Version of the feature.
	///          version: '1.0.0';
	///          // Properties of the feature.
	///          ciphers: readonly 'x25519-xsalsa20-poly1305'[];
	///          // Methods of the feature.
	///          encrypt (data: Uint8Array): Promise<Uint8Array>;
	///      };
	///  };
	/// ```
	///
	/// The {@link "@wallet-standard/features".EventsFeature | `standard:events`
	/// feature} should be used to notify the app if the value changes.
	#[wasm_bindgen(getter, method, js_name = features)]
	pub fn features_object(this: &BrowserWalletInfo) -> Object;
	/// {@link `WalletAccount` | Accounts} that the app is authorized to use.
	///
	/// This can be set by the Wallet so the app can use authorized accounts on
	/// the initial page load.
	///
	/// The {@link "@wallet-standard/features".ConnectFeature |
	/// `standard:connect` feature} should be used to obtain authorization to
	/// the accounts.
	///
	/// The {@link "@wallet-standard/features".EventsFeature | `standard:events`
	/// feature} should be used to notify the app if the value changes.
	#[wasm_bindgen(getter, method, js_name = accounts)]
	pub fn _accounts(this: &BrowserWalletInfo) -> Vec<BrowserWalletAccountInfo>;
	/// Interface of a **`WalletAccount`**, also referred to as an **Account**.
	///
	/// An account is a _read-only data object_ that is provided from the Wallet
	/// to the app, authorizing the app to use it.
	///
	/// The app can use an account to display and query information from a
	/// chain.
	///
	/// The app can also act using an account by passing it to {@link
	/// Wallet.features | features} of the Wallet.
	///
	/// Wallets may use or extend {@link
	/// "@wallet-standard/wallet".ReadonlyWalletAccount} which implements this
	/// interface.
	#[derive(Clone, Debug)]
	pub type BrowserWalletAccountInfo;
	/// Address of the account, corresponding with a public key.
	#[wasm_bindgen(getter, method, js_name = address)]
	pub fn _address(this: &BrowserWalletAccountInfo) -> String;
	/// Public key of the account, corresponding with a secret key to use.
	#[wasm_bindgen(getter, method, js_name = publicKey)]
	pub fn _public_key(this: &BrowserWalletAccountInfo) -> Vec<u8>;
	/// Chains supported by the account.
	///
	/// This must be a subset of the {@link Wallet.chains | chains} of the
	/// Wallet.
	#[wasm_bindgen(getter, method, js_name = chains)]
	pub fn _chains(this: &BrowserWalletAccountInfo) -> Vec<String>;
	/// Feature names supported by the account.
	///
	/// This must be a subset of the names of {@link Wallet.features | features}
	/// of the Wallet.
	#[wasm_bindgen(getter, method, js_name = features)]
	pub fn _features(this: &BrowserWalletAccountInfo) -> Vec<String>;
	/// Optional user-friendly descriptive label or name for the account. This
	/// may be displayed by the app.
	#[wasm_bindgen(getter, method, js_name = label)]
	pub fn _label(this: &BrowserWalletAccountInfo) -> Option<String>;
	/// Optional user-friendly icon for the account. This may be displayed by
	/// the app.
	#[wasm_bindgen(getter, method, js_name = icon)]
	pub fn _icon(this: &BrowserWalletAccountInfo) -> Option<String>;
	#[derive(Clone, Debug)]
	pub type Wallets;
	/// Get all Wallets that have been registered.
	///
	/// @return Registered Wallets.
	#[wasm_bindgen(method)]
	pub fn get(this: &Wallets) -> Vec<BrowserWalletInfo>;
	/// Add an event listener and subscribe to events for Wallets that are
	/// {@link WalletsEventsListeners.register | registered} and
	/// {@link WalletsEventsListeners.unregister | unregistered}.
	///
	/// @param event    Event type to listen for. {@link
	/// WalletsEventsListeners.register | `register`} and
	/// {@link WalletsEventsListeners.unregister | `unregister`} are the only
	/// event types. @param listener Function that will be called when an event
	/// of the type is emitted.
	///
	/// @return
	/// `off` function which may be called to remove the event listener and
	/// unsubscribe from events.
	///
	/// As with all event listeners, be careful to avoid memory leaks.
	#[wasm_bindgen(method)]
	pub fn on(
		this: &Wallets,
		event_name: &str,
		callback: &Closure<dyn Fn(BrowserWalletInfo)>,
	) -> Function;
	/// Register Wallets. This can be used to programmatically wrap non-standard
	/// wallets as Standard Wallets.
	///
	/// Apps generally do not need to, and should not, call this.
	///
	/// @param wallets Wallets to register.
	///
	/// @return
	/// `unregister` function which may be called to programmatically unregister
	/// the registered Wallets.
	///
	/// Apps generally do not need to, and should not, call this.
	#[wasm_bindgen(method, js_name = register, getter)]
	pub fn register_fn(this: &Wallets) -> Function;

	#[wasm_bindgen(js_name = getWallets)]
	pub fn get_wallets() -> Wallets;
}

impl PartialEq for BrowserWalletInfo {
	fn eq(&self, other: &Self) -> bool {
		self.name().eq(&other.name())
			&& self.chains().eq(&other.chains())
			&& self.version().eq(&other.version())
			&& self.icon().eq(&other.icon())
	}
}

impl Eq for BrowserWalletInfo {}

impl Hash for BrowserWalletInfo {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name().hash(state);
		self.chains().hash(state);
		self.version().hash(state);
		self.icon().hash(state);
	}
}

impl BrowserWalletInfo {
	pub fn get_hash(&self) -> u64 {
		let mut hasher = DefaultHasher::new();
		self.name().hash(&mut hasher);
		self.chains().hash(&mut hasher);

		hasher.finish()
	}

	/// Get the feature from the provide type. Must implement `FeatureFromJs`.
	pub fn get_feature_option<T: FeatureFromJs>(&self) -> Option<T> {
		T::feature_from_js_object(&self.features_object())
	}

	/// Get the required feature and throw an error if it isn't supported.
	pub fn get_feature<T: FeatureFromJs>(&self) -> WalletResult<T> {
		self.get_feature_option::<T>()
			.ok_or(WalletError::UnsupportedFeature {
				feature: T::NAME.to_string(),
				wallet: self.name(),
			})
	}

	/// Check whether a feature is supported by the given wallet.
	pub fn is_feature_supported<T: FeatureFromJs>(&self) -> bool {
		self.get_feature_option::<T>().is_some()
	}

	/// Determines whether the wallet supports the `StandardConnect`, `StandardEvents`, and `StandardDisconnect` features.
	///
	/// # Examples
	///
	/// ```
	/// // `info` is a BrowserWalletInfo (or type implementing the same API)
	/// // that may be obtained from the JS bindings.
	/// let compatible = info.is_standard_compatible();
	/// // `compatible` is true only when connect, events and disconnect features are all present.
	/// ```
	///
	/// # Returns
	///
	/// `true` if the wallet supports all three Standard features (`StandardConnectFeature`, `StandardEventsFeature`, `StandardDisconnectFeature`), `false` otherwise.
	pub fn is_standard_compatible(&self) -> bool {
		self.is_feature_supported::<StandardConnectFeature>()
			&& self.is_feature_supported::<StandardEventsFeature>()
			&& self.is_feature_supported::<StandardDisconnectFeature>()
	}
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BrowserWalletInfoFeatures(#[serde(with = "serde_wasm_bindgen::preserve")] Object);

impl BrowserWalletInfoFeatures {
	/// Inserts a feature into the internal JS features object under the feature's canonical name.
	///
	/// This stores the feature's JS representation at the key `T::NAME` inside the wrapped `Object`.
	///
	/// # Panics
	///
	/// Panics if setting the property on the underlying JS object fails.
	pub fn add_feature<T: FeatureFromJs>(&self, feature: &T) {
		Reflect::set(&self.0, &JsValue::from_str(T::NAME), feature.as_ref()).unwrap();
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct BrowserWalletInfoProps {
	#[builder(setter(into))]
	pub name: String,
	#[builder(default)]
	pub chains: Vec<String>,
	#[builder(setter(into))]
	pub version: String,
	#[builder(setter(into))]
	pub icon: String,
	#[builder(default)]
	pub features: BrowserWalletInfoFeatures,
	#[builder(default)]
	pub accounts: Vec<BrowserWalletAccountInfoProps>,
}

impl WalletInfo for BrowserWalletInfo {
	type Account = BrowserWalletAccountInfo;

	fn version(&self) -> String {
		self._version()
	}

	fn name(&self) -> String {
		self._name()
	}

	fn icon(&self) -> String {
		self._icon()
	}

	fn chains(&self) -> Vec<String> {
		self._chains()
	}

	fn features(&self) -> Vec<String> {
		Object::keys(&self.features_object())
			.into_iter()
			.map(|value| value.as_string().unwrap_throw())
			.collect::<Vec<_>>()
	}

	fn accounts(&self) -> Vec<Self::Account> {
		self._accounts()
	}
}

impl PartialEq for BrowserWalletAccountInfo {
	fn eq(&self, other: &Self) -> bool {
		self.address().eq(&other.address())
			&& self.public_key().eq(&other.public_key())
			&& self.chains().eq(&other.chains())
			&& self.features().eq(&other.features())
			&& self.label().eq(&other.label())
			&& self.icon().eq(&other.icon())
	}
}

impl Eq for BrowserWalletAccountInfo {}

impl Hash for BrowserWalletAccountInfo {
	/// Hashes the account's identifying fields into the provided hasher.
	///
	/// Inserts the account's address, public key, chains, features, label, and icon
	/// into `state` in that order to produce a combined hash for the account.
	///
	/// # Examples
	///
	/// ```
	/// use std::hash::{Hash, Hasher};
	/// use std::collections::hash_map::DefaultHasher;
	///
	/// let mut hasher = DefaultHasher::new();
	/// // Hash a tuple with the same field order used by the account implementation.
	/// ("addr", vec![1u8], vec!["chain".to_string()], vec!["feat".to_string()], Some("lbl"), Some("ico"))
	///     .hash(&mut hasher);
	/// let _digest = hasher.finish();
	/// ```
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.address().hash(state);
		self.public_key().hash(state);
		self.chains().hash(state);
		self.features().hash(state);
		self.label().hash(state);
		self.icon().hash(state);
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct BrowserWalletAccountInfoProps {
	#[builder(setter(into))]
	pub address: String,
	#[builder(setter(into))]
	pub public_key: Vec<u8>,
	#[builder(default)]
	pub chains: Vec<String>,
	#[builder(default)]
	pub features: Vec<String>,
	#[builder(default)]
	pub label: Option<String>,
	#[builder(default)]
	pub icon: Option<String>,
}

impl BrowserWalletAccountInfo {
	/// Constructs a `BrowserWalletAccountInfo` from the given `BrowserWalletAccountInfoProps`.
	///
	/// On success returns a `BrowserWalletAccountInfo` that can be passed to JS interop APIs.
	///
	/// # Errors
	///
	/// Returns a `WalletError` if serialization of `props` to a JS value fails or if the resulting
	/// JS value cannot be converted into a `BrowserWalletAccountInfo`.
	///
	/// # Examples
	///
	/// ```
	/// let props = BrowserWalletAccountInfoProps::builder()
	///     .address("addr://example".to_string())
	///     .public_key(vec![])
	///     .build();
	///
	/// let account = BrowserWalletAccountInfo::try_new(&props).unwrap();
	/// assert_eq!(account.address(), "addr://example");
	/// ```
	pub fn try_new(props: &BrowserWalletAccountInfoProps) -> WalletResult<Self> {
		let result = serde_wasm_bindgen::to_value(props)?.dyn_into::<BrowserWalletAccountInfo>()?;

		Ok(result)
	}
}

impl WalletAccountInfo for BrowserWalletAccountInfo {
	/// Gets the account address.
	///
	/// # Returns
	///
	/// `String` containing the account's address.
	///
	/// # Examples
	///
	/// ```
	/// // given a BrowserWalletAccountInfo `acct` obtained from JS bindings:
	/// let addr = acct.address();
	/// assert!(!addr.is_empty());
	/// ```
	fn address(&self) -> String {
		self._address()
	}

	fn public_key(&self) -> Vec<u8> {
		self._public_key()
	}

	fn chains(&self) -> Vec<String> {
		self._chains()
	}

	fn features(&self) -> Vec<String> {
		self._features()
	}

	fn label(&self) -> Option<String> {
		self._label()
	}

	fn icon(&self) -> Option<String> {
		self._icon()
	}
}

impl Wallets {
	/// Currently only supports one wallet at a time.
	/// <https://github.com/rustwasm/wasm-bindgen/issues/3715>
	pub fn on_register(&self, callback: &Closure<dyn Fn(BrowserWalletInfo)>) -> Box<dyn Fn()> {
		let dispose = self.on("register", callback);

		Box::new(move || {
			let _ = dispose.call0(&JsValue::NULL);
		})
	}

	/// Currently only supports one wallet at a time.
	/// <https://github.com/rustwasm/wasm-bindgen/issues/3715>
	pub fn on_unregister(&self, callback: &Closure<dyn Fn(BrowserWalletInfo)>) -> Box<dyn Fn()> {
		let dispose = self.on("unregister", callback);

		Box::new(move || {
			let _ = dispose.call0(&JsValue::NULL);
		})
	}

	pub fn register(&self, wallets: &[BrowserWalletInfo]) -> Box<dyn Fn()> {
		let args = Array::new();

		for wallet in wallets {
			args.push(wallet.unchecked_ref());
		}

		let dispose: Function = self
			.register_fn()
			.apply(self.unchecked_ref(), &args)
			.unwrap()
			.dyn_into()
			.unwrap();

		Box::new(move || {
			let _ = dispose.call0(&JsValue::NULL);
		})
	}
}