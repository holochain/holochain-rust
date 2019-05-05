let
  pkgs = import ../nixpkgs/nixpkgs.nix;
in
[

  # fails to build in some contexts without this
  # e.g. gcc issues in scaffolding tests on circle ci
  pkgs.coreutils

]
