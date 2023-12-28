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
      in rec {
        packages = {
          wasm = naersk'.buildPackage {
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

            src = nix-filter.lib {
              root = ./.;
              include = [
                "Cargo.lock"
                "Cargo.toml"
                (nix-filter.lib.inDirectory "src")
                (nix-filter.lib.inDirectory "assets")
              ];
            };
          };

          web-application = pkgs.runCommand "quibble-web-app" { } ''
            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ${packages.wasm}/bin/pong.wasm --out-dir $out/web-app/wasm --no-modules --no-typescript

            cp ${./public/index.html} $out/web-app/index.html
            cp -r ${./assets} $out/web-app/assets
          '';
        };

        defaultPackage = packages.web-application;

        devShell = with pkgs;
          mkShell {
            buildInputs = [
              toolchain
              iconv
              simple-http-server
              darwin.apple_sdk.frameworks.AppKit
            ];

            RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          };
      });
}
