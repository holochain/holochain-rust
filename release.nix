let
  overlay = final: previous: import ./. { pkgs = final; };

  nixpkgs = args: import ./pkgs.nix (args // {
    overlays = [ overlay ];
  });
in

with nixpkgs {};

let
  getAttrVal = name: lib.const (lib.getAttr name);

  platforms = {
    aarch64-linux-musl-cross = pkgsCross.aarch64-multiplatform-musl.pkgsStatic;
    aarch64-linux-gnu-native = nixpkgs { system = "aarch64-linux"; };
    x86_64-darwin-native = nixpkgs { system = "x86_64-darwin"; };
    x86_64-linux-musl-cross = pkgsCross.musl64.pkgsStatic;
    x86_64-linux-gnu-native = nixpkgs { system = "x86_64-linux"; };
    x86_64-windows-gnu-cross = pkgsCross.mingwW64.pkgsStatic;
  };
in

{
  holochain-cli = lib.mapAttrs (getAttrVal "holochain-cli") platforms;

  holochain-conductor = lib.mapAttrs (getAttrVal "holochain-conductor") platforms;
}
