{ pkgs }:
let
  docker-build = pkgs.writeShellScriptBin "hc-sim2h-docker-build"
  ''
  ./docker/build sim2h_server
  '';

  docker-push = pkgs.writeShellScriptBin "hc-sim2h-docker-push"
  ''
  ./docker/push sim2h_server
  '';

  docker-run = pkgs.writeShellScriptBin "hc-sim2h-docker-run"
  ''
  ./docker/server-run sim2h_server
  '';

  docker-attach = pkgs.writeShellScriptBin "hc-sim2h-docker-attach"
  ''
  ./docker/server-attach sim2h_server
  '';
in
{
  buildInputs = [ docker-build docker-push docker-run docker-attach ];
}
