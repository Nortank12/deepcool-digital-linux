{
  description = "Linux version for the DeepCool Digital Windows software";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    (inputs.flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ (import inputs.rust-overlay) ];
        };
      in
      {
        packages.default = pkgs.callPackage ./nix/package.nix {
          inherit inputs;
          projectRoot = builtins.path { path = ./.; };
        };

        devShells.default = import ./nix/devShell.nix pkgs;
      }
    ))
    // {
      nixosModules.default = import ./nix/module.nix inputs.self;
    };
}
