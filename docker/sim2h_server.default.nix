let
 minimal = import ./minimal.default.nix;
in
{
 dev-shell = minimal.holonix.pkgs.stdenv.mkDerivation(minimal.shell-config // {
  buildInputs = [
   minimal.holonix.pkgs.wget
  ]
  ++ minimal.shell-config.buildInputs
  ;
 });

 holochain = {
  holochain = minimal.holochain.holochain;
 };

}
