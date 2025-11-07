use crate::WalletStandardConnect;
use crate::WalletStandardDisconnect;

/// Provides information about a wallet implementation.
///
/// This trait defines the metadata and capabilities of a wallet, including its
/// name, icon, supported chains, features, and accounts. It serves as the
/// primary interface for applications to discover wallet capabilities.
///
/// # Example
///
/// ```rust,ignore
/// struct MyWalletInfo {
///     name: String,
///     icon: String,
///     chains: Vec<String>,
///     features: Vec<String>,
///     accounts: Vec<MyAccount>,
/// }
///
/// impl WalletInfo for MyWalletInfo {
///     type Account = MyAccount;
///
///     fn version(&self) -> String {
///         "1.0.0".to_string()
///     }
///
///     fn name(&self) -> String {
///         self.name.clone()
///     }
///
///     // ... other implementations
/// }
/// ```
pub trait WalletInfo {
	type Account: WalletAccountInfo;

	/// {@link `WalletVersion` | Version} of the Wallet Standard implemented by
	/// the Wallet.
	///
	/// Must be read-only, static, and canonically defined by the Wallet
	/// Standard.
	fn version(&self) -> String;
	/// Name of the Wallet. This may be displayed by the app.
	///
	/// Must be read-only, static, descriptive, unique, and canonically defined
	/// by the wallet extension or application.
	fn name(&self) -> String;
	/// {@link `WalletIcon` | Icon} of the Wallet. This may be displayed by the
	/// app.
	///
	/// Must be read-only, static, and canonically defined by the wallet
	/// extension or application.
	fn icon(&self) -> String;
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
	///
	/// # Example
	///
	/// ```
	/// vec!["solana:mainnet".to_string(), "solana:devnet".to_string()]
	/// ```
	fn chains(&self) -> Vec<String>;
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
	///
	/// # Example
	///
	/// ```
	/// vec![
	/// 	"standard:connect".to_string(),
	/// 	"standard:disconnect".to_string(),
	/// 	"solana:signMessage".to_string(),
	/// 	"solana:signTransaction".to_string(),
	/// ]
	/// ```
	fn features(&self) -> Vec<String>;
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
	fn accounts(&self) -> Vec<Self::Account>;
}

/// Interface of a **`WalletAccount`**, also referred to as an **Account**.
///
/// An account is a _read-only data object_ that is provided from the Wallet to
/// the app, authorizing the app to use it.
///
/// The app can use an account to display and query information from a chain.
///
/// The app can also act using an account by passing it to {@link
/// Wallet.features | features} of the Wallet.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Clone)]
/// struct MyAccount {
///     address: String,
///     public_key: Vec<u8>,
///     chains: Vec<String>,
///     features: Vec<String>,
///     label: Option<String>,
/// }
///
/// impl WalletAccountInfo for MyAccount {
///     fn address(&self) -> String {
///         self.address.clone()
///     }
///
///     fn public_key(&self) -> Vec<u8> {
///         self.public_key.clone()
///     }
///
///     // ... other implementations
/// }
/// ```
pub trait WalletAccountInfo {
	/// Address of the account, corresponding with a public key.
	///
	/// This is typically a human-readable string representation of the public
	/// key, formatted according to the blockchain's conventions.
	///
	/// # Example
	///
	/// For Solana: `"HN7cABqLq46Es1jh92dQQisAq662SmxELLLsHHe4YWrH"`
	fn address(&self) -> String;
	/// Public key of the account, corresponding with a secret key to use.
	///
	/// This is the raw binary representation of the public key.
	///
	/// # Example
	///
	/// ```
	/// // A 32-byte Ed25519 public key
	/// vec![0, 1, 2, 3 /* ... */]
	/// ```
	fn public_key(&self) -> Vec<u8>;
	/// Chains supported by the account.
	///
	/// This must be a subset of the {@link Wallet.chains | chains} of the
	/// Wallet.
	///
	/// # Example
	///
	/// ```
	/// vec!["solana:mainnet".to_string()]
	/// ```
	fn chains(&self) -> Vec<String>;
	/// Feature names supported by the account.
	///
	/// This must be a subset of the names of {@link Wallet.features | features}
	/// of the Wallet.
	///
	/// # Example
	///
	/// ```
	/// vec![
	/// 	"solana:signMessage".to_string(),
	/// 	"solana:signTransaction".to_string(),
	/// ]
	/// ```
	fn features(&self) -> Vec<String>;
	/// Optional user-friendly descriptive label or name for the account. This
	/// may be displayed by the app.
	///
	/// # Example
	///
	/// ```
	/// Some("Main Account".to_string())
	/// ```
	fn label(&self) -> Option<String>;
	/// Optional user-friendly icon for the account. This may be displayed by
	/// the app.
	///
	/// The icon should be a data URL containing image data.
	///
	/// # Example
	///
	/// ```
	/// Some("data:image/svg+xml;base64,...".to_string())
	/// ```
	fn icon(&self) -> Option<String>;
}

/// The core trait for wallet implementations.
///
/// This trait provides access to wallet information and the currently connected
/// account. It serves as the foundation for all wallet functionality and is
/// extended by other traits like `WalletStandardConnect` and
/// `WalletStandardDisconnect`.
///
/// # Example
///
/// ```rust,ignore
/// struct MyWallet {
///     wallet_info: MyWalletInfo,
///     current_account: Option<MyAccount>,
/// }
///
/// impl Wallet for MyWallet {
///     type Wallet = MyWalletInfo;
///     type Account = MyAccount;
///
///     fn wallet(&self) -> Self::Wallet {
///         self.wallet_info.clone()
///     }
///
///     fn wallet_account(&self) -> Option<Self::Account> {
///         self.current_account.clone()
///     }
/// }
/// ```
pub trait Wallet {
	type Wallet: WalletInfo;
	type Account: WalletAccountInfo;

	/// Returns the wallet information.
	///
	/// This provides access to metadata about the wallet, such as its name,
	/// icon, supported chains, and features.
	fn wallet(&self) -> Self::Wallet;

	/// Returns the currently connected account, if any.
	///
	/// If no account is connected, this returns `None`.
	fn wallet_account(&self) -> Option<Self::Account>;

	/// Get the wallet's display name.
	///
	/// # Returns
	///
	/// `String` with the wallet's name.
	///
	/// # Examples
	///
	/// ```
	/// // Assuming `w` implements `Wallet`
	/// let name = w.name();
	/// assert!(!name.is_empty());
	/// ```
	fn name(&self) -> String {
		self.wallet().name()
	}

	/// Retrieves the wallet's icon.
	///
	/// The icon is the wallet's canonical, read-only representation (typically a data URL).
	///
	/// # Examples
	///
	/// ```
	/// // `w` is any type implementing the `Wallet` trait.
	/// let icon = w.icon();
	/// assert!(!icon.is_empty());
	/// ```
	fn icon(&self) -> String {
		self.wallet().icon()
	}

	/// Indicates whether the wallet currently has a connected account.
	///
	/// # Returns
	///
	/// `true` if the wallet has a current account, `false` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// // `wallet` implements the `Wallet` trait
	/// let is_connected = wallet.connected();
	/// assert_eq!(is_connected, wallet.wallet_account().is_some());
	/// ```
	fn connected(&self) -> bool {
		self.wallet_account().is_some()
	}

	/// Retrieve the public key of the currently connected account.
	///
	/// # Returns
	///
	/// `Some(Vec<u8>)` with the account's raw public key bytes if an account is connected, `None` if no account is connected.
	///
	/// # Examples
	///
	/// ```
	/// // `wallet` implements the `Wallet` trait.
	/// if let Some(pk) = wallet.try_public_key() {
	///     // `pk` contains the account's public key bytes
	///     assert!(!pk.is_empty());
	/// } else {
	///     // no account is connected
	///     assert!(wallet.wallet_account().is_none());
	/// }
	/// ```
	fn try_public_key(&self) -> Option<Vec<u8>> {
		self.wallet_account().map(|account| account.public_key())
	}

	/// Get the public key of the currently connected account.
	///
	/// # Panics
	///
	/// Panics if no account is connected.
	///
	/// # Examples
	///
	/// ```
	/// // `wallet` must have a connected account for this to succeed.
	/// let pk = wallet.public_key();
	/// assert!(!pk.is_empty());
	/// ```
	fn public_key(&self) -> Vec<u8> {
		self.try_public_key().unwrap()
	}
}

/// A trait that combines the core wallet functionality with standard connect
/// and disconnect features.
///
/// This trait is automatically implemented for any type that implements
/// `Wallet`, `WalletStandardConnect`, and `WalletStandardDisconnect`.
pub trait WalletStandard: WalletStandardConnect + WalletStandardDisconnect + Wallet {}

impl<T> WalletStandard for T where T: WalletStandardConnect + WalletStandardDisconnect + Wallet {}