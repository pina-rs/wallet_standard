import { expect, type Page } from "@playwright/test";

/** Deterministic address of the embedded Surfpool Dev Wallet keypair. */
export const DEV_ADDRESS = "4acVT7jfykgHZNunZYEwg1NNCUnvuXwFH292ebEGnN4g";

/**
 * Drives one full Wallet Standard round-trip against the surfpool backend:
 *
 *  1. the Rust-registered dev wallet is discovered on the page,
 *  2. `standard:connect` authorizes the deterministic account,
 *  3. `requestAirdrop` funds it over JSON-RPC,
 *  4. `solana:signMessage` produces a signature that the app verifies locally,
 *  5. `solana:signTransaction` + `sendTransaction` (dApp broadcasts),
 *  6. `solana:signAndSendTransaction` (wallet broadcasts).
 */
export async function runWalletFlow(page: Page) {
	await expect(page.getByTestId("wallet-status")).toContainText(
		"detected: Surfpool Dev Wallet",
		{ timeout: 60_000 },
	);

	await page.getByTestId("connect").click();
	await expect(page.getByTestId("account-address")).toHaveText(DEV_ADDRESS);

	await page.getByTestId("airdrop").click();
	await expect(page.getByTestId("airdrop-status")).toContainText("confirmed", {
		timeout: 60_000,
	});
	await expect(page.getByTestId("balance-value")).toContainText(/\d+ lamports/);

	await page.getByTestId("sign-message").click();
	const signStatus = page.getByTestId("sign-message-status");
	await expect(signStatus).toContainText("signature", { timeout: 30_000 });
	await expect(signStatus).toContainText("VALID");

	await page.getByTestId("send-app").click();
	await expect(page.getByTestId("send-app-status")).toContainText("confirmed", {
		timeout: 60_000,
	});

	await page.getByTestId("send-wallet").click();
	await expect(page.getByTestId("send-wallet-status")).toContainText(
		"confirmed",
		{
			timeout: 60_000,
		},
	);
}

/** Connect then disconnect through the standard features. */
export async function runDisconnectFlow(page: Page) {
	await expect(page.getByTestId("wallet-status")).toContainText(
		"detected: Surfpool Dev Wallet",
		{ timeout: 60_000 },
	);
	await page.getByTestId("connect").click();
	await expect(page.getByTestId("account-address")).toHaveText(DEV_ADDRESS);

	await page.getByTestId("disconnect").click();
	await expect(page.getByTestId("account-address")).toHaveText("");
	await expect(page.getByTestId("log")).toContainText("disconnected");

	// connecting again works after a disconnect
	await page.getByTestId("connect").click();
	await expect(page.getByTestId("account-address")).toHaveText(DEV_ADDRESS);
}
