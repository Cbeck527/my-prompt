{
  systems = [ "x86_64-linux" ];

  onlyBuild = [
    "packages.x86_64-linux.my-prompt"
    "checks.x86_64-linux.home-manager"
    "checks.x86_64-linux.rust-quality"
    "checks.x86_64-linux.minimum-supported-rust"
    "checks.x86_64-linux.dependency-policy"
    "checks.x86_64-linux.license-inventory"
    "checks.x86_64-linux.repository-policy"
    "checks.x86_64-linux.website"
  ];

  cachix = {
    name = "my-prompt";
    public-key = "my-prompt.cachix.org-1:aIzUDavhE5lzcsn6awg73yVAUnjMrAeqPATi3XrIZ0Q=";
  };

  # Run outside the Nix sandbox so both tools fetch current RustSec data.
  test.security-advisories = {
    package = "packages.x86_64-linux.ci-security-audit";
    system = "x86_64-linux";
    branches = "any";
    in-repo = true;
  };
}
