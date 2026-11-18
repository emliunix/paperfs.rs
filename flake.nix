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
        # Keep Cargo sources plus the HTML pages embedded with include_str!.
        # Exclude the local HawDB database and large OneDrive test fixtures.
        src = lib.cleanSourceWith {
            src = lib.cleanSource ./.;
            filter = path: type:
              let
                full = toString path;
                base = baseNameOf full;
                parent = baseNameOf (dirOf full);
              in
                !(lib.hasInfix "/target" full)
                && !(lib.hasInfix "/opendal" full)
                && !(lib.hasInfix "/paper-graph/data" full)
                && !(lib.hasInfix "/test_onedrive" full)
                && (type == "directory"
                    || lib.hasSuffix ".rs" base
                    || lib.hasSuffix ".toml" base
                    || lib.hasSuffix ".html" base
                    || base == "Cargo.lock"
                    || (parent == ".cargo" && base == "config"));
        };


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
                contents = [ paperfs_rs pkgs.cacert ];
                config.Cmd = [ "paperfs_rs" ];
            };
        };
        devShells.default = craneLib.devShell {
            inputsFrom = [ paperfs_rs ];
        };
      });
}
