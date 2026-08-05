{ self }:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.my-prompt;
  packages = self.packages.${pkgs.stdenv.hostPlatform.system};
in
{
  options.programs.my-prompt = {
    enable = lib.mkEnableOption "my-prompt";

    package = lib.mkPackageOption packages "my-prompt" {
      pkgsText = "inputs.my-prompt.packages.\${pkgs.stdenv.hostPlatform.system}";
    };

    enableFishIntegration = lib.hm.shell.mkFishIntegrationOption { inherit config; };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    programs.fish.interactiveShellInit = lib.mkIf cfg.enableFishIntegration ''
      ${lib.getExe cfg.package} init | source
    '';
  };
}
