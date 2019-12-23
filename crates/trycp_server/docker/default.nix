{ pkgs }:
let

  git-branch = "`git rev-parse --abbrev-ref HEAD`";

  docker-build = pkgs.writeShellScriptBin "hc-trycp-docker-build"
  ''
  set -euxo pipefail
  for image in minimal trycp_server
  do
  ./docker/build $image ${git-branch}
  done
  '';

  docker-push = pkgs.writeShellScriptBin "hc-trycp-docker-push"
  ''
  set -euxo pipefail
  for image in minimal trycp_server
  do
  ./docker/push $image ${git-branch}
  done
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
