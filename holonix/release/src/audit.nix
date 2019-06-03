let
  pkgs = import ../../nixpkgs/nixpkgs.nix;
  git = import ../../git/config.nix;
  release = import ../config.nix;
  release-pulse = import ../pulse/config.nix;

  name = "hc-release-audit";

  script = pkgs.writeShellScriptBin name
  ''
  echo
  echo "Current git:"
  echo

  git show --pretty=oneline

  echo
  echo "All the important release vars:"
  echo

  echo "Target commit: ${release.commit}"

  echo

  echo "Dev pulse URL hash: ${release-pulse.url-hash}"
  echo "Dev pulse version: ${release-pulse.version}"
  echo "Dev pulse URL (derived): ${release-pulse.url}"

  echo

  echo "New core version: ${release.core.version.current}"
  echo "Previous core version: ${release.core.version.previous}"

  echo

  echo "New node conductor version: ${release.node-conductor.version.current}"
  echo "Previous node conductor version: ${release.node-conductor.version.previous}"

  echo

  echo "Release process url: ${release.process-url}"
  '';
in
script
