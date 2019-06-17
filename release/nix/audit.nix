{ pkgs, config, pulse }:
let
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
  echo "Target commit: ${config.commit}"
  echo
  echo "Dev pulse URL hash: ${pulse.config.url-hash}"
  echo "Dev pulse version: ${pulse.config.version}"
  echo "Dev pulse URL (derived): ${pulse.config.url}"
  echo
  echo "New core version: ${config.version.current}"
  echo "Previous core version: ${config.version.previous}"
  echo
  echo "Release process url: ${config.process-url}"
  '';
in
script
