let
  pkgs = import ../nixpkgs/nixpkgs.nix;
in
{

 ld-flags = if pkgs.stdenv.isDarwin then "-F${pkgs.frameworks.CoreFoundation}/Library/Frameworks -framework CoreFoundation " else "";

}
