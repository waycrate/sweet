{
  description = "devshell for github:waycrate/sweet";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      # leaving out darwin as it cannot run wayland
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [ pkgs.stdenv.cc.cc ];
          LD_LIBRARY_PATH = pkgs.stdenv.cc.cc.LIBRARY_PATH;
        };
      });
    };

}
