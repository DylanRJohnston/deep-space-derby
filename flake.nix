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
        overlays = [ (import rust-overlay) ];

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
            cp -r simulation/assets site/assets

            cargo build -p server     --release --bins --target wasm32-unknown-unknown --no-default-features --features server/ssr
            cargo build -p server     --release --lib  --target wasm32-unknown-unknown --no-default-features --features server/hydrate
            cargo build -p simulation --release        --target wasm32-unknown-unknown --no-default-features

            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ./target/wasm32-unknown-unknown/release/server.wasm     --no-typescript --out-name index --target bundler --out-dir ./site    
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ./target/wasm32-unknown-unknown/release/client.wasm     --no-typescript --out-name index --target web     --out-dir ./site/pkg
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ./target/wasm32-unknown-unknown/release/simulation.wasm --no-typescript --out-name index --target web     --out-dir ./site/pkg
          '';

          dev = pkgs.writeShellScriptBin "dev" ''
            rm -rf site/*
            cp -r assets/* site/
            ln -s ../simulation/assets site/assets

            SERVER_TARGET="./target/wasm32-unknown-unknown/debug/server.wasm"
            CLIENT_TARGET="./target/wasm32-unknown-unknown/debug/client.wasm"
            SIMULATION_TARGET="./target/wasm32-unknown-unknown/release/simulation.wasm"

            #entr can't execute bash functions, so we do a little bash metaprogramming
            function build() {
              echo "cargo build -p server --$1 --no-default-features --features server/$2 --target wasm32-unknown-unknown"
            }

            function build_server() {
              echo "$(build bins ssr)"
            }

            function build_client() {
              echo "$(build lib hydrate)"
            }

            function build_simulation() {
              echo "cargo build -p simulation --target wasm32-unknown-unknown --release"
            }

            function bindgen() {
              echo "${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen $1 --out-name index --target $2 --keep-debug --out-dir $3 --no-typescript"
            }

            function bindgen_server() {
                echo "$(bindgen $SERVER_TARGET bundler ./site)"
            }

            function bindgen_client() {
                echo "$(bindgen $CLIENT_TARGET web ./site/pkg)"
            }

            function bindgen_simulation() {
              echo "wasm-bindgen $SIMULATION_TARGET --out-name simulation --target web --out-dir ./site/pkg --no-typescript"
            }

            # Need to build things synchronously first so they're available for wangler
            $(build_client)
            $(bindgen_client)

            $(build_server)
            $(bindgen_server)

            $(build_simulation)
            $(bindgen_simulation)

            find server | ${pkgs.entr}/bin/entr -n $(build_server) &
            find server | ${pkgs.entr}/bin/entr -n $(build_client) &
            find simulation | ${pkgs.entr}/bin/entr -n $(build_simulation) &

            echo $SERVER_TARGET | ${pkgs.entr}/bin/entr -n $(bindgen_server) &
            echo $CLIENT_TARGET | ${pkgs.entr}/bin/entr -n $(bindgen_client) &
            echo $SIMULATION_TARGET | ${pkgs.entr}/bin/entr -n $(bindgen_simulation) &

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
            ];

            RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          };
      });
}
