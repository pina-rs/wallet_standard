{
  pkgs,
  lib,
  config,
  ...
}:

let
  llvm = pkgs.llvmPackages_19;
in
{
  packages =
    with pkgs;
    [
      binaryen
      cargo-run-bin
      chromedriver
      curl
      cmake
      dprint
      eget
      gcc
      libiconv
      llvm.bintools
      llvm.clang
      llvm.clang-tools
      llvm.libclang.lib
      llvm.lld
      llvm.llvm
      llvm.mlir
      nixfmt-rfc-style
      openssl
      perl
      pkg-config
      protobuf # needed for `solana-test-validator` in tests
      rust-jemalloc-sys
      rustup
      shfmt
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
    EGET_CONFIG = "${config.env.DEVENV_ROOT}/.eget/.eget.toml";
    OPENSSL_NO_VENDOR = "1";
    LIBCLANG_PATH = "${llvm.libclang.lib}/lib";
    CC = "${llvm.clang}/bin/clang";
    CXX = "${llvm.clang}/bin/clang++";
    PROTOC = "${pkgs.protobuf}/bin/protoc";
    LD_LIBRARY_PATH = "${config.env.DEVENV_PROFILE}/lib";
    WASM_BINDGEN_TEST_WEBDRIVER_JSON = "${config.env.DEVENV_ROOT}/setup/webdriver.json";
  };

  # Rely on the global sdk for now as the nix apple sdk is not working for me.
  # apple.sdk = if pkgs.stdenv.isDarwin then pkgs.apple-sdk_15 else null;
  apple.sdk = null;

  # Use the stdenv conditionally.
  # stdenv = if pkgs.stdenv.isLinux then llvm.stdenv else pkgs.stdenv;
  stdenv = pkgs.stdenv;

  enterShell = ''
    set -e
    export PATH="$DEVENV_ROOT/.eget/bin:$PATH";
    export LDFLAGS="$NIX_LDFLAGS";
  '';

  # disable dotenv since it breaks the variable interpolation supported by `direnv`
  dotenv.disableHint = true;

  tasks = {
    "rustfmt:nightly" = {
      exec = ''
        rustup toolchain install nightly --component rustfmt --force
      '';
      before = [ "devenv:enterShell" ];
    };
  };

  scripts = {
    "knope" = {
      exec = ''
        set -e
        cargo bin knope $@
      '';
      description = "The `knope` executable";
    };
    "wasm-bindgen-test-runner" = {
      exec = ''
        set -e
        cargo bin wasm-bindgen-test-runner $@
      '';
      description = "The `wasm-bindgen-test-runner` executable";
    };
    "install:all" = {
      exec = ''
        set -e
        install:cargo:bin
        install:eget
      '';
      description = "Install all packages.";
      binary = "bash";
    };
    "install:eget" = {
      exec = ''
        HASH=$(nix hash path --base32 ./.eget/.eget.toml)
        echo "HASH: $HASH"
        if [ ! -f ./.eget/bin/hash ] || [ "$HASH" != "$(cat ./.eget/bin/hash)" ]; then
          echo "Updating eget binaries"
          eget -D --to "$DEVENV_ROOT/.eget/bin"
          echo "$HASH" > ./.eget/bin/hash
        else
          echo "eget binaries are up to date"
        fi
      '';
      description = "Install github binaries with eget.";
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
          echo "Builing project locally"
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
        RUSTUP_TOOLCHAIN="nightly" RUSTDOCFLAGS="--cfg docsrs" cargo doc --workspace
      '';
      description = "Build documentation site.";
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
    "lint:all" = {
      exec = ''
        set -e
        lint:clippy
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
        pids=$(lsof -i :8899 -t)

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
