let
  pkgs = import ./nixpkgs/nixpkgs.nix;

  flush = import ./src/flush.nix;
  test = import ./src/test.nix;

  release = import ./release/config.nix;

  holochain = pkgs.stdenv.mkDerivation {
   name = "holochain-conductor";

   src = pkgs.fetchurl {
    url = "https://github.com/holochain/holochain-rust/releases/download/v0.0.12-alpha1/conductor-v0.0.12-alpha1-x86_64-generic-linux-gnu.tar.gz";
    sha256 = "0wdlv85vwwp9cwnmnsp20aafrxljsxlc6m00h0905q0cydsf86kq";
   };

   unpackPhase = ":";

   installPhase = ''
     mkdir -p $out/{bin,share}
     cp $src $out/share/holochain
   '';

  };


  # import (builtins.fetchTarball holochain-src);
  # cli = import (builtins.fetchTarball https://github.com/holochain/holochain-rust/releases/download/v0.0.12-alpha1/cli-v0.0.12-alpha1-x86_64-generic-linux-gnu.tar.gz);
in
[
  # I forgot what these are for!
  # Reinstate and organise them ᕙ༼*◕_◕*༽ᕤ
  # coreutils

  flush
  test

  holochain
  # cli
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
