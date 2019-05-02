let
  pkgs = import ../nixpkgs/nixpkgs.nix;

  audit = import ./src/audit.nix;
  dist = import ./src/dist.nix;
  flush = import ./src/flush.nix;
in
[
  pkgs.nix-prefetch-scripts

  audit
  dist
  flush
]
++ import ./cli/build.nix
++ import ./conductor/build.nix
