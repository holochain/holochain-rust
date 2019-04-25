let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-rust-manifest-set-ver";

  script = pkgs.writeShellScriptBin name
  ''
   # node dist can mess with the process
   hc-node-flush
   find . -name "Cargo.toml" | xargs -I {} cargo upgrade "$1" --all --manifest-path {}
  '';
in
script
