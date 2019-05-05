let
  pkgs = import ./nixpkgs/nixpkgs.nix;

  flush = import ./src/flush.nix;
  test = import ./src/test.nix;

  release = import ./release/config.nix;
in
[

  flush
  test

]

++ import ./app-spec/build.nix
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
++ import ./scaffold/build.nix
++ import ./sodium/build.nix
