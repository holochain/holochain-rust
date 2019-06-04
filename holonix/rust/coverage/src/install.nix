let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-rust-coverage-install";

  script = pkgs.writeShellScriptBin name
  ''
  if ! cargo --list | grep --quiet tarpaulin;
  then RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin;
  fi;
  '';
in
script
