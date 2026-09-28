import { defineConfig } from "@playwright/test";

const root = new URL(".", import.meta.url).pathname;

export default defineConfig({
	testDir: "./tests",
	timeout: 120_000,
	expect: { timeout: 30_000 },
	fullyParallel: false,
	workers: 1,
	retries: process.env.CI ? 1 : 0,
	reporter: [["list"], ["html", { open: "never" }]],
	use: {
		headless: true,
		// CI runners disable the user namespaces Chromium's sandbox needs;
		// the tests only ever talk to localhost services.
		chromiumSandbox: false,
		trace: "retain-on-failure",
	},
	webServer: [
		{
			command: "node scripts/surfpool-server.mjs",
			url: "http://127.0.0.1:8898/health",
			reuseExistingServer: true,
			cwd: root,
			timeout: 120_000,
		},
		{
			command: "trunk serve --port 3021",
			url: "http://127.0.0.1:3021",
			reuseExistingServer: true,
			cwd: `${root}../leptos-wallet`,
			timeout: 300_000,
		},
		{
			command: "trunk serve --port 3022",
			url: "http://127.0.0.1:3022",
			reuseExistingServer: true,
			cwd: `${root}../dioxus-wallet`,
			timeout: 300_000,
		},
	],
});
