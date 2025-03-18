{
  inputs = {
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nix-filter.url = "github:numtide/nix-filter";
  };
  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        # wasm-bindgen-cli-update = final: prev: {
        #   wasm-bindgen-cli = prev.wasm-bindgen-cli.override {
        #     version = "0.2.99";
        #     hash = "sha256-1AN2E9t/lZhbXdVznhTcniy+7ZzlaEp/gwLEAucs6EA=";
        #     cargoHash = "sha256-DbwAh8RJtW38LJp+J9Ht8fAROK9OabaJ85D9C/Vkve4=";
        #   };
        # };

        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs { inherit system overlays; };

        toolchain = pkgs.rust-bin.nightly.latest.complete.override {
          targets = [ "wasm32-unknown-unknown" ];
        };

        cargo-llvm-cov =
          let
            src = pkgs.fetchzip {
              url = "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.6.14/cargo-llvm-cov-aarch64-apple-darwin.tar.gz";
              hash = "sha256-Gs621B/BcPYnmYHDBJofryugLuFoefPhfPKnacafH38=";
            };
          in
          pkgs.runCommand "cargo-llvm-cov" { } ''
            mkdir -p $out/bin
            ${pkgs.tree}/bin/tree
            cp ${src}/cargo-llvm-cov $out/bin/cargo-llvm-cov
          '';

      in
      rec {
        packages = {
          build-site = pkgs.writeShellScriptBin "build-site" ''
            set -x
            set -o nounset
            set -o errexit
            set -o pipefail

            rm -rf site/*
            cp -r assets/. site/
            cp -r game/assets site/assets

            cargo build --target wasm32-unknown-unknown --no-default-features --release -p app  --lib        --features app/hydrate
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ./target/wasm32-unknown-unknown/release/app.wasm    --no-typescript --remove-name-section --remove-producers-section --out-name index --target web     --out-dir ./site/pkg

            cargo build --target wasm32-unknown-unknown --no-default-features --release -p app  --bin worker --features app/ssr
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ./target/wasm32-unknown-unknown/release/worker.wasm --no-typescript --remove-name-section --remove-producers-section --out-name index --target bundler --out-dir ./site

            cargo build --target wasm32-unknown-unknown --no-default-features --release -p game --bin game
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ./target/wasm32-unknown-unknown/release/game.wasm   --no-typescript --remove-name-section --remove-producers-section --out-name game  --target web     --out-dir ./site/pkg
          '';

          deploy-site = pkgs.writeShellScriptBin "deploy-site" ''
            set -x
            set -o nounset
            set -o errexit
            set -o pipefail

            ${packages.build-site}/bin/build-site

            wrangler deploy
          '';

          dev-clean = pkgs.writeShellScriptBin "dev-clean" ''
            set -x
            set -o nounset
            set -o errexit
            set -o pipefail

            rm -rf site/*
            ln -s ../game/assets site/assets
          '';

          dev-copy-assets = pkgs.writeShellScriptBin "dev-copy-assets" ''
            set -x
            set -o nounset
            set -o errexit
            set -o pipefail

            cp -r assets/* site/
          '';

          dev-build-client = pkgs.writeShellScriptBin "dev-build-client" ''
            set -o nounset
            set -o errexit
            set -o pipefail

            CLIENT_TARGET="./target/wasm32-unknown-unknown/debug/app.wasm"

            cargo build --target wasm32-unknown-unknown --no-default-features -p app --lib --features app/hydrate
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen $CLIENT_TARGET --keep-debug --no-typescript --out-name index --target web --out-dir ./site/pkg

            echo "############### FINISHED BUILDING CLIENT ###############"
          '';

          dev-build-game = pkgs.writeShellScriptBin "dev-build-game" ''
            set -o nounset
            set -o errexit
            set -o pipefail

            GAME_TARGET="./target/wasm32-unknown-unknown/debug/game.wasm"

            cargo build --target wasm32-unknown-unknown --no-default-features -p game --bin game
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen $GAME_TARGET --no-typescript --out-name game --target web --out-dir ./site/pkg

            echo "############### FINISHED BUILDING GAME ###############"
          '';

          dev-run-native-server = pkgs.writeShellScriptBin "dev-run-native-server" ''
            set -o nounset
            set -o errexit
            set -o pipefail

            RUST_LOG=info cargo run --package app --bin server
          '';

          dev-run-wrangler-server = pkgs.writeShellScriptBin "dev-run-wrangler-server" ''
            set -o nounset
            set -o errexit
            set -o pipefail

            cargo build --target wasm32-unknown-unknown --no-default-features -p app --bin worker --features app/ssr
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ./target/wasm32-unknown-unknown/debug/worker.wasm --keep-debug --no-typescript --out-name index --target bundler --out-dir ./site
            wrangler pages dev site --ip 0.0.0.0 --local-protocol https --compatibility-date=2023-10-30
          '';

          dev-mprocs-config = pkgs.writeText "mprocs.dev.yaml" ''
            procs:
              clean: ${packages.dev-clean}/bin/dev-clean
              assets: find assets | entr -r ${packages.dev-copy-assets}/bin/dev-copy-assets
              client: find {app,shared} | entr -n ${packages.dev-build-client}/bin/dev-build-client
              game: find {game,shared} | entr -n ${packages.dev-build-game}/bin/dev-build-game
              server: find {app,shared} | entr -rn ${packages.dev-run-native-server}/bin/dev-run-native-server
          '';

          dev-wrangler-mprocs-config = pkgs.writeText "mprocs.wrangler.dev.yaml" ''
            procs:
              clean: ${packages.dev-clean}/bin/dev-clean
              assets: find assets | entr -r ${packages.dev-copy-assets}/bin/dev-copy-assets
              client: find {app,shared} | entr -n ${packages.dev-build-client}/bin/dev-build-client
              game: find {game,shared} | entr -n ${packages.dev-build-game}/bin/dev-build-game
              server: find {app,shared} | entr -rn ${packages.dev-run-wrangler-server}/bin/dev-run-wrangler-server
          '';

          dev = pkgs.writeShellScriptBin "dev" ''
            ${pkgs.mprocs}/bin/mprocs --config ${packages.dev-mprocs-config}
          '';

          dev-wrangler = pkgs.writeShellScriptBin "dev-wrangler" ''
            ${pkgs.mprocs}/bin/mprocs --config ${packages.dev-wrangler-mprocs-config}
          '';
        };

        devShell =
          with pkgs;
          with packages;
          mkShell {
            nativeBuildInputs = with pkgs; [ rustPlatform.bindgenHook ];
            buildInputs =
              [
                binaryen
                build-site
                cargo-expand
                cargo-leptos
                cargo-llvm-cov
                cargo-watch
                deploy-site
                dev
                dev-build-client
                dev-build-game
                dev-clean
                dev-copy-assets
                dev-run-native-server
                dev-run-wrangler-server
                dev-wrangler
                entr
                iconv
                leptosfmt
                llvmPackages.bintools-unwrapped
                mprocs
                nodejs
                toolchain
                twiggy
                wasm-bindgen-cli
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (
                with pkgs.darwin.apple_sdk.frameworks;
                [
                  AudioToolbox
                  AudioUnit
                  AppKit
                  CoreAudio
                  CoreFoundation
                  CoreMIDI
                  OpenAL
                ]
              );

            RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          };
      }
    );
}
