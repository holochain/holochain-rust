let
  list-missing-references = import ./src/list-missing-references.nix;
  sync-version = import ./src/sync-version.nix;
in
[
 list-missing-references
 sync-version
]
