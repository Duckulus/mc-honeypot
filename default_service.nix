{ config, lib, ... }:
with lib;
let
  cfg = config.services.mc-honeypot;
in
{
  config = mkMerge [
    (mkIf cfg.enable {
      networking.firewall.allowedTCPPorts = (mkIf cfg.openFirewall [ cfg.settings.port ]);

      systemd.services.mc-honeypot = {
        description = "MC Honeypot Server";
        documentation = [ "https://github.com/Duckulus/mc-honeypot" ];
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];

        serviceConfig = {
          DynamicUser = true;
          User = "mc-honeypot";

          ExecStart = "${cfg.package}/bin/mc-honeypot --" + (
            lib.concatStringsSep " --" (
              # Main program args
              lib.mapAttrsToList 
              (n: v: "${n} '${builtins.toString v}'") 
              (
                # Filter out the nulls
                lib.filterAttrs 
                (n: v: v != null)
                (
                  # We do our own processing on this arg
                  builtins.removeAttrs cfg.settings [ "webhook-url-file" ]
                )
              )
            )) + (
              # Webhook URL
              if   (cfg.settings.webhook-url-file != null)
              then (" --webhook-url " + (builtins.readFile cfg.settings.webhook-url-file))
              else ""
            );

          Restart = "on-failure";
          RestartSec = 10;
        };
      };
    })
  ];
}
