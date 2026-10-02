import { defineConfig } from "@playwright/test";

const root = new URL(".", import.meta.url).pathname;

// One wait covers a test; the other covers the trunk build the first server
// start triggers. Local runs are warm after the first suite.
const TEST_TIMEOUT_MS = 120_000;
const EXPECT_TIMEOUT_MS = 30_000;
const DEV_SERVER_TIMEOUT_MS = 300_000;

export default defineConfig({
	testDir: "./tests",
	timeout: TEST_TIMEOUT_MS,
	expect: { timeout: EXPECT_TIMEOUT_MS },
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
			timeout: TEST_TIMEOUT_MS,
		},
		{
			command: "trunk serve --port 3021",
			url: "http://127.0.0.1:3021",
			reuseExistingServer: true,
			cwd: `${root}../leptos-wallet`,
			timeout: DEV_SERVER_TIMEOUT_MS,
		},
		{
			command: "trunk serve --port 3022",
			url: "http://127.0.0.1:3022",
			reuseExistingServer: true,
			cwd: `${root}../dioxus-wallet`,
			timeout: DEV_SERVER_TIMEOUT_MS,
		},
	],
});
