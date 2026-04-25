{
  description = "Flake for gztool-rs";

  inputs = {
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      crane,
      fenix,
      flake-utils,
      nixpkgs,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };
        inherit (pkgs) lib;

        zlib-ng-compat = pkgs.zlib-ng.override { withZlibCompat = true; };
        gztool = pkgs.callPackage ./gztool.nix { };

        craneLib =
          (crane.mkLib nixpkgs.legacyPackages.${system}).overrideToolchain
            fenix.packages.${system}.stable.toolchain;

        unfilteredRoot = ./.; # The original, unfiltered source
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.maybeMissing ./include)
          ];
        };

        commonArgs = {
          inherit src;

          nativeBuildInputs = with pkgs; [
            zlib-ng-compat
            gztool
            rustPlatform.bindgenHook
          ];

          buildInputs = with pkgs; [
            pkg-config
            openssl
          ];
        };

        runtimeLibDeps = with pkgs; [
          openssl
          zlib-ng-compat
        ];

        runtimeBinDeps = with pkgs; [
        ];

        gztool-rs = craneLib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;

            doCheck = false;
            # Additional environment variables or build phases/hooks can be set
            # here *without* rebuilding all dependency crates
            # MY_CUSTOM_VAR = "some value";
          }
        );
      in
      {
        inherit runtimeLibDeps;

        checks = {
          inherit gztool-rs;
        };

        packages.default = gztool-rs;

        devShells.default = pkgs.mkShell {
          inherit (commonArgs) nativeBuildInputs buildInputs;

          packages =
            with pkgs;
            [
              (pkgs.fenix.stable.withComponents [
                "cargo"
                "clippy"
                "rust-src"
                "rustc"
                "rustfmt"
              ])
              rust-analyzer-unwrapped
            ]
            ++ runtimeLibDeps
            ++ runtimeBinDeps;

          env = {
            GZTOOL_PATH = "${gztool}";
            RUST_SRC_PATH = "${pkgs.fenix.stable.rust-src}/lib/rustlib/src/rust/library";
            LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
              lib.makeLibraryPath (
                with pkgs;
                [
                  gztool
                ]
                ++ runtimeLibDeps
              )
            }";
          };
        };
      }
    );
}
