{config, pkgs, lib, self, ...}:
let
  cfg = config.services.anisync;
  system = pkgs.stdenv.hostPlatform.system
in
{
  options.services.anisync = {
    enable = lib.mkEnableOption "Enable AniSync Service";
  };

  config = lib.mkIf cfg.enable {
    systemd.services.anisync = {
        description = "AniSync Daemon";
        wantedBy = [ "multi-user.target" ];

        serviceConfig = {
            Type = "simple";
            ExecStart = "${self.packages.${system}.default}/bin/anisyncd";
            Restart = "always";
        }
    }
  }
}
