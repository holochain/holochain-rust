let
  pkgs = import ../nixpkgs/nixpkgs.nix;
in
[
  pkgs.git
  pkgs.gitAndTools.git-hub
  pkgs.github-release
]
