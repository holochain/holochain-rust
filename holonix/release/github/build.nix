let
  branch = import ./src/branch.nix;
  check-artifacts = import ./src/check-artifacts.nix;
  merge-back = import ./src/merge-back.nix;
  notes = import ./src/notes.nix;
  pr = import ./src/pr.nix;
in
[
  branch
  check-artifacts
  merge-back
  notes
  pr
]
