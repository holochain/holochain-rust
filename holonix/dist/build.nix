let
  dist = import ./src/dist.nix;
  flush = import ./src/flush.nix;

  holochain = import ./src/holochain.nix;
  hc = import ./src/hc.nix;
in
[
  dist
  flush

  holochain
  hc
]
