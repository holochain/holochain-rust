let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;
  release = import ../../config.nix;

  name = "hc-release-github-check-artifacts";

  script = pkgs.writeShellScriptBin name
  ''
  echo
  echo "Checking core artifacts"
  echo

  echo
  echo "checking ${release.core.tag}"
  echo

  core_binaries=( "cli" "conductor" )
  core_platforms=( "apple-darwin" "pc-windows-gnu" "pc-windows-msvc" "unknown-linux-gnu" )

  for binary in "''${core_binaries[@]}"; do
   for platform in "''${core_platforms[@]}"; do
    file="$binary-${release.core.tag}-x86_64-$platform.tar.gz"
    url="https://github.com/holochain/holochain-rust/releases/download/${release.core.tag}/$file"
    echo
    echo "pinging $file for release $release..."
    if curl -Is "$url" | grep -q "HTTP/1.1 302 Found"
     then echo "FOUND ✔"
     else echo "NOT FOUND ⨯"
    fi
    echo
   done
  done

  echo
  echo "Checking node conductor artifacts"
  echo

  echo
  echo "checking ${release.node-conductor.tag}"
  echo

  node_versions=( "57" "64" "67" )
  conductor_platforms=( "darwin" "linux" "win32" )

  for node_version in "''${node_versions[@]}"; do
   for platform in "''${conductor_platforms[@]}"; do
    file="index-v${release.node-conductor.version.current}-node-v''${node_version}-''${platform}-x64.tar.gz"
    url="https://github.com/holochain/holochain-rust/releases/download/${release.node-conductor.tag}/$file"
    echo
    echo "pinging $file for release $release..."
    if curl -Is "$url" | grep -q "HTTP/1.1 302 Found"
     then echo "FOUND ✔"
     else echo "NOT FOUND ⨯"
    fi
    echo
   done
  done
  '';
in
script
