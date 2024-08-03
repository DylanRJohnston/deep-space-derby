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
  outputs = { nixpkgs, flake-utils, naersk, rust-overlay, nix-filter, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        wasm-bindgen-cli-update = final: prev: {
          wasm-bindgen-cli = prev.wasm-bindgen-cli.override {
            version = "0.2.92";
            hash = "sha256-1VwY8vQy7soKEgbki4LD+v259751kKxSxmo/gqE6yV0=";
            cargoHash = "sha256-aACJ+lYNEU8FFBs158G1/JG8sc6Rq080PeKCMnwdpH0=";
          };
        };

        overlays = [ (import rust-overlay) (wasm-bindgen-cli-update) ];

        pkgs = import nixpkgs { inherit system overlays; };

        toolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

        buildWorkspacePackage = pname:
          naersk'.buildPackage {
            inherit pname;

            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            cargoBuildOptions = options: options ++ [ "-p" pname ];
            copyLibs = true;

            src = nix-filter {
              root = ./.;

              include = [
                ./Cargo.toml
                ./Cargo.lock

                ./client/Cargo.toml
                ./client/src/main.rs

                ./server/Cargo.toml
                ./server/src/lib.rs

                ./core/Cargo.toml
                ./core/src/lib.rs

                pname
              ];
            };
          };
      in rec {
        packages = {
          client-wasm = buildWorkspacePackage "client";
          server-wasm = buildWorkspacePackage "server";

          wasm-bindgen = pkgs.writeShellScriptBin "wasm-bindgen" ''
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen $1 --out-name index --target bundler --out-dir $2 --no-typescript
          '';

          build-site = pkgs.writeShellScriptBin "site" ''
            set -x
            set -o nounset
            set -o errexit
            set -o pipefail

            rm -rf site/*
            cp -r assets/* site/
            cp -r game/assets site/assets

            cargo build --target wasm32-unknown-unknown --no-default-features --release -p app  --bin worker --features app/ssr
            cargo build --target wasm32-unknown-unknown --no-default-features --release -p app  --bin client --features app/hydrate
            cargo build --target wasm32-unknown-unknown --no-default-features --release -p game --bin game          

            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ./target/wasm32-unknown-unknown/release/worker.wasm --no-typescript --out-name index --target bundler --out-dir ./site    
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ./target/wasm32-unknown-unknown/release/client.wasm --no-typescript --out-name index --target web     --out-dir ./site/pkg
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ./target/wasm32-unknown-unknown/release/game.wasm   --no-typescript --out-name game  --target web     --out-dir ./site/pkg
          '';

          dev = pkgs.writeShellScriptBin "dev" ''
            set -x
            set -o nounset
            set -o errexit
            set -o pipefail

            rm -rf site/*
            cp -r assets/* site/
            ln -s ../game/assets site/assets

            export RUST_LOG=info

            WORKER_TARGET="./target/wasm32-unknown-unknown/debug/worker.wasm"
            CLIENT_TARGET="./target/wasm32-unknown-unknown/debug/client.wasm"
            GAME_TARGET="./target/wasm32-unknown-unknown/release/game.wasm"

            #entr can't execute bash functions, so we do a little bash metaprogramming
            function build() {
              echo "cargo build --target wasm32-unknown-unknown --no-default-features -p app --bin $1 --features app/$2"
            }

            function build_worker() {
              echo "$(build worker ssr)"
            }

            function build_client() {
              echo "$(build client hydrate)"
            }

            function build_game() {
              echo "cargo build --target wasm32-unknown-unknown --no-default-features --release -p game --bin game"
            }

            function bindgen() {
              echo "${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen --keep-debug --no-typescript $1 --out-name $2 --target $3 --out-dir $4"
            }

            function bindgen_worker() {
                echo "$(bindgen $WORKER_TARGET index bundler ./site)"
            }

            function bindgen_client() {
                echo "$(bindgen $CLIENT_TARGET index web ./site/pkg)"
            }

            function bindgen_game() {
              echo "$(bindgen $GAME_TARGET game web ./site/pkg)"
            }

            # Need to build things synchronously first so they're available for wangler
            $(build_client)
            $(bindgen_client)

            $(build_worker)
            $(bindgen_worker)

            $(build_game)
            $(bindgen_game)

            find app | ${pkgs.entr}/bin/entr -n $(build_worker) &
            find app | ${pkgs.entr}/bin/entr -n $(build_client) &
            find game | ${pkgs.entr}/bin/entr -n $(build_game) &

            echo $WORKER_TARGET | ${pkgs.entr}/bin/entr -n $(bindgen_worker) &
            echo $CLIENT_TARGET | ${pkgs.entr}/bin/entr -n $(bindgen_client) &
            echo $GAME_TARGET | ${pkgs.entr}/bin/entr -n $(bindgen_game) &

            find assets | ${pkgs.entr}/bin/entr cp -r assets/* site &

            find site | grep -Ev '(pkg|_worker.js)' | ${pkgs.entr}/bin/entr touch site/_worker.js &

            wrangler pages dev site --local-protocol https --compatibility-date=2023-10-30 &

            wait
          '';
        };

        devShell = with pkgs;
          mkShell {
            buildInputs = [
              toolchain
              iconv
              darwin.apple_sdk.frameworks.AppKit
              wrangler
              nodejs
              wasm-bindgen-cli
              entr
              cargo-watch
              cargo-expand
              cargo-leptos
              leptosfmt
              twiggy
            ];

            RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          };
      });
}
