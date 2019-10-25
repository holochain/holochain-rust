{ pkgs }:
let
  docker-build = pkgs.writeShellScriptBin "hc-trycp-docker-build"
  ''
  docker build . -f crates/trycp_server/docker/Dockerfile.trycp -t holochain/holochain-rust:trycp
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
  buildInputs = [ docker-build docker-run ];
}
