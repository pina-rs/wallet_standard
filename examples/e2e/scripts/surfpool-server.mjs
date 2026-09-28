#!/usr/bin/env node
/**
 * Playwright `webServer` wrapper around a surfpool instance.
 *
 * Surfpool's JSON-RPC endpoint only answers `POST /`, so it cannot be used
 * directly as a Playwright `url` health check. This wrapper:
 *
 *  1. reuses an already-healthy surfpool on RPC_PORT (handy during
 *     development), otherwise spawns `surfpool start --offline`;
 *  2. exposes `GET /health` on HEALTH_PORT once surfpool answers
 *     `getVersion`, which is the URL Playwright polls;
 *  3. tears surfpool down when the wrapper process exits.
 */
import { spawn } from "node:child_process";
import { createServer } from "node:http";

const RPC_PORT = Number(process.env.SURFPOOL_RPC_PORT ?? 8899);
const WS_PORT = Number(process.env.SURFPOOL_WS_PORT ?? 8900);
const HEALTH_PORT = Number(process.env.SURFPOOL_HEALTH_PORT ?? 8898);
const RPC_URL = `http://127.0.0.1:${RPC_PORT}`;

let surfpool = null;

async function jsonRpc(method) {
	const response = await fetch(RPC_URL, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify({ jsonrpc: "2.0", id: 1, method }),
	});
	return response.json();
}

async function healthy() {
	try {
		const body = await jsonRpc("getVersion");
		return body?.result !== undefined;
	} catch {
		return false;
	}
}

async function waitForHealth(timeoutMs = 60_000) {
	const deadline = Date.now() + timeoutMs;
	while (Date.now() < deadline) {
		// eslint-disable-next-line no-await-in-loop
		if (await healthy()) return true;
		// eslint-disable-next-line no-await-in-loop
		await new Promise((resolve) => setTimeout(resolve, 300));
	}
	return false;
}

function shutdown() {
	if (surfpool !== null) {
		surfpool.kill("SIGTERM");
		surfpool = null;
	}
	process.exit(0);
}

process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);

if (await healthy()) {
	console.log(`[surfpool-server] reusing existing surfpool at ${RPC_URL}`);
} else {
	console.log(
		`[surfpool-server] starting surfpool (rpc :${RPC_PORT}, ws :${WS_PORT})`,
	);
	surfpool = spawn(
		"surfpool",
		[
			"start",
			"--offline",
			"--port",
			String(RPC_PORT),
			"--ws-port",
			String(WS_PORT),
		],
		{ stdio: ["ignore", "inherit", "inherit"] },
	);
	surfpool.on("exit", (code) => {
		console.error(`[surfpool-server] surfpool exited with code ${code}`);
		process.exit(1);
	});
}

if (!(await waitForHealth())) {
	console.error("[surfpool-server] surfpool did not become healthy in time");
	shutdown();
}

createServer((request, response) => {
	if (request.url === "/health") {
		response.writeHead(200, { "Content-Type": "application/json" });
		response.end(JSON.stringify({ ok: true, rpc: RPC_URL }));
		return;
	}
	response.writeHead(404);
	response.end();
}).listen(HEALTH_PORT, "127.0.0.1", () => {
	console.log(`[surfpool-server] health endpoint ready on :${HEALTH_PORT}`);
});
