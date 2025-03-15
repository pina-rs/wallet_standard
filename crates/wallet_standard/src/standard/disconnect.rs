use async_trait::async_trait;

use crate::Wallet;
use crate::WalletResult;

/// Feature identifier for the standard disconnect feature.
///
/// This constant is used to identify the disconnect feature in the wallet's
/// feature list. Wallets that implement the `WalletStandardDisconnect` trait
/// should include this identifier in their feature list.
pub const STANDARD_DISCONNECT: &str = "standard:disconnect";

/// Trait for wallets that support disconnecting.
///
/// This trait defines a method for disconnecting from a wallet, which revokes
/// the app's authorization to use the wallet's accounts. After disconnecting,
/// the app will need to connect again to use the wallet.
///
/// # Example Implementation
///
/// ```rust,ignore
/// #[async_trait(?Send)]
/// impl WalletStandardDisconnect for MyWallet {
///     async fn disconnect(&mut self) -> WalletResult<()> {
///         // Clear the current account
///         self.current_account = None;
///
///         // Optionally, perform any cleanup or notify the wallet's backend
///         self.notify_backend_of_disconnect().await?;
///
///         Ok(())
///     }
/// }
/// ```
#[async_trait(?Send)]
pub trait WalletStandardDisconnect: Wallet {
	/// Disconnect from the wallet.
	///
	/// This method revokes the app's authorization to use the wallet's
	/// accounts. It should clear the wallet's current account and perform any
	/// necessary cleanup.
	///
	/// # Returns
	///
	/// A `WalletResult` containing `()` if the disconnection is successful,
	/// or a `WalletError` if the disconnection fails.
	///
	/// # Errors
	///
	/// This method may return errors such as:
	/// - `WalletError::WalletDisconnection` if the disconnection fails
	/// - `WalletError::WalletDisconnected` if the wallet is already
	///   disconnected
	async fn disconnect(&mut self) -> WalletResult<()>;
}
