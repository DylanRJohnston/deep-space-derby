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

          wasm-bindgen-server = pkgs.runCommandLocal "intermediate" { } ''
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ${packages.server-wasm}/lib/server.wasm  \
              --out-name index \
              --target bundler \
              --outdir $out \
              --no-typescript

            cp ${./build/shim.js} $out/shim.js
          '';

          wasm-bindgen-client = pkgs.runCommandLocal "quibble-web-app" { } ''
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ${packages.client-wasm}/bin/pong.wasm --out-dir $out/web-app/wasm --no-modules --no-typescript

            cp ${./public/index.html} $out/web-app/index.html
            cp -r ${./assets} $out/web-app/assets
          '';

          dev = pkgs.writeShellScriptBin "dev" ''
            rm -rf site/*
            cp -r assets/* site/

            SERVER_TARGET="./target/wasm32-unknown-unknown/debug/server.wasm"
            CLIENT_TARGET="./target/wasm32-unknown-unknown/debug/client.wasm"

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

            function bindgen() {
              echo "${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen $1 --out-name index --target $2 --keep-debug --out-dir $3 --no-typescript"
            }

            function bindgen_server() {
                echo "$(bindgen $SERVER_TARGET bundler ./site)"
            }

            function bindgen_client() {
                echo "$(bindgen $CLIENT_TARGET web ./site/pkg)"
            }

            # Need to build things synchronously first so they're available for wangler
            $(build_client)
            $(bindgen_client)

            $(build_server)
            $(bindgen_server)

            find server | ${pkgs.entr}/bin/entr -n $(build_server) &
            find server | ${pkgs.entr}/bin/entr -n $(build_client) &

            echo $SERVER_TARGET | ${pkgs.entr}/bin/entr -n $(bindgen_server) &
            echo $CLIENT_TARGET | ${pkgs.entr}/bin/entr -n $(bindgen_client) &

            find assets | ${pkgs.entr}/bin/entr cp -r assets/* site &

            find site | ${pkgs.entr}/bin/entr -rn wrangler pages dev site --compatibility-date=2023-10-30 &

            wait
          '';
        };

        devShell = with pkgs;
          mkShell {
            buildInputs = [
              toolchain
              iconv
              simple-http-server
              darwin.apple_sdk.frameworks.AppKit
              wrangler
              nodejs
              wasm-bindgen-cli
              entr
              cargo-watch
              cargo-expand
              trunk
              cargo-leptos
              leptosfmt
            ];

            RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          };
      });
}
