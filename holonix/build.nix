let
  dist = import ./src/dist.nix;
  flush = import ./src/flush.nix;
in
[
  dist
  flush
]

++ import ./cli/build.nix
++ import ./conductor/build.nix
++ import ./darwin/build.nix
++ import ./dist/build.nix
++ import ./git/build.nix
++ import ./node/build.nix
++ import ./openssl/build.nix
++ import ./qt/build.nix
++ import ./release/build.nix
++ import ./rust/build.nix
