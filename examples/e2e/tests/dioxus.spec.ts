import { test } from "@playwright/test";
import { runDisconnectFlow, runWalletFlow } from "./helpers";

test.describe("dioxus 0.7 + wallet_standard_browser + surfpool", () => {
	test("full wallet standard flow against surfpool", async ({ page }) => {
		await page.goto("http://127.0.0.1:3022/");
		await runWalletFlow(page);
	});

	test("connect, disconnect and reconnect", async ({ page }) => {
		await page.goto("http://127.0.0.1:3022/");
		await runDisconnectFlow(page);
	});
});
