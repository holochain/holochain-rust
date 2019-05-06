let
 pkgs = import ../nixpkgs/nixpkgs.nix;
 rust = import ../rust/config.nix;
in rec
{

 path = "dist";

 version = "0.0.14-alpha1";

 normalize-artifact-target = target:
  builtins.replaceStrings
    [ "unknown" ]
    [ "generic" ]
    target
 ;

 artifact-target = normalize-artifact-target ( if pkgs.stdenv.isDarwin then rust.generic-mac-target else rust.generic-linux-target );

}
