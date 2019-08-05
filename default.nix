{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

let
  buildHolochain = args:
    let
      inherit (rust.packages.nightly) rustPlatform;
    in
    (buildRustPackage rustPlatform args).overrideAttrs (super: {
      buildType = "debug";
      nativeBuildInputs = super.nativeBuildInputs ++ [ buildPackages.perl ];

      OPENSSL_STATIC = "1";
      RUST_SODIUM_LIB_DIR = "${pkgsStatic.libsodium}/lib";
    });
in

{
  holochain-cli = buildHolochain {
    name = "holochain-cli";
    src = gitignoreSource ./.;
    cargoDir = "cli";
  };

  holochain-conductor = buildHolochain {
    name = "holochain-conductor";
    src = gitignoreSource ./.;
    cargoDir = "conductor";
  };
}
