{
  pkgs,
  ...
}:
pkgs.rustPlatform.buildRustPackage {
  pname = "deepcool-digital-linux";
  version = "0.1";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
  nativeBuildInputs = with pkgs; [
    hidapi
    pkg-config
  ];
  buildInputs = with pkgs; [
    libudev-zero
  ];
}
