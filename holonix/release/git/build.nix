let
  branch = import ./src/branch.nix;
  merge-back = import ./src/merge-back.nix;
  notes = import ./src/notes.nix;
in
[
  branch
  merge-back
  notes
]
