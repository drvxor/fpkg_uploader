{
  description = "fpkg uploader environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          pkgs.python3
          pkgs.python3Packages.cython
          pkgs.python3Packages.setuptools
          pkgs.gcc
        ];

        shellHook = ''
          echo "Entering fpkg dev shell..."
        '';
      };
    };
}
