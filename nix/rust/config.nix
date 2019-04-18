{

  # the mozilla rust overlay
  # allows us to track cargo nightlies in a nixos friendly way
  # avoids rustup
  # not compatible with parallel rustup installation
  moz-overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);

  # our rust nightly version
  nightly-date = "2019-01-24";

  # the target used by rust when compiling wasm
  wasm-target = "wasm32-unknown-unknown";


}
