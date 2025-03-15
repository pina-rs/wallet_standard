use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use typed_builder::TypedBuilder;

use crate::Wallet;
use crate::WalletAccountInfo;
use crate::WalletResult;

/// Feature identifier for the standard connect feature.
///
/// This constant is used to identify the connect feature in the wallet's
/// feature list.
pub const STANDARD_CONNECT: &str = "standard:connect";

/// Output of a successful wallet connection.
///
/// This trait defines the structure of the data returned when a wallet
/// connection is established. It provides access to the accounts that the app
/// has been authorized to use.
///
/// # Example Implementation
///
/// ```rust,ignore
/// struct MyConnectOutput {
///     accounts: Vec<MyAccount>,
/// }
///
/// impl StandardConnectOutput for MyConnectOutput {
///     type Account = MyAccount;
///
///     fn accounts(&self) -> Vec<Self::Account> {
///         self.accounts.clone()
///     }
/// }
/// ```
pub trait StandardConnectOutput {
	type Account: WalletAccountInfo;

	/// Returns the list of accounts that the app has been authorized to use.
	///
	/// These accounts can be used for operations like signing messages or
	/// transactions.
	fn accounts(&self) -> Vec<Self::Account>;
}

/// Input options for connecting to a wallet.
///
/// This struct provides configuration options for the wallet connection
/// process. It allows apps to specify whether the connection should be silent
/// (without user prompts) or interactive.
///
/// # Example
///
/// ```rust
/// use wallet_standard::StandardConnectInput;
///
/// // Create a silent connection request
/// let silent_connect = StandardConnectInput::builder().silent(true).build();
///
/// // Create a default connection request (interactive)
/// let default_connect = StandardConnectInput::default();
/// ```
#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "browser", wasm_bindgen::prelude::wasm_bindgen)]
pub struct StandardConnectInput {
	/// By default, using the {@link `StandardConnectFeature`} should prompt the
	/// user to request authorization to accounts. Set the `silent` flag to
	/// `true` to request accounts that have already been authorized without
	/// prompting.
	///
	/// This flag may or may not be used by the Wallet and the app should not
	/// depend on it being used. If this flag is used by the Wallet, the Wallet
	/// should not prompt the user, and should return only the accounts that the
	/// app is authorized to use.
	///
	/// If set to `true`, the wallet should not display any UI and should only
	/// return previously authorized accounts.
	///
	/// If set to `false` or not provided, the wallet may display UI to prompt
	/// the user to authorize accounts.
	#[builder(default, setter(into, strip_option))]
	silent: Option<bool>,
}

/// Trait for wallets that support connecting to authorize accounts.
///
/// This trait defines methods for connecting to a wallet and authorizing
/// accounts for use by the app. It provides both a simple connect method and a
/// method that accepts connection options.
///
/// # Example Implementation
///
/// ```rust,ignore
/// #[async_trait(?Send)]
/// impl WalletStandardConnect for MyWallet {
///     async fn connect(&mut self) -> WalletResult<Vec<Self::Account>> {
///         // Prompt the user to select accounts
///         let selected_accounts = prompt_user_for_accounts();
///
///         // Update the wallet's state
///         self.current_account = selected_accounts.first().cloned();
///
///         Ok(selected_accounts)
///     }
///
///     async fn connect_with_options(
///         &mut self,
///         options: StandardConnectInput,
///     ) -> WalletResult<Vec<Self::Account>> {
///         if options.silent.unwrap_or(false) {
///             // Return previously authorized accounts without prompting
///             Ok(self.authorized_accounts.clone())
///         } else {
///             // Prompt the user
///             self.connect().await
///         }
///     }
/// }
/// ```
#[async_trait(?Send)]
pub trait WalletStandardConnect: Wallet {
	/// Connect to the wallet and authorize accounts.
	///
	/// This method prompts the user to authorize accounts for use by the app.
	/// It returns a list of authorized accounts and updates the wallet's state
	/// to reflect the connection.
	///
	/// # Returns
	///
	/// A `WalletResult` containing a vector of authorized accounts if
	/// successful, or a `WalletError` if the connection fails.
	///
	/// # Errors
	///
	/// This method may return errors such as:
	/// - `WalletError::WalletConnection` if the connection fails
	/// - `WalletError::WalletWindowClosed` if the user closes the wallet window
	/// - `WalletError::WalletWindowBlocked` if the wallet window is blocked
	async fn connect(&mut self) -> WalletResult<Vec<Self::Account>>;

	/// Connect to the wallet with specific options.
	///
	/// This method allows more control over the connection process through the
	/// provided options. It can be used to request a silent connection without
	/// user prompts.
	///
	/// # Parameters
	///
	/// * `options` - Connection options that control the behavior of the
	///   connection process
	///
	/// # Returns
	///
	/// A `WalletResult` containing a vector of authorized accounts if
	/// successful, or a `WalletError` if the connection fails.
	///
	/// # Errors
	///
	/// This method may return errors such as:
	/// - `WalletError::WalletConnection` if the connection fails
	/// - `WalletError::WalletWindowClosed` if the user closes the wallet window
	/// - `WalletError::WalletWindowBlocked` if the wallet window is blocked
	async fn connect_with_options(
		&mut self,
		options: StandardConnectInput,
	) -> WalletResult<Vec<Self::Account>>;
}
