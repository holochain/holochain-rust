let
  install = import ./src/install.nix;
  list-unpinned = import ./src/list-unpinned.nix;
  set-ver = import ./src/set-ver.nix;
  test-ver = import ./src/test-ver.nix;
in
[
  install
  list-unpinned
  set-ver
  test-ver
]
