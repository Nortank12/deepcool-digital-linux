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
            cfg = config.services.ravensiris-web;
            user = "ravensiris-web";
            dataDir = "/var/lib/ravensiris-web";
          in
          {
            options.services.ravensiris-web = {
              enable = lib.mkEnableOption "ravensiris-web";
            };
            config = lib.mkIf cfg.enable {
              environment.systemPackages = [ default ];

              users.users.${user} = {
                isSystemUser = true;
                group = user;
                home = dataDir;
                createHome = true;
              };
              users.groups.${user} = { };

              systemd.services = {
                ravensiris-web = {
                  description = "Start up the homepage";
                  wantedBy = [ "multi-user.target" ];
                  script = ''
                    echo 'ok'
                  '';
                  serviceConfig = {
                    User = user;
                    WorkingDirectory = "${dataDir}";
                    Group = user;
                  };
                };
              };
            };
          };
      });
    };
}
