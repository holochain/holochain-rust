let
  list-missing-references = import ./src/list-missing-references.nix;
  sync-version = import ./src/version-sync.nix;
in
[
 list-missing-references
 sync-version
]
