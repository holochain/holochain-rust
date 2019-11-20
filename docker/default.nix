{ pkgs }:
let
  docker-build = pkgs.writeShellScriptBin "hc-docker-build"
  ''
  ./docker/build.sh ''${1}
  '';

  docker-login = pkgs.writeShellScriptBin "hc-docker-login"
  ''
  ./docker/login.sh ''${1}
  '';

  docker-push = pkgs.writeShellScriptBin "hc-docker-push"
  ''
  ./docker/push.sh ''${1}
  '';
in
{
  buildInputs = [ docker-build docker-login docker-push ];
}
