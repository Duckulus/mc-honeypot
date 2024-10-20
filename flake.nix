{
  description = "Minecraft Server Scanner Honeypot";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
  flake-utils.lib.eachDefaultSystem (system:
  let
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

    my-crate = craneLib.buildPackage (commonArgs // {
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      #OPENSSL_DIR = "${pkgs.openssl}/etc/ssl";
      #PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
    });
  in
  {
    checks = {
      inherit my-crate;
    };

    packages.default = my-crate;

    apps.default = flake-utils.lib.mkApp {
      drv = my-crate;
    };

    devShells.default = craneLib.devShell {
      # Inherit inputs from checks.
      checks = self.checks.${system};
    };
  });
}
