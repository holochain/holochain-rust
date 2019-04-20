let
  sync = import ./src/sync.nix;
  tag = import ./src/tag.nix;
in
[
  sync
  tag
]
