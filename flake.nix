{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=c07b471b52be8fbc49a7dc194e9b37a6e19ee04d";
    flake-utils.url = "github:numtide/flake-utils?rev=846b2ae0fc4cc943637d3d1def4454213e203cba";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
      in {
       devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            flyctl
            nodejs-16_x
          ];
        };
      });
}
