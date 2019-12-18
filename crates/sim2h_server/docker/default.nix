{ pkgs }:
let
  git-branch = "`git rev-parse --abbrev-ref HEAD`";

  docker-build = pkgs.writeShellScriptBin "hc-sim2h-docker-build"
  ''
  set -euxo pipefail
  for image in minimal sim2h_server
  do
  ./docker/build $image ${git-branch}
  done
  '';

  docker-push = pkgs.writeShellScriptBin "hc-sim2h-docker-push"
  ''
  set -euxo pipefail
  for image in minimal sim2h_server
  do
  ./docker/push $image ${git-branch}
  done
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
