{config, lib, anisync, ...}:
let
  cfg = config.services.anisync;
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
            ExecStart = "${anisync}/bin/anisyncd";
            Restart = "always";
        };
    };
  };
}
