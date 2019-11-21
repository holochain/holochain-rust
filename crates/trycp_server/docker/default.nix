{ pkgs }:
let
  docker-build = pkgs.writeShellScriptBin "hc-trycp-docker-build"
  ''
  docker build crates/trycp_server/docker -f crates/trycp_server/docker/Dockerfile.trycp -t holochain/holochain-rust:trycp
  # docker build . -f crates/trycp_server/docker/Dockerfile.trycp -t holochain/holochain-rust:trycp
  '';

  docker-push = pkgs.writeShellScriptBin "hc-trycp-docker-push"
  ''
  docker push holochain/holochain-rust:trycp
  '';

  docker-run = pkgs.writeShellScriptBin "hc-trycp-docker-run"
  ''
  docker run --rm -d -p 443:443/tcp --name holochain-trycp -t holochain/holochain-rust:trycp
  '';

  docker-attach = pkgs.writeShellScriptBin "hc-trycp-docker-attach"
  ''
  docker attach holochain-trycp
  '';
in
{
  buildInputs = [ docker-build docker-push docker-run docker-attach ];
}
