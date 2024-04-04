{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=a3ed7406349a9335cb4c2a71369b697cecd9d351";
    flake-utils.url = "github:numtide/flake-utils?rev=b1d9ab70662946ef0850d488da1c9019f3a9752a";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rust;

        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = [ pkgs.openssl ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          # pkgs.darwin.libiconv
        ];

        ofcrse = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;

          inherit nativeBuildInputs buildInputs;
        };
      in
      rec {
        # TODO: build Astro app.
        packages.default = ofcrse;
        apps.default = flake-utils.lib.mkApp {
          drv = ofcrse;
        };

        devShell = pkgs.mkShell {
          inputsFrom = [ ofcrse ];

          buildInputs = [
            pkgs.flyctl
            pkgs.nodejs
            pkgs.nodePackages."@astrojs/language-server"
            pkgs.nodePackages.typescript-language-server
            rust
            pkgs.rust-analyzer
          ] ++ pkgs.lib.optional (!pkgs.stdenv.isDarwin) [
            pkgs.ttfautohint
            (pkgs.python3.withPackages (ps: [ ps.fonttools ] ++ ps.fonttools.optional-dependencies.woff))
          ];

          RUST_LOG = "info";
          RUST_BACKTRACE = 1;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      });
}
