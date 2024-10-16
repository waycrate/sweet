{
  description = "devshell for github:lavafroth/sweet";

  inputs.flake-utils.url = "github:numtide/flake-utils";

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
        devShells.default = pkgs.mkShell rec {
          packages = with pkgs; [
            stdenv.cc.cc.lib
          ];

          LD_LIBRARY_PATH = "${nixpkgs.lib.makeLibraryPath packages}";
        };
      }
    );
}
