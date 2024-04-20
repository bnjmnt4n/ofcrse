{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=a76c4553d7e741e17f289224eda135423de0491d";
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
        ];

        site = pkgs.buildNpmPackage {
          name = "ofcrse-site";
          src = pkgs.lib.cleanSourceWith {
            src = pkgs.lib.cleanSource ./.;
            filter = name: type:
              let baseName = baseNameOf (toString name); in
              !(type == "directory" && (baseName == "node_modules" || baseName == "target" || baseName == "dist"));
          };

          buildInputs = [ pkgs.vips ];
          nativeBuildInputs = [ pkgs.pkg-config ];

          installPhase = ''
            runHook preInstall
            cp -pr --reflink=auto dist $out/
            runHook postInstall
          '';

          npmDepsHash = "sha256-22jDjw0E1hkWJoyypPLMGScsTXZBK8TYed+v6YwrC3s=";
        };

        server = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;

          inherit nativeBuildInputs buildInputs;
        };
      in
      rec {
        packages.site = site;
        packages.server = server;
        packages.default = server;
        apps.default = flake-utils.lib.mkApp {
          drv = server;
        };

        devShell = pkgs.mkShell {
          inputsFrom = [ server ];

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

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      });
}
