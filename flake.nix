{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=0eeebd64de89e4163f4d3cf34ffe925a5cf67a05";
    flake-utils.url = "github:numtide/flake-utils?rev=a1720a10a6cfe8234c0e93907ffe81be440f4cef";
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

        nativeBuildInputs = [ pkgs.openssl pkgs.pkg-config ];
        buildInputs = [ ];

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
            pkgs.ttfautohint
            (pkgs.python3.withPackages(ps: [ps.fonttools] ++ ps.fonttools.optional-dependencies.woff))
            rust
            pkgs.rust-analyzer
          ];

          RUST_LOG = "info";
          RUST_BACKTRACE = 1;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      });
}
