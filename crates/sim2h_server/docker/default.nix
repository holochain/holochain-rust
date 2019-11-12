{ pkgs }:
let
  docker-build = pkgs.writeShellScriptBin "hc-sim2h-docker-build"
  ''
  docker build crates/sim2h_server/docker -f crates/sim2h_server/docker/Dockerfile.sim2h -t holochain/sim2h_server:latest
  # docker build . -f crates/sim2h_server/docker/Dockerfile.sim2h -t holochain/sim2h_server:latest
  '';

  docker-push = pkgs.writeShellScriptBin "hc-sim2h-docker-push"
  ''
  docker push holochain/sim2h_server:latest
  '';

  docker-run = pkgs.writeShellScriptBin "hc-sim2h-docker-run"
  ''
  docker run --rm -d -p 443:443/tcp --name sim2h-server -t holochain/sim2h_server:latest
  '';

  docker-attach = pkgs.writeShellScriptBin "hc-sim2h-docker-attach"
  ''
  docker attach sim2h-server
  '';
in
{
  buildInputs = [ docker-build docker-push docker-run docker-attach ];
}
