#!/usr/bin/env bash
# Gates the release size of the browser crate's wasm surface.
#
# The wasm-size probe example links the public API into one binary; its
# byte size is the deterministic budget for code growth. Together with the
# boundary-crossing tests in the browser test suite, this is the crate's
# performance regression gate: timings flake in CI, counted bytes do not.
#
# Usage:
#   setup/perf/wasm-size-gate.sh           # fail when over budget
#   setup/perf/wasm-size-gate.sh --update  # write the current size as the budget
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

wasm="target/wasm32-unknown-unknown/release/examples/wasm_size_probe.wasm"
budget_file="setup/perf/wasm-size-budget"

cargo build --locked --release --target wasm32-unknown-unknown \
	--package wallet_standard_browser --features solana \
	--example wasm_size_probe >&2

size="$(wc -c <"$wasm" | tr -d '[:space:]')"

if [ "${1:-}" = "--update" ]; then
	printf '%s\n' "$size" >"$budget_file"
	echo "wasm size budget updated to $size bytes"
	exit 0
fi

budget="$(grep -m1 -Eo '^[0-9]+' "$budget_file")"

echo "wasm probe size: $size bytes (budget: $budget bytes)"

if [ "$size" -gt "$budget" ]; then
	echo "error: the wasm probe grew past its budget ($size > $budget bytes)." >&2
	echo "" >&2
	echo "If the growth is intentional, rerun 'setup/perf/wasm-size-gate.sh --update'" >&2
	echo "and commit the new budget together with the change that caused it." >&2
	exit 1
fi
