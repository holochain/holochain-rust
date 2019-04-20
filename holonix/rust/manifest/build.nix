let
  install = import ./src/install.nix;
  list-unpinned = import ./src/list-unpinned.nix;
in
[
  install
  list-unpinned
]
