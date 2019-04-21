let
  pkgs = import ../nixpkgs/nixpkgs.nix;
  
  frameworks = if pkgs.stdenv.isDarwin then pkgs.darwin.apple_sdk.frameworks else {};
in
{

 ld-flags = if pkgs.stdenv.isDarwin then "-F${frameworks.CoreFoundation}/Library/Frameworks -framework CoreFoundation " else "";

}
