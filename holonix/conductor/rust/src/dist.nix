let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;
  release = import ../../../release/config.nix;

  dist-lib = import ../../../dist/rust/lib.nix;

  name = "hc-conductor-rust-dist";

  artifact = {
   path = "conductor";
   name = "holochain";
  };

  script = pkgs.writeShellScriptBin name
  ''
  ${dist-lib.build-rust-artifact artifact}
  '';
in
script
