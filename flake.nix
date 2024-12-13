{
  description = "Deepcool Digital";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgs = nixpkgs.legacyPackages;
    in
    {
      packages = forAllSystems (system: rec {
        default = pkgs.${system}.callPackage ./default.nix { };
        nixosModule = pkgs.${system}.callPackage ./module.nix { inherit default; };
      });
    };
}
