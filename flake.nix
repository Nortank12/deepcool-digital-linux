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
        nixosModule =
          {
            config,
            lib,
            ...
          }:
          let
            cfg = config.services.deepcool-digital;
          in
          {
            options.services.deepcool-digital = {
              enable = lib.mkEnableOption "deepcool-digital";
              mode = lib.mkOption {
                type = lib.types.enum [
                  "temp"
                  "usage"
                  "power"
                  "auto"
                ];
                default = "temp";
                description = "Change the display mode between temp, usage, power, auto. default: temp";
              };
              product_id = lib.mkOption {
                type = lib.types.str;
                default = "";
                description = "Specify the Product ID if you use mutiple devices";
              };
              use_fahrenheit = lib.mkOption {
                type = lib.types.bool;
                default = false;
                description = "Change temperature unit to Fahrenheit";
              };
              alarm = lib.mkOption {
                type = lib.types.bool;
                default = false;
                description = "Enable the alarm [85˚C | 185˚F]";
              };
            };
            config = lib.mkIf cfg.enable {
              environment.systemPackages = [ default ];

              systemd.services = {
                deepcool-digital = {
                  description = "Start up deepcool-digital";
                  script = ''
                    ${default}/bin/deepcool-digital-linux --mode ${config.services.deepcool-digital.mode} ${
                      lib.strings.optionalString (config.services.deepcool-digital.product_id != "")
                        "--pid ${config.services.deepcool-digital.product_id} ${lib.strings.optionalString config.services.deepcool-digital.use_fahrenheit "-f"} ${lib.strings.optionalString config.services.deepcool-digital.alarm "-a"}"
                    }
                  '';
                };
              };
            };
          };
      });
    };
}
