{
  pkgs,
  ...
}:

{
  packages = with pkgs; [
    hidapi
    libudev-zero
  ];
  languages.rust.enable = true;
}
