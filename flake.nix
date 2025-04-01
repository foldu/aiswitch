{
  description = "Building static binaries with musl";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # FIXME: build this with nix too, currently bun lockfiles aren't supported
        frontend = pkgs.fetchzip {
          url = "https://github.com/foldu/aiswitch/releases/download/frontend%2Fv0.1.0/frontend.zip";
          sha256 = "sha256-TczvtZluX8tRTKpaTjbESbalcCjjVBa1k5WfB+WHzZU=";
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            targets = [ "x86_64-unknown-linux-musl" ];
          }
        );
        unfilteredRoot = ./.;
        lib = pkgs.lib;

        aiswitch = craneLib.buildPackage {
          src = lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = lib.fileset.unions [
              # Default files from crane (Rust and cargo files)
              (craneLib.fileset.commonCargoSources unfilteredRoot)
              # Also keep any html files
              (lib.fileset.fileFilter (file: file.hasExt "html") unfilteredRoot)
              (lib.fileset.maybeMissing ./frontend/dist)
            ];
          };
          preBuildPhases = [ "copyFrontend" ];

          copyFrontend = ''
            if ! [[ $pname =~ -deps$ ]]; then
                cp -r ${frontend}/* frontend/dist
            fi
          '';

          strictDeps = true;

          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        };
      in
      {
        checks = {
          inherit aiswitch;
        };

        packages = {
          default = aiswitch;
          inherit aiswitch;
        };
      }
    );
}
