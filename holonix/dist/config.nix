let
 pkgs = import ../nixpkgs/nixpkgs.nix;
 rust = import ../rust/config.nix;
in
{

 path = "dist";

 version = "0.0.12-alpha1";

 artifact-target = if pkgs.stdenv.isDarwin
  then
   rust.generic-mac-target
  else
   builtins.replaceStrings
    [ "unknown" ]
    [ "generic" ]
    rust.generic-linux-target;

}
