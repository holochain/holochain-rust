let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;
  release = import ../../config.nix;
  git = import ../../../git/config.nix;

  name = "hc-release-pulse-sync";

  script = pkgs.writeShellScriptBin name
  ''
   export GITHUB_USER='holochain'
   export GITHUB_REPO='holochain-rust'
   export GITHUB_TOKEN=$( git config --get hub.oauthtoken )

   echo
   echo 'Injecting medium summary/highlights into github release notes'
   echo
   github-release -v edit --tag ${release.core.tag} --name ${release.core.tag} --description "$( hc-release-github-notes )" --pre-release
  '';
in
script
