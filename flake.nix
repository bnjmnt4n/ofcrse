{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=b2c1f10bfbb3f617ea8e8669ac13f3f56ceb2ea2";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, rust-overlay }:
    let
      systems = ["aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux"];
      forEachSystem = systems: f: builtins.foldl' (acc: system: nixpkgs.lib.recursiveUpdate acc (f system)) {} systems;
    in
    forEachSystem systems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        npmPackageCommonArgs = {
          src = pkgs.lib.cleanSource ./site;

          buildInputs = [ pkgs.vips ];
          nativeBuildInputs = [ pkgs.pkg-config ];

          installPhase = ''
            runHook preInstall
            mkdir $out
            cp -r dist $out/
            runHook postInstall
          '';

          npmDepsHash = "sha256-vwHvdIMFSOEIZApCtXjk+KI1vqzHJ81FZIcRAWqWUww=";
        };

        siteDev = pkgs.buildNpmPackage (npmPackageCommonArgs // {
          name = "ofcrse-site-dev";
          npmBuildScript = "build-dev";
        });
        site = pkgs.buildNpmPackage (npmPackageCommonArgs // {
          name = "ofcrse-site";
          npmBuildScript = "build";
        });

        rust = pkgs.rust-bin.stable.latest.default;

        craneLib = (crane.mkLib pkgs).overrideToolchain rust;

        cranePackageCommonArgs = {
          src = craneLib.cleanCargoSource ./server;
          strictDeps = true;

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.libiconv
          ];
        };

        server = craneLib.buildPackage (cranePackageCommonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly cranePackageCommonArgs;
        });

        # TODO: Setup cross compilation? Probably not required...
        dockerImageCommonArgs = {
          tag = self.rev or self.dirtyRev or "latest";
          contents = [
            # Debugging utilities for `fly ssh console`
            pkgs.busybox
            # Now required by the fly.io sshd
            pkgs.dockerTools.fakeNss
            server
          ];
          config = {
            Cmd = [ "${server}/bin/ofcrse" ];
            WorkingDir = "/";
          };
        };
        dockerImage = pkgs.dockerTools.streamLayeredImage (dockerImageCommonArgs // {
          name = "registry.fly.io/ofcrse";
          contents = dockerImageCommonArgs.contents ++ [ site ];
        });
        dockerImageDev = pkgs.dockerTools.streamLayeredImage (dockerImageCommonArgs // {
          name = "registry.fly.io/ofcrse-dev";
          tag = self.rev or self.dirtyRev or "latest";
          contents = dockerImageCommonArgs.contents ++ [ siteDev ];
        });
      in
      {
        checks.${system} = {
          inherit site siteDev server;
        };
        packages.${system} = {
          inherit site siteDev server dockerImage dockerImageDev;
          default = server;
        };

        devShells.${system}.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = [
            (rust.override {
              extensions = [ "rust-src" "rustfmt" "rust-analyzer" ];
            })
            pkgs.dive
            pkgs.flyctl
            pkgs.nodejs
            pkgs.nodePackages."@astrojs/language-server"
            pkgs.nodePackages.typescript-language-server
          ] ++ pkgs.lib.optional (!pkgs.stdenv.isDarwin) [
            pkgs.ttfautohint
            (pkgs.python3.withPackages (ps: [ ps.fonttools ] ++ ps.fonttools.optional-dependencies.woff))
          ];
        };
      });
}
