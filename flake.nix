{
  description = "Development environment for my-prompt";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              rustfmt
              clippy
              cargo-audit
              cargo-deny
              fish
              git
              perl
              gnumake
              stdenv.cc
            ];
          };
        });

      # TODO: Use Nix package outputs to build multi-platform release artifacts.
    };
}
