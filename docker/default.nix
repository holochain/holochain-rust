{ pkgs }:
let
  docker-build = pkgs.writeShellScriptBin "hc-docker-build"
  ''
  docker build . -f Dockerfile -t holochain/holochain-rust:latest
  '';

  docker-push = pkgs.writeShellScriptBin "hc-docker-pus"
  ''
  docker push -t holochain/holochain-rust:latest
  '';

  docker-update-all = pkgs.writeShellScriptBin "hc-docker-update-all"
  ''
  hc-docker-build
  hc-docker-push

  hc-trycp-docker-build
  hc-trycp-docker-push
  '';
in
{
  buildInputs = [ docker-build docker-push docker-update-all ];
}
