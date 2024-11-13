{
  description = "Minecraft Server Scanner Honeypot";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, crane, ... }:
  let
    system = "x86_64-linux";


    pkgs = nixpkgs.legacyPackages.${system};

    craneLib = crane.mkLib pkgs;
    commonArgs = {
      src = craneLib.cleanCargoSource ./.;
      #strictDeps = true;

      buildInputs = with pkgs; [
        openssl
        pkg-config
      ];
    };

    mc-honeypot = craneLib.buildPackage (commonArgs // {
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      #OPENSSL_DIR = "${pkgs.openssl}/etc/ssl";
      #PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
    });
  in {
    checks.${system} = {
      inherit mc-honeypot;
    };

    packages.${system} = {
      mc-honeypot = mc-honeypot;
      default =    mc-honeypot;
    };

    nixosModules.default = import ./. {
      inherit (nixpkgs) lib;
      selfpkgs = self.packages.${system};
    };

    devShells.${system}.default = craneLib.devShell {
      # Inherit inputs from checks.
      checks = self.checks.${system};
    };
  };
}
