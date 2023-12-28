{ inputs = {
  naersk = {
    url = "github:nix-community/naersk";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  fenix = {
    url = "github:nix-community/fenix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  nix-filter.url = "github:numtide/nix-filter";
};
  outputs = { nixpkgs, flake-utils, naersk, fenix, nix-filter, ...  }: flake-utils.lib.eachDefaultSystem (system:
  let
    pkgs = import nixpkgs { inherit system; };

    dev-toolchain = fenix.packages.${system}.complete.toolchain;

    wasm-toolchain = with fenix.packages.${system}; combine [
      minimal.cargo
      minimal.rustc
      targets.wasm32-unknown-unknown.latest.rust-std
    ];

    rust-wasm = naersk.lib.${system}.override {
      cargo = wasm-toolchain;
      rustc = wasm-toolchain;
    };

    darwin-compatability = with pkgs.darwin.apple_sdk.frameworks; [ Security AppKit ];
  in
  rec {
    packages = {
      wasm = rust-wasm.buildPackage {
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

      web-application = pkgs.runCommand "quibble-web-app" {} ''
        ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen ${packages.wasm}/bin/pong.wasm --out-dir $out/web-app/wasm --no-modules --no-typescript

        cp ${./public/index.html} $out/web-app/index.html
        cp -r ${./assets} $out/web-app/assets
      '';
    };

    defaultPackage = packages.web-application;


    devShell = with pkgs; mkShell {
      # buildInputs = [ rustc cargo rust-analyzer rustfmt iconv clippy darwin.apple_sdk.frameworks.AppKit ];
      buildInputs = [ dev-toolchain pkgs.wasm-bindgen-cli pkgs.simple-http-server ] ++ (if pkgs.stdenv.isDarwin then darwin-compatability else []);

      RUST_SRC_PATH = "${dev-toolchain}/lib/rustlib/src/rust/library";
    };
  });
}