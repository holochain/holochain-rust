let
  pkgs = import ../../../../nixpkgs/nixpkgs.nix;
  rust = import ../../../../rust/config.nix;

  name = "hc-release-docs-readme-list-stale-nightlies";

  script = pkgs.writeShellScriptBin name
  ''
  find . -iname "readme.*" | xargs cat | grep -E 'nightly-' | grep -v '${rust.nightly.date}' | cat
  '';
in
script
