{ pkgs }:
let
  docker-build = pkgs.writeShellScriptBin "hc-trycp-docker-build"
  ''
  ./docker/build trycp_server
  '';

  docker-push = pkgs.writeShellScriptBin "hc-trycp-docker-push"
  ''
  ./docker/push trycp_server
  '';

  docker-run = pkgs.writeShellScriptBin "hc-trycp-docker-run"
  ''
  ./docker/server-run trycp_server
  '';

  docker-attach = pkgs.writeShellScriptBin "hc-trycp-docker-attach"
  ''
  ./docker/server-attach trycp_server
  '';
in
{
  buildInputs = [ docker-build docker-push docker-run docker-attach ];
}
