{
  description = "Deep Space Derby — development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/566acc07c54dc807f91625bb286cb9b321b5f42a";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            bun
          ];

          shellHook = ''
            echo "deep-space-derby dev shell"
            echo "  bun $(bun --version)"
            echo ""
            echo "  bun install        install dependencies"
            echo "  bun run dev        start server + both Vite apps"
            echo "  bun run test       run unit tests"
            echo "  bun run typecheck  type check all packages"
            echo ""
          '';
        };
      }
    );
}
