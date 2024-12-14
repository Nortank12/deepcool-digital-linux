{
  description = "Deepcool Digital Module";
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
            user = "deepcool-digital";
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
              services.udev.extraRules = ''
                # Intel RAPL energy usage file
                ACTION=="add", SUBSYSTEM=="powercap", KERNEL=="intel-rapl:0", RUN+="${pkgs.${system}.coreutils}/bin/chmod 444 /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj"

                # DeepCool HID raw devices
                SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3633", MODE="0666"
              '';

              users.users.${user} = {
                isSystemUser = true;
                group = user;
                home = "/var/empty";
                shell = "${pkgs.${system}.util-linux}/bin/nologin";
              };
              users.groups.${user} = { };
              systemd.services = {
                deepcool-digital = {
                  description = "Start up deepcool-digital";
                  script = ''
                    ${default}/bin/deepcool-digital-linux \
                      --mode ${cfg.mode} \
                      ${lib.optionalString (cfg.product_id != "") "--pid ${cfg.product_id}"} \
                      ${lib.optionalString cfg.use_fahrenheit "-f"} \
                      ${lib.optionalString cfg.alarm "-a"}
                  '';

                  serviceConfig = {
                    User = user;
                    Group = user;
                  };

                };

              };
            };
          };
      });
    };
}
