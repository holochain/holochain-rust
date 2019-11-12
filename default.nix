{ pkgs ? import ./pkgs.nix {} }:

with pkgs;
with lib;

let
  buildHolochain = args:
    let
      inherit (rust.packages.nightly) rustPlatform;

    in
    (buildRustPackage rustPlatform args).overrideAttrs (super: {
      nativeBuildInputs = super.nativeBuildInputs ++ (with buildPackages; [
        nodejs-12_x
        perl
      ]);

      buildInputs = optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
        CoreServices
        Security
      ]);

      postInstall = (super.postInstall or "") + ''
        mkdir -p $out/nix-support
        for f in $out/bin/*; do
          echo "file binary-dist $f" >> $out/nix-support/hydra-build-products
        done
      '';

      stripAllList = [ "bin" ];

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
