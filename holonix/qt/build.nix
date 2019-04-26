let
  pkgs = import ../nixpkgs/nixpkgs.nix;
in
[
  pkgs.qt5.qmake
]
++ import ./c-bindings/build.nix
