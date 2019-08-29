let
  overlay = final: previous: import ./. { pkgs = final; };
in

with import ./pkgs.nix { overlays = [ overlay ]; };

let
  intoRelease = drv: _double: pkgs: pkgs."${drv}".overrideAttrs (super: {
    buildType = "release";

    postInstall = (super.postInstall or "") + ''
      mkdir -p $out/nix-support
      for f in $out/bin/*; do
        echo "file binary-dist $f" >> $out/nix-support/hydra-build-products
      done
    '';

    stripAllList = [ "bin" ];
  });

  platforms = {
    aarch64-linux = pkgsCross.aarch64-multiplatform-musl.pkgsStatic;
    x86_64-linux = pkgsCross.musl64.pkgsStatic;
    x86_64-windows = pkgsCross.mingwW64.pkgsStatic;
  };
in

{
  holochain-cli = lib.mapAttrs (intoRelease "holochain-cli") platforms;

  holochain-conductor = lib.mapAttrs (intoRelease "holochain-conductor") platforms;
}
