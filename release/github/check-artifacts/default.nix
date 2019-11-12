{ pkgs, config }:
let

  name = "hc-release-github-check-artifacts";

  script = pkgs.writeShellScriptBin name
  ''
  echo
  echo "Checking core artifacts"
  echo

  echo
  echo "checking ${config.release.tag}"
  echo

  core_binaries=( "cli" "holochain" "sim2h_server" "trycp_server" )
  core_platforms=( "apple-darwin" "generic-linux-gnu" )

  for binary in "''${core_binaries[@]}"; do for platform in "''${core_platforms[@]}"; do file="$binary-${config.release.tag}-x86_64-$platform.tar.gz"
    url="https://github.com/holochain/holochain-rust/releases/download/${config.release.tag}/$file"
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
{
 buildInputs = [ script ];
}
