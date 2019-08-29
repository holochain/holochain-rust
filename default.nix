{ pkgs ? import ./pkgs.nix {} }:

with pkgs;
with lib;

let
  buildHolochain = args:
    let
      inherit (rust.packages.nightly) rustPlatform;
    in
    (buildRustPackage rustPlatform args).overrideAttrs (super: {
      buildType = "debug";

      nativeBuildInputs = super.nativeBuildInputs ++ [ buildPackages.perl ];
      buildInputs = optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
        CoreServices
        Security
      ]);

      OPENSSL_STATIC = "1";
      RUST_SODIUM_LIB_DIR = "${libsodium}/lib";
    } // optionalAttrs (!stdenv ? "static") {
      RUST_SODIUM_SHARED = "1";
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
