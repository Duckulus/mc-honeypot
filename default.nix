{ selfpkgs, lib, ... }:
{
  imports = [ ./default_service.nix ];

  options.services.mc-honeypot = {
    enable = lib.mkEnableOption "mc-honeypot";

    package = lib.mkPackageOption selfpkgs "mc-honeypot" {};

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Automatically open ports in the firewall.";
    };

    settings = lib.mkOption {
      default = {};
      description = "Arguments provided to the program. See <https://github.com/Duckulus/mc-honeypot#options>";
      type = lib.types.submodule {
        freeformType = lib.types.attrsOf lib.types.str;

        options = {
          port = lib.mkOption {
            type = lib.types.port;
            default = 25565;
            description = "The port mc-honeypot will be listening on";
          };

          icon-file = lib.mkOption {
            type = with lib.types; nullOr (oneOf [ str path ] );
            default = null;
            description = "Path to png image which is displayed as the server icon. Needs to be 64x64 pixels in size";
          };

          webhook-url = lib.mkOption {
            type = lib.types.nullOr lib.types.str;
            default = null;
            description = ''
              URL of discord webhook to send logs to.
              Please use `webhook-url-file` instead if you don't want to commit
              your webhook url to your git repository.
            '';
          };
          webhook-url-file = lib.mkOption {
            type = lib.types.nullOr lib.types.str;
            default = null;
            description = "Path to the file containing the webhook url";
          };
        };
      };
    };
  };
}
