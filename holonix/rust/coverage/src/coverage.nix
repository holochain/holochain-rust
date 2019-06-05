let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-rust-coverage";

  # TODO this is currently dead and segfaults
  # @see https://github.com/xd009642/tarpaulin/issues/190
  script = pkgs.writeShellScriptBin name
  ''
  cargo tarpaulin --ignore-tests --timeout 600 --all --out Xml -v -e holochain_core_api_c_binding -e hdk -e hc -e lib3h_persistence_derive
  '';
in
script
