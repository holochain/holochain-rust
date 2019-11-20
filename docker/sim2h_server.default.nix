let
 minimal = import ./minimal.default.nix;
 sim2h_server = minimal.holonix.pkgs.callPackage ../crates/sim2h_server/default.nix {
  pkgs = minimal.holonix.pkgs;
  config = { };
 };
in
{
 dev-shell = minimal.holonix.pkgs.stdenv.mkDerivation(minimal.shell-config // {
  buildInputs = [
   minimal.holonix.pkgs.wget
  ]
  ++ sim2h_server.buildInputs
  ++ minimal.shell-config.buildInputs
  ;
 });

 holochain = {
  holochain = minimal.holochain.holochain;
 };

}
