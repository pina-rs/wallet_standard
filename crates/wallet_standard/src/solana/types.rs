use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use crate::WalletError;
use crate::WalletResult;
use crate::WalletSolanaSignAndSendTransaction;
use crate::WalletSolanaSignIn;
use crate::WalletSolanaSignMessage;
use crate::WalletSolanaSignTransaction;
use crate::WalletStandard;

/// Bridges the standard's raw account bytes into a Solana [`Pubkey`].
///
/// Wallet accounts expose their public key as untyped bytes; signing and
/// address display need a real `Pubkey`, and this trait supplies the
/// fallible conversion in one place.
pub trait WalletSolanaPubkey {
	/// In order to prevent clashes with the built in
	/// [`solana_signer::Signer`] this is named differently.
	fn try_solana_pubkey(&self) -> WalletResult<Pubkey>;

	/// In order to prevent clashes with the built in
	/// [`solana_signer::Signer`] this is named differently.
	fn solana_pubkey(&self) -> Pubkey {
		self.try_solana_pubkey().unwrap_or_default()
	}
}

impl WalletSolanaPubkey for Keypair {
	fn try_solana_pubkey(&self) -> WalletResult<Pubkey> {
		let pubkey = Signer::try_pubkey(self)?;
		Ok(pubkey)
	}
}

impl<T> WalletSolanaPubkey for T
where
	T: WalletSolana,
{
	fn try_solana_pubkey(&self) -> WalletResult<Pubkey> {
		self.try_public_key()
			.ok_or(WalletError::WalletNotConnected)
			.and_then(|bytes| Pubkey::try_from(bytes).map_err(|_| WalletError::WalletPublicKey))
	}
}

/// Marker for wallets that support the Solana feature set as a whole.
///
/// Apps can require this single bound to claim connect, sign, and send
/// capabilities together instead of naming each feature trait.
pub trait WalletSolana:
	WalletSolanaSignMessage
	+ WalletSolanaSignTransaction
	+ WalletSolanaSignAndSendTransaction
	+ WalletSolanaSignIn
	+ WalletStandard
{
}

impl<T> WalletSolana for T where
	T: WalletSolanaSignMessage
		+ WalletSolanaSignTransaction
		+ WalletSolanaSignAndSendTransaction
		+ WalletSolanaSignIn
		+ WalletStandard
{
}
