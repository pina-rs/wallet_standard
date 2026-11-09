#!/usr/bin/env bash
# Bootstrap a Linux cloud-agent VM so it can build, lint, and test this repo.
#
# The project toolchain is the pinned devenv shell (devenv.nix / devenv.yaml /
# devenv.lock): Rust from rust-toolchain.toml, cargo binaries from
# workspace.metadata.bin, and the lint tools (dprint, monochange, mdt).
# Cursor's default image does not include Nix, so this script installs a
# single-user Nix with flakes and nix-command, then the devenv revision
# locked in devenv.lock from the devenv Cachix cache.
#
# Idempotent: a second run reuses Nix and devenv and only refreshes the
# shell and cargo binaries. macOS and iOS builds are out of scope; cloud
# agents are Linux x86_64.
set -euo pipefail

if [ "$(uname -s)" != "Linux" ]; then
	echo "This install script supports Linux cloud agents only (found $(uname -s))." >&2
	exit 1
fi

export DEBIAN_FRONTEND=noninteractive

sudo apt-get update
sudo apt-get install -y --no-install-recommends ca-certificates curl xz-utils

# System config is read by every Nix invocation, including non-login shells
# that never source a profile. cache.nixos.org stays in the list because
# `substituters` replaces the default rather than appending to it.
sudo mkdir -p /etc/nix
sudo tee /etc/nix/nix.conf >/dev/null <<'EOF'
experimental-features = nix-command flakes
accept-flake-config = true
substituters = https://cache.nixos.org https://devenv.cachix.org
trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=
EOF

# The system file owns the substituters. Repeating them here makes Nix
# list devenv.cachix.org twice.
nix_conf="${XDG_CONFIG_HOME:-$HOME/.config}/nix/nix.conf"
mkdir -p "$(dirname "$nix_conf")"
cat >"$nix_conf" <<'EOF'
experimental-features = nix-command flakes
accept-flake-config = true
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

# Login shells source /etc/profile.d. Non-login shells do not, so the
# profile binaries are also symlinked into /usr/local/bin, which is on
# the default PATH.
sudo tee /etc/profile.d/nix-devenv.sh >/dev/null <<'EOF'
if [ -f "${HOME}/.nix-profile/etc/profile.d/nix.sh" ]; then
	. "${HOME}/.nix-profile/etc/profile.d/nix.sh"
fi
case ":${PATH}:" in
*":/usr/local/bin:"*) ;;
*) PATH="/usr/local/bin:${PATH}" ;;
esac
export PATH
EOF

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

# Pin devenv to the lockfile revision and pull it from devenv.cachix.org.
# `nixpkgs#devenv` would float and would not match CI.
if ! nix profile list | grep -q "github:cachix/devenv/${devenv_revision}"; then
	nix profile install --accept-flake-config "github:cachix/devenv/${devenv_revision}#devenv"
	link_profile_bins
fi

if ! command -v devenv >/dev/null 2>&1; then
	echo "devenv is not on PATH after installation." >&2
	exit 1
fi

# Realize the shell once so later `devenv shell` commands hit the cache.
devenv shell -- true

# cargo-nextest and the other aliases in .cargo/config.toml come from
# workspace.metadata.bin. CI installs them the same way.
devenv shell -- bash -e -c "install:cargo:bin"

nix --version
devenv version
