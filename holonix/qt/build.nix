let
  pkgs = import ../nixpkgs/nixpkgs.nix;
in
[
  pkgs.qt59.qmake
]
++ import ./c-bindings/build.nix
