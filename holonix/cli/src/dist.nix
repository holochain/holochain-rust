let
  pkgs = import ../../nixpkgs/nixpkgs.nix;
  release = import ../../release/config.nix;

  dist-lib = import ../../dist/rust/lib.nix;

  name = "hc-cli-dist";

  artifact = {
   path = "cli";
   name = "hc";
  };

  script = pkgs.writeShellScriptBin name
  ''
   ${dist-lib.build-rust-artifact artifact}
  '';
in
script
