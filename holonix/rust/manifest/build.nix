let
  install = import ./src/install.nix;
  list-unpinned = import ./src/list-unpinned.nix;
  set-ver = import ./src/set-ver.nix;
in
[
  install
  list-unpinned
  set-ver
]
