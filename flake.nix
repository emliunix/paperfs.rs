# -*- indent-tabs-mode: nil; tab-width: 2; -*-

{
  description = "paperfs.rs flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;


        args = {
            inherit src;
            inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
            pname = "paperfs_rs";
            strictDeps = true;
            buildInputs = with pkgs; [
                openssl
            ];
            nativeBuildInputs = with pkgs; [
                pkg-config
            ];
        };
        paperfs_rs = craneLib.buildPackage args;
      in
      {
        packages = {
            default = paperfs_rs;
            image = pkgs.dockerTools.buildLayeredImage {
                name = "paperfs";
                tag = "latest";
                contents = [ paperfs_rs ];
                config.Cmd = [ "paperfs_rs" ];
            };
        };
        devShells.default = craneLib.devShell {
            inputsFrom = [ paperfs_rs ];
        };
      });
}
