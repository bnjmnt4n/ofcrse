{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?rev=38860c9e91cb00f4d8cd19c7b4e36c45680c89b5";
    flake-utils.url = "github:numtide/flake-utils?rev=7e2a3b3dfd9af950a856d66b0a7d01e3c18aa249";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
      in {
       devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            flyctl
            nodejs-18_x
          ];
        };
      });
}
