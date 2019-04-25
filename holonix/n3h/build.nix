let
  pkgs = import ../nixpkgs/nixpkgs.nix;
in
[
  # which is used to manage the n3h AppImage
  pkgs.which
]
