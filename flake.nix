{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=0c6d8c783336a59f4c59d4a6daed6ab269c4b361";
    flake-utils.url = "github:numtide/flake-utils?rev=4022d587cbbfd70fe950c1e2083a02621806a725";
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
            pkgs.nodejs-18_x
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
