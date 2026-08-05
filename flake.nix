{
  description = "A fast shell prompt built with Rust";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs, ... }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          myPrompt = pkgs.rustPlatform.buildRustPackage {
            pname = "my-prompt";
            inherit version;

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeCheckInputs = with pkgs; [
              fish
              git
              writableTmpDirAsHomeHook
            ];
            strictDeps = true;

            meta = {
              description = "My shell prompt";
              homepage = "https://github.com/cbeck527/my-prompt";
              license = pkgs.lib.licenses.mit;
              mainProgram = "my-prompt";
              platforms = systems;
            };
          };
        in
        {
          my-prompt = myPrompt;
          default = myPrompt;
        }
      );

      homeModules.default = import ./nix/home-manager.nix { inherit self; };

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              rust-analyzer
              rustfmt
              clippy
              cargo-audit
              cargo-deny
              cargo-edit
              cargo-release
              fish
              git
              stdenv.cc
            ];
          };
        }
      );
    };
}
