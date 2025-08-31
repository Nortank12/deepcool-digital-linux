{
  rustPlatform,
  pkg-config,
  libudev-zero,
  projectRoot,
  ...
}:
let
  lockFile = "${projectRoot}/Cargo.lock";
  tomlFile = builtins.fromTOML (builtins.readFile "${projectRoot}/Cargo.toml");

  requiredInputs = [
    pkg-config
    libudev-zero
  ];
in
rustPlatform.buildRustPackage {
  pname = tomlFile.package.name;
  version = tomlFile.package.version;

  src = projectRoot;
  cargoLock = {
    inherit lockFile;
  };

  buildInputs = requiredInputs;
  nativeBuildInputs = requiredInputs;
}
