# `mc-honeypot`

This is a server-side implementation of minecraft's [Server List Ping Protocol](https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Server_List_Ping). It
logs all incoming connections. The server's appearance in the client server list is fully customizable. This
implementation supports both the current protocol (1.7+) and the legacy protocol (1.6 and below).

## Why

There are certain people who use programs to scan the internet for minecraft servers. I wanted a simple way to collect
metrics about these scanning tools that does not involve running an actual minecraft server.

## Usage

```
mc-honeypot [OPTIONS]
```

## Options

```
  -p, --port <PORT>
          The port the honeypot will listen on [default: 25565]
  -v, --version-string <VERSION_STRING>
          The version string displayed by the Client [default: 1.20.4]
      --protocol-version <PROTOCOL_VERSION>
          This is used by clients to determine if it is compatible with our server [default: 765]
  -m, --max-players <MAX_PLAYERS>
          The displayed maximum player count [default: 100]
  -o, --online-players <ONLINE_PLAYERS>
          The displayed online player count. Defaults to player count if not provided
      --players <NAME:UUID>
          The Username and UUID (seperated by ":") of fake players you want to add to the server (providable multiple times)
      --motd <MOTD>
          The displayed "Message of the Day" [default: "§aHello, World"]
  -i, --icon-file <ICON_FILE>
          Path to png image which is displayed as the server icon. Needs to be 64x64 pixels in size
  -w, --webhook-url <WEBHOOK_URL>
          URL of discord webhook to send logs to
  -h, --help
          Print help
  -V, --version
          Print version
```

## Nix

If you are using the Nix package manager, you can run it using flakes with:
`nix run "github:Duckulus/mc-honeypot" -- [OPTIONS]`

If you are running a NixOS server and using flakes,
you can import it to run as a systemd service like this:
```nix
{
  description = "NixOS configuration";

  inputs = {
    mc-honeypot.url = "github:Duckulus/mc-honeypot";
  };

  outputs = { self, nixpkgs, mc-honeypot }: {
    nixosConfigurations = {
      hostname = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          mc-honeypot.nixosModules.default
          {
            services.mc-honeypot = {
              enable = true;

              openFirewall = true;

              settings = {
                # Interpreted as command line options
              };
            };
          }
        ];
      };
    };
  };
}
```
