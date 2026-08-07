{
  description = "My personal shell prompt built with Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      home-manager,
      nixpkgs,
      ...
    }:
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

      checks = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          homeConfiguration = home-manager.lib.homeManagerConfiguration {
            inherit pkgs;
            modules = [
              self.homeModules.default
              {
                home = {
                  username = "my-prompt-check";
                  homeDirectory =
                    if pkgs.stdenv.isDarwin then
                      "/Users/my-prompt-check"
                    else
                      "/home/my-prompt-check";
                  stateVersion = "24.11";
                };
                programs = {
                  fish.enable = true;
                  my-prompt = {
                    enable = true;
                    enableFishIntegration = true;
                  };
                };
              }
            ];
          };
        in
        {
          package = self.packages.${system}.my-prompt;
          home-manager = homeConfiguration.activationPackage;
        }
      );

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
              cargo-about
              cargo-deny
              cargo-edit
              cargo-release
              fish
              git
              actionlint
              hyperfine
              nodejs_24
              pnpm
              typescript-go
              typescript-language-server
              shellcheck
              stdenv.cc
            ];
          };
        }
      );
    };
}
