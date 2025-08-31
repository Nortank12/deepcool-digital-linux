flake:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  inherit (lib)
    mkOption
    mkIf
    types
    optionals
    ;
  cfg = config.hardware.deepcool-digital-linux;
  pkg = cfg.package;

  daemonArgs = lib.escapeShellArgs (
    [ ]
    ++ optionals (cfg.systemd.mode != null) [
      "--mode"
      cfg.systemd.mode
    ]
    ++ optionals (cfg.systemd.secondary != null) [
      "--secondary"
      cfg.systemd.secondary
    ]
    ++ optionals (cfg.systemd.pid != null) [
      "--pid"
      (toString cfg.systemd.pid)
    ]
    ++ optionals (cfg.systemd.updateMs != null) [
      "--update"
      (toString cfg.systemd.updateMs)
    ]
    ++ optionals cfg.systemd.fahrenheit [ "--fahrenheit" ]
    ++ optionals cfg.systemd.alarm [ "--alarm" ]
  );
in
{
  options.hardware.deepcool-digital-linux = {
    enable = mkOption {
      type = types.bool;
      default = false;
      description = "Enable the DeepCool digital software";
    };
    package = mkOption {
      type = types.package;
      default = flake.packages.${pkgs.stdenv.hostPlatform.system}.default;
      description = "Set DeepCool Digital package to be used";
    };

    systemd = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable the systemd service for DeepCool Digital";
      };

      mode = mkOption {
        type = types.nullOr (
          types.enum [
            "auto"
            "cpu_freq"
            "cpu_fan"
            "gpu"
            "psu"
          ]
        );
        default = null;
        description = "Change the display mode of your device";
      };

      secondary = mkOption {
        type = types.nullOr (
          types.enum [
            "auto"
            "cpu_freq"
            "cpu_fan"
            "gpu"
            "psu"
          ]
        );
        default = null;
        description = "Change the secondary display mode of your device (if supported)";
      };

      pid = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Specify the Product ID if you use multiple devices";
      };

      updateMs = mkOption {
        type = types.nullOr (types.ints.between 100 2000);
        default = null;
        description = "Change the update interval of the display in milliseconds";
      };

      fahrenheit = mkOption {
        type = types.bool;
        default = false;
        description = "Change the temperature unit to °F";
      };

      alarm = mkOption {
        type = types.bool;
        default = false;
        description = "Enable the alarm";
      };
    };
  };

  config = mkIf cfg.enable ({
    environment.systemPackages = [ pkg ];

    systemd.services.deepcool-digital-linux = mkIf cfg.systemd.enable {
      description = "DeepCool Digital Software Daemon";
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Type = "simple";
        ExecStart = "${pkg}/bin/deepcool-digital-linux ${daemonArgs}";
        Restart = "on-failure";
      };
    };

    services.udev.packages = [
      (pkgs.writeTextDir "etc/udev/rules.d/41-deepcool-digital-linux.rules" ''
        # Intel RAPL energy usage file
        ACTION=="add", SUBSYSTEM=="powercap", KERNEL=="intel-rapl:0", RUN+="${pkgs.coreutils}/bin/chmod 444 /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj"

        # DeepCool HID raw devices
        SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3633", MODE="0666"

        # CH510 MESH DIGITAL
        SUBSYSTEM=="hidraw", ATTRS{idVendor}=="34d3", ATTRS{idProduct}=="1100", MODE="0666"
      '')
    ];
  });
}
