let
  pkgs = import ../../../../nixpkgs/nixpkgs.nix;

  name = "hc-release-docs-changelog-list-missing-references";

  script = pkgs.writeShellScriptBin name
  ''
  cat CHANGELOG.md | grep -E '^-\s' | grep -Ev '[0-9]\]' | cat
  '';
in
script
