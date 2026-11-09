#!/usr/bin/env bash
# Bootstrap a Linux cloud-agent VM so it can build, lint, and test this repo.
#
# The project toolchain is the pinned devenv shell (see devenv.nix and
# devenv.lock): Rust from rust-toolchain.toml, cargo binaries from
# workspace.metadata.bin, and the lint tools (dprint, monochange, mdt).
# Cursor's default image does not include Nix, so this script installs a
# single-user Nix, the devenv revision locked in devenv.lock, then the
# same cargo binaries CI installs.
#
# Idempotent: a second run reuses Nix and devenv and only refreshes the
# cargo binaries. macOS and iOS builds are out of scope; cloud agents are
# Linux x86_64.
set -euo pipefail

if [ "$(uname -s)" != "Linux" ]; then
	echo "This install script supports Linux cloud agents only (found $(uname -s))." >&2
	exit 1
fi

export DEBIAN_FRONTEND=noninteractive

sudo apt-get update
sudo apt-get install -y --no-install-recommends ca-certificates curl xz-utils

nix_conf="${XDG_CONFIG_HOME:-$HOME/.config}/nix/nix.conf"
mkdir -p "$(dirname "$nix_conf")"
cat >"$nix_conf" <<'EOF'
experimental-features = nix-command flakes
accept-flake-config = true
extra-substituters = https://devenv.cachix.org
extra-trusted-public-keys = devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=
EOF

if ! command -v nix >/dev/null 2>&1; then
	curl -fsSL https://nixos.org/nix/install -o /tmp/nix-install.sh
	sh /tmp/nix-install.sh --no-daemon --yes
fi

if [ -f "$HOME/.nix-profile/etc/profile.d/nix.sh" ]; then
	# shellcheck disable=SC1091
	. "$HOME/.nix-profile/etc/profile.d/nix.sh"
fi

if ! command -v nix >/dev/null 2>&1; then
	echo "nix is not on PATH after installation." >&2
	exit 1
fi

# Login shells started later must find nix and devenv without rc files.
# Symlink the profile binaries that exist now; devenv is linked again
# after `nix profile add`.
link_profile_bins() {
	local bin
	for bin in nix nix-env nix-store nix-instantiate nix-build devenv; do
		if [ -x "$HOME/.nix-profile/bin/$bin" ]; then
			sudo ln -sfn "$HOME/.nix-profile/bin/$bin" "/usr/local/bin/$bin"
		fi
	done
}

link_profile_bins

devenv_revision="$(
	nix eval --impure --raw --expr '
    let
      lock = builtins.fromJSON (builtins.readFile ./devenv.lock);
      revision = lock.nodes.devenv.locked.rev;
    in
      if builtins.match "^[0-9a-f]{40}$" revision != null then
        revision
      else
        throw "devenv.lock does not contain an immutable Devenv revision"
  '
)"

if ! command -v devenv >/dev/null 2>&1; then
	nix profile add --accept-flake-config "github:cachix/devenv/${devenv_revision}#devenv"
	link_profile_bins
fi

devenv shell -- bash -e -c "install:cargo:bin"
