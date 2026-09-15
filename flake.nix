{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: {
    packages = builtins.mapAttrs (
      system: pkgs:
      let
        pks = pkgs.callPackage ./package.nix { };
      in
      {
        default = pkgs.symlinkJoin {
          name = "anisync";
          paths = [
            self.packages.${system}.anisyncd
            self.packages.${system}.anisync-cli
          ];
        };
        anisyncd = pks.anisyncd;
        anisync-cli = pks.anisync-cli;
      }
    ) nixpkgs.legacyPackages;

    devShells = builtins.mapAttrs (system: pkgs: {
      default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustc
          cargo
          clippy
          rustfmt
          rust-analyzer
        ];
      };
    }) nixpkgs.legacyPackages;

    nixosModules.default = import ./module.nix;
  };
}
