let
 minimal = import ./minimal.default.nix;
 trycp_server = minimal.holonix.pkgs.callPackage ../crates/trycp_server/default.nix {
  pkgs = minimal.holonix.pkgs;
  config = { };
 };
in
{
 dev-shell = minimal.holonix.pkgs.stdenv.mkDerivation(minimal.shell-config // {
  buildInputs = [
  ]
  ++ trycp_server.buildInputs
  ++ minimal.shell-config.buildInputs
  ;
 });

 holochain = {
  holochain = minimal.holochain.holochain;
 };

}
