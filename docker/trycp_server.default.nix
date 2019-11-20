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
   minimal.holonix.pkgs.wget
   minimal.holonix.pkgs.ps
   minimal.holonix.pkgs.more
  ]
  ++ trycp_server.buildInputs
  ++ minimal.shell-config.buildInputs
  ;
 });

 holochain = {
  holochain = minimal.holochain.holochain;
 };

}
