let
  overlay = final: previous: import ./. { pkgs = final; };

  nixpkgs = args: import ./pkgs.nix (args // {
    overlays = [ overlay ];
  });
in

with nixpkgs {};

let
  # nativeRelease = import "${pkgs.path}/pkgs/top-level/release-lib.nix" {
  #   nixpkgsArgs.overlays = [ overlay ];
  #   supportedSystems = [ "aarch64-linux" "x86_64-linux" ];
  # };

  platforms = {
    aarch64-linux = pkgsCross.aarch64-multiplatform-musl.pkgsStatic;
    x86_64-linux = pkgsCross.musl64.pkgsStatic;
    x86_64-linux-native = nixpkgs { system = "x86_64-linux"; };
    x86_64-windows = pkgsCross.mingwW64.pkgsStatic;
  };
in

{
  holochain-cli = lib.mapAttrs (lib.const (lib.getAttr "holochain-cli")) platforms;

  holochain-conductor = lib.mapAttrs (lib.const (lib.getAttr "holochain-conductor")) platforms;

  # native = with nativeRelease;
  #   mapTestOn (packagePlatforms (lib.getAttrs [ "holochain-cli" "holochain-conductor" ] pkgs));
}
