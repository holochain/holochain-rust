let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-rust-coverage-install";

  version = "0.8.0";

  script = pkgs.writeShellScriptBin name
  ''
   if ! cargo --list | grep --quiet tarpaulin;
   then
    RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install --version ${version} cargo-tarpaulin;
   fi;
  '';
in
script
