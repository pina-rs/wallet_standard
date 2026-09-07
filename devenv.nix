{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

let
  llvm = pkgs.llvmPackages_21;
  custom = inputs.ifiokjr-nixpkgs.packages.${pkgs.stdenv.hostPlatform.system};
in

{
  packages =
    with pkgs;
    [
      binaryen
      cargo-audit
      cargo-binstall
      cargo-deny
      cargo-llvm-cov
      cargo-nextest
      cargo-run-bin
      cargo-semver-checks
      chromedriver
      cmake
      curl
      custom.agave
      custom.monochange
      dprint
      gcc
      git
      gitleaks
      libiconv
      mdbook
      llvm.bintools
      llvm.clang
      llvm.clang-tools
      llvm.libclang.lib
      llvm.lld
      llvm.llvm
      llvm.mlir
      ninja
      nixfmt-rfc-style
      openssl
      perl
      pkg-config
      protobuf # needed for `solana-test-validator` in tests
      rust-jemalloc-sys
      # Upstream rustup 1.28+ fails in nix builds: check suite is network-sensitive
      # and the install phase fails generating shell completions because the sandbox
      # creates an empty settings.toml missing the required `version` field.
      (rustup.overrideAttrs (old: {
        doCheck = false;
        preInstall = (old.preInstall or "") + ''
          export HOME="$(mktemp -d)"
          mkdir -p "$HOME/.rustup"
          echo 'version = "12"' > "$HOME/.rustup/settings.toml"
        '';
      }))
      shfmt
      zizmor
      zlib
      zstd
    ]
    ++ lib.optionals stdenv.isDarwin [
      coreutils
    ]
    ++ lib.optionals stdenv.isLinux [
      libgcc.lib
      udev
    ];

  env = {
    OPENSSL_NO_VENDOR = "1";
    LIBCLANG_PATH = "${llvm.libclang.lib}/lib";
    CC = "${llvm.clang}/bin/clang";
    CXX = "${llvm.clang}/bin/clang++";
    PROTOC = "${pkgs.protobuf}/bin/protoc";
    LD_LIBRARY_PATH = "${config.env.DEVENV_PROFILE}/lib";
    WASM_BINDGEN_TEST_WEBDRIVER_JSON = "${config.env.DEVENV_ROOT}/setup/webdriver.json";
  }
  # cc-rs compiles vendored C/C++ by passing `--target=arm64-apple-macosx` to
  # `clang++`. The nix cc-wrapper explicitly does not support that override
  # ("multi-target compilers" warning) and its mishandled include paths then
  # fail to resolve libc++ headers against the Xcode sysroot (`unknown type
  # name 'uint8_t'`). Hand cc-rs Apple's toolchain on macOS instead; it pairs
  # with the global Xcode SDK, matching `apple.sdk = null` above.
  #
  # HOST_* is required because the nix stdenv setup hooks re-export
  # CC=clang/CXX=clang++ after `env`, silently overriding shell-level CC/CXX
  # values; cc-rs prefers HOST_* over CC/CXX, and nothing overwrites those.
  // lib.optionalAttrs pkgs.stdenv.hostPlatform.isDarwin {
    HOST_CC = "/usr/bin/clang";
    HOST_CXX = "/usr/bin/clang++";
  };

  # Rely on the global sdk for now as the nix apple sdk is not working for me.
  apple.sdk = null;

  # Use the stdenv conditionally.
  stdenv = pkgs.stdenv;

  git-hooks = {
    package = pkgs.prek;
    hooks = {
      "secrets:commit" = {
        enable = true;
        verbose = true;
        pass_filenames = false;
        name = "secrets";
        description = "Scan staged changes for leaked secrets with gitleaks.";
        entry = "${pkgs.gitleaks}/bin/gitleaks protect --staged --verbose --redact";
        stages = [ "pre-commit" ];
      };
      dprint = {
        enable = true;
        verbose = true;
        pass_filenames = true;
        name = "dprint fmt";
        description = "Format changed files with dprint before commit.";
        entry = "${pkgs.dprint}/bin/dprint fmt --allow-no-files";
        stages = [ "pre-commit" ];
      };
      nixfmt-rfc-style = {
        enable = true;
        pass_filenames = true;
        name = "nixfmt";
        description = "Format changed nix files.";
        entry = "${pkgs.nixfmt-rfc-style}/bin/nixfmt";
        stages = [ "pre-commit" ];
      };
    };
  };

  enterShell = ''
    set -e
    export LDFLAGS="$NIX_LDFLAGS";
  '';

  # disable dotenv since it breaks the variable interpolation supported by `direnv`
  dotenv.disableHint = true;

  tasks = {
    "rustfmt:nightly" = {
      exec = ''
        # Without --force this is idempotent: rustup skips the install when the
        # toolchain already exists, so an interrupted download can never leave a
        # broken toolchain (missing librustc_driver) behind on CI.
        rustup toolchain install nightly --profile minimal --component rustfmt --no-self-update
      '';
      before = [ "devenv:enterShell" ];
    };
  };

  scripts = {
    "release:change" = {
      exec = ''
        set -euo pipefail
        monochange run change $@
      '';
      description = "Create a changeset for the next release. Pass --package <name> --reason <text> and --bump <patch|minor|major>.";
    };
    "release:local" = {
      exec = ''
        set -euo pipefail
        # Runs the same steps as the CI release-pr workflow: PrepareRelease,
        # format, CommitRelease and (unless --create-pr=false) OpenReleaseRequest.
        monochange run release $@
      '';
      description = "Run the release flow locally to prepare, commit and open the release pull request.";
    };
    "publish:local" = {
      exec = ''
        set -euo pipefail
        # Escape hatch when CI publishing fails: verifies publish readiness
        # from the release record, then publishes every package with the local
        # cargo credentials (CARGO_REGISTRY_TOKEN or ~/.cargo/credentials).
        monochange step publish-readiness --from HEAD --format json
        monochange step publish-packages --log-level info --all
      '';
      description = "Publish the prepared release locally from the current release commit.";
    };
    "wasm-bindgen-test-runner" = {
      exec = ''
        set -e
        cargo bin wasm-bindgen-test-runner $@
      '';
      description = "The `wasm-bindgen-test-runner` executable";
    };
    "install:cargo:bin" = {
      exec = ''
        set -e
        cargo bin --install
      '';
      description = "Install cargo binaries locally.";
    };
    "copy:js" = {
      exec = ''
        set -e
        curl -L https://esm.sh/v135/@wallet-standard/app@1/es2022/app.development.mjs -o $DEVENV_ROOT/crates/wallet_standard_browser/js/app.js
        curl -L https://esm.sh/v135/@wallet-standard/wallet@1/es2022/wallet.development.mjs -o $DEVENV_ROOT/crates/wallet_standard_browser/js/wallet.js
        dprint fmt "./crates/wallet_standard_browser/js/*.js"
      '';
      description = "Copy the JS needed for the `wallet_standard_browser`.";
    };
    "update:deps" = {
      exec = ''
        set -e
        cargo update
        devenv update
        copy:js
      '';
      description = "Update dependencies.";
    };
    "build:all" = {
      exec = ''
        set -e
        if [ -z "$CI" ]; then
          echo "Building project locally"
          cargo build --all-features
        else
          echo "Building in CI"
          cargo build --all-features --locked
        fi
      '';
      description = "Build all crates with all features activated.";
    };
    "build:docs" = {
      exec = ''
        set -euo pipefail
        mdbook build "$DEVENV_ROOT/docs"
      '';
      description = "Build the mdBook documentation.";
    };
    "docs:serve" = {
      exec = ''
        set -euo pipefail
        mdbook serve "$DEVENV_ROOT/docs"
      '';
      description = "Serve the mdBook documentation locally with live reload.";
    };
    "test:all" = {
      exec = ''
        set -e
        cargo test_wallet_standard
      '';
      description = "Run all tests across the crates";
    };
    "coverage:all" = {
      exec = ''
        set -e
        cargo coverage_wallet_standard
        cargo coverage_codecov_report
      '';
      description = "Run coverage across the crates";
    };
    "security:deny" = {
      exec = ''
        set -euo pipefail
        # cargo-deny 0.20+ auto-discovers deny.toml from the working directory;
        # the --config flag was removed from the CLI.
        cargo-deny check bans licenses sources
      '';
      description = "Run cargo-deny checks (bans, licenses, sources).";
    };
    "security:audit" = {
      exec = ''
        set -euo pipefail
        cargo audit --file Cargo.lock
      '';
      description = "Audit Rust dependencies against the RustSec advisory database.";
    };
    "security:zizmor" = {
      exec = ''
        set -euo pipefail
        # --no-online-audits keeps the gate reproducible and sandbox-friendly;
        # offline audits still cover workflow injection, credential persistence,
        # dependency pinning, and permissions hardening.
        zizmor --no-online-audits --no-progress .github
      '';
      description = "Audit GitHub Actions workflows and composite actions with zizmor.";
    };
    "security:all" = {
      exec = ''
        set -e
        security:deny
        security:audit
        security:zizmor
      '';
      description = "Run all dependency and workflow security checks.";
    };
    "fix:all" = {
      exec = ''
        set -e
        fix:clippy
        fix:format
      '';
      description = "Fix all autofixable problems.";
    };
    "fix:format" = {
      exec = ''
        set -e
        dprint fmt --config "$DEVENV_ROOT/dprint.json"
      '';
      description = "Format files with dprint.";
    };
    "fix:clippy" = {
      exec = ''
        set -e
        cargo clippy --fix --allow-dirty --allow-staged --all-features
      '';
      description = "Fix clippy lints for rust.";
    };
    "lint:monochange" = {
      exec = ''
        set -euo pipefail
        monochange check
      '';
      description = "Validate monochange release metadata.";
    };
    "lint:all" = {
      exec = ''
        set -e
        lint:clippy
        lint:monochange
        lint:format
      '';
      description = "Run all checks.";
    };
    "lint:format" = {
      exec = ''
        set -e
        dprint check
      '';
      description = "Check that all files are formatted.";
    };
    "lint:clippy" = {
      exec = ''
        set -e
        cargo clippy --all-features
      '';
      description = "Check that all rust lints are passing.";
    };
    "validator:run" = {
      exec = ''
        set -e
        solana-test-validator --warp-slot 1000 --reset --quiet
      '';
      description = "Run the solana validator.";
    };
    "validator:bg" = {
      exec = ''
        set -e
        validator:kill
        validator:run
      '';
      description = "Run the solana validator in the background";
    };
    "validator:kill" = {
      exec = ''
        port=8899
        pids=$(lsof -i :$port -t)

        if [ -n "$pids" ]; then
          kill $pids
          echo "Killed processes listening on port $port: $pids"
        else
          echo "No processes found listening on port $port"
        fi
      '';
      description = "Kill any running validator";
    };
    "setup:vscode" = {
      exec = ''
        set -e
        rm -rf .vscode
        cp -r $DEVENV_ROOT/setup/editors/vscode .vscode
      '';
      description = "Setup the environment for vscode.";
    };
    "setup:helix" = {
      exec = ''
        set -e
        rm -rf .helix
        cp -r $DEVENV_ROOT/setup/editors/helix .helix
      '';
      description = "Setup for the helix editor.";
    };
  };
}
