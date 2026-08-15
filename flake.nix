{
  description = "My personal shell prompt built with Rust";

  nixConfig = {
    extra-substituters = [ "https://my-prompt.cachix.org" ];
    extra-trusted-public-keys = [
      "my-prompt.cachix.org-1:aIzUDavhE5lzcsn6awg73yVAUnjMrAeqPATi3XrIZ0Q="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-manifest = {
      url = "https://static.rust-lang.org/dist/channel-rust-1.96.0.toml";
      flake = false;
    };
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      fenix,
      home-manager,
      nixpkgs,
      rust-manifest,
      ...
    }:
    let
      lib = nixpkgs.lib;
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = lib.genAttrs systems;
      version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

      rustFiles = lib.fileset.unions [
        ./Cargo.lock
        ./Cargo.toml
        ./src
        ./tests
      ];
      rustSource = lib.fileset.toSource {
        root = ./.;
        fileset = rustFiles;
      };
      dependencyPolicySource = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          rustFiles
          ./deny.toml
        ];
      };
      licenseSource = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          rustFiles
          ./THIRD_PARTY_LICENSES.html
          ./about.hbs
          ./about.toml
          ./scripts/generate-third-party-licenses.sh
        ];
      };
      repositorySource = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          ./.github/workflows
          ./flake.nix
          ./nix
          ./nix-ci.nix
        ];
      };
      websiteSource = lib.fileset.toSource {
        root = ./www;
        fileset = ./www;
      };

      pnpmFor =
        pkgs:
        pkgs.pnpm_11.overrideAttrs (_: {
          version = "11.20.0";
          src = pkgs.fetchurl {
            url = "https://registry.npmjs.org/pnpm/-/pnpm-11.20.0.tgz";
            hash = "sha256-NOGYyx5DI3UX7O39MfmuJqbAo+U2bOWKLQX0sh+18Zo=";
          };
        });
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          myPrompt = pkgs.rustPlatform.buildRustPackage {
            pname = "my-prompt";
            inherit version;

            src = rustSource;
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
              license = lib.licenses.mit;
              mainProgram = "my-prompt";
              platforms = systems;
            };
          };
        in
        {
          my-prompt = myPrompt;
          default = myPrompt;
        }
        // lib.optionalAttrs (system == "x86_64-linux") {
          ci-security-audit = pkgs.writeShellApplication {
            name = "ci-security-audit";
            runtimeInputs = with pkgs; [
              cargo
              cargo-audit
              cargo-deny
              git
              rustc
            ];
            text = ''
              cargo deny --locked check advisories
              cargo audit
            '';
          };
        }
      );

      homeModules.default = import ./nix/home-manager.nix { inherit self; };

      checks = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          package = self.packages.${system}.my-prompt;
          cargoDeps = package.cargoDeps;
          pnpm = pnpmFor pkgs;

          homeConfiguration = home-manager.lib.homeManagerConfiguration {
            inherit pkgs;
            modules = [
              self.homeModules.default
              {
                home = {
                  username = "my-prompt-check";
                  homeDirectory = if pkgs.stdenv.isDarwin then "/Users/my-prompt-check" else "/home/my-prompt-check";
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

          mkRustCheck =
            {
              name,
              commands,
              nativeBuildInputs ? [
                pkgs.cargo
                pkgs.rustc
              ],
              source ? rustSource,
            }:
            pkgs.stdenv.mkDerivation {
              pname = "my-prompt-${name}";
              inherit version;
              src = source;
              inherit cargoDeps;

              nativeBuildInputs = [ pkgs.rustPlatform.cargoSetupHook ] ++ nativeBuildInputs;
              strictDeps = true;
              dontConfigure = true;

              buildPhase = ''
                runHook preBuild
                export CARGO_NET_OFFLINE=true
                ${commands}
                runHook postBuild
              '';

              installPhase = ''
                touch "$out"
              '';
            };

          msrvToolchain = (fenix.packages.${system}.fromManifestFile rust-manifest).minimalToolchain;

          website = pkgs.stdenvNoCC.mkDerivation (finalAttrs: {
            pname = "my-prompt-website";
            inherit version;
            src = websiteSource;

            pnpmDeps = pkgs.fetchPnpmDeps {
              inherit (finalAttrs) pname version src;
              inherit pnpm;
              fetcherVersion = 4;
              prePnpmInstall = ''
                pnpm config set child-concurrency 2
                pnpm config set network-concurrency 4
              '';
              env.NODE_NO_WARNINGS = "1";
              hash = "sha256-4UIJZrRrQzHPMk9Q/KBWCES8dm32+HpXk1ExkViYS6g=";
            };

            nativeBuildInputs = [
              pkgs.nodejs_24
              pnpm
              pkgs.pnpmConfigHook
            ];
            strictDeps = true;

            env = {
              CI = "true";
              WRANGLER_SEND_METRICS = "false";
            };

            buildPhase = ''
              runHook preBuild
              pnpm check
              pnpm build
              pnpm exec wrangler deploy --dry-run
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              cp -R dist "$out"
              runHook postInstall
            '';
          });
        in
        {
          inherit package;
          home-manager = homeConfiguration.activationPackage;
        }
        // lib.optionalAttrs (system == "x86_64-linux") {
          inherit website;

          rust-quality = mkRustCheck {
            name = "rust-quality";
            nativeBuildInputs = with pkgs; [
              cargo
              clippy
              fish
              git
              rustc
              rustfmt
              writableTmpDirAsHomeHook
            ];
            commands = ''
              cargo fmt -- --check
              cargo clippy --all-targets --all-features --locked -- -D warnings
              cargo build --locked --verbose
              cargo test --locked --verbose
            '';
          };

          minimum-supported-rust = mkRustCheck {
            name = "minimum-supported-rust";
            nativeBuildInputs = [ msrvToolchain ];
            commands = ''
              cargo check --all-targets --locked --verbose
            '';
          };

          dependency-policy = mkRustCheck {
            name = "dependency-policy";
            source = dependencyPolicySource;
            nativeBuildInputs = with pkgs; [
              cargo
              cargo-deny
              rustc
            ];
            commands = ''
              cargo deny --locked --offline check licenses bans sources
            '';
          };

          license-inventory = mkRustCheck {
            name = "license-inventory";
            source = licenseSource;
            nativeBuildInputs = with pkgs; [
              cargo
              cargo-about
              rustc
            ];
            commands = ''
              scripts/generate-third-party-licenses.sh generated.html
              diff -u THIRD_PARTY_LICENSES.html generated.html
            '';
          };

          repository-policy = pkgs.stdenvNoCC.mkDerivation {
            pname = "my-prompt-repository-policy";
            inherit version;
            src = repositorySource;
            nativeBuildInputs = with pkgs; [
              actionlint
              nixfmt
              shellcheck
            ];
            strictDeps = true;
            dontConfigure = true;

            buildPhase = ''
              runHook preBuild
              nixfmt --check flake.nix nix-ci.nix nix/*.nix
              actionlint .github/workflows/*.yml
              runHook postBuild
            '';

            installPhase = ''
              touch "$out"
            '';
          };
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
              nixfmt
              nodejs_24
              (pnpmFor pkgs)
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
