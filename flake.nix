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
        # nixosModule = pkgs.${system}.callPackage ./module.nix { inherit default; };
        nixosModule =
          {
            config,
            lib,
            ...
          }:
          let
            cfg = config.services.deepcool-digital;
          in
          # user = "ravensiris-web";
          # dataDir = "/var/lib/ravensiris-web";
          {
            options.services.deepcool-digital = {
              enable = lib.mkEnableOption "deepcool-digital";
            };
            config = lib.mkIf cfg.enable {
              environment.systemPackages = [ default ];

              # users.users.${user} = {
              #   isSystemUser = true;
              #   group = user;
              #   home = dataDir;
              #   createHome = true;
              # };
              # users.groups.${user} = { };

              systemd.services = {
                deepcool-digital = {
                  description = "Start up deepcool-digital";
                  #wantedBy = [ "multi-user.target" ];
                  script = ''
                    ${default}/bin/deepcool-digital-linux
                  '';
                  #  serviceConfig = {
                  #    User = user;
                  #    WorkingDirectory = "${dataDir}";
                  #    Group = user;
                  #  };
                };
              };
            };
          };
      });
    };
}
