{ pkgs }:
let
  name = "hc-conductor-wasm-install";

  version = "0.2.32";

  script = pkgs.writeShellScriptBin name
  ''
  # check if wasm-bindgen is already installed
  installed () { command -v wasm-bindgen &> /dev/null; };

  # check if wasm-bindgen has the correct version
  correct-version () { wasm-bindgen -V | grep "${version}" &> /dev/null; };

  # drop the incorrect version of wasm-bindgen
  if installed && ! correct-version;
   then
        hc-conductor-wasm-uninstall;
  fi;

  # install the correct version of wasm-bindgen
  if ! installed;
   then
     cargo install wasm-bindgen-cli --version "${version}";
  fi;

  # report the installed version
  wasm-bindgen -V;
  '';
in
{
 buildInputs = [ script ];
}
