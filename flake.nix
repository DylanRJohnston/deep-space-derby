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

        toolchain = pkgs.rust-bin.stable.latest.default.override {
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

          wasm-bindgen-server = pkgs.runCommand "intermediate" { } ''
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ${packages.server-wasm}/lib/server.wasm --out-name index --target bundler --out-dir $out --no-typescript
            cp ${./build/shim.js} $out/shim.js
          '';

          wasm-bindgen-client = pkgs.runCommand "quibble-web-app" { } ''
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ${packages.client-wasm}/bin/pong.wasm --out-dir $out/web-app/wasm --no-modules --no-typescript

            cp ${./public/index.html} $out/web-app/index.html
            cp -r ${./assets} $out/web-app/assets
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
            ];

            RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          };
      });
}
