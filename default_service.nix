{ config, lib, pkgs, ... }:
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
          # TODO: If this is used, then the service will no longer
          #       be able to access the webhook-url-file tometimes.
          #       So find a way to get around it.
          #DynamicUser = true;
          #User = "mc-honeypot";

          # NOTE: Using bash to evaluate the webhook-url-file
          ExecStart = "${pkgs.bash}/bin/bash -c \"${cfg.package}/bin/mc-honeypot --" + (
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
              then " --webhook-url \\\\\\$(cat ${cfg.settings.webhook-url-file})"
              else ""
            ) + "\"";

          Restart = "on-failure";
          RestartSec = 10;
        };
      };
    })
  ];
}
