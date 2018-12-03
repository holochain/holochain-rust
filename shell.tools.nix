# This imports the nix package collection,
# so we can access the `pkgs` and `stdenv` variables
let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };

  date = "2018-07-17";

  rust-build = (nixpkgs.rustChannelOf {channel = "nightly"; date = date;}).rust;

  hc-fmt = nixpkgs.writeShellScriptBin "hc-fmt" "cargo fmt";
  hc-fmt-check = nixpkgs.writeShellScriptBin "hc-fmt-check" "cargo fmt -- --check";

  # nix-shell on mac os x:
  # https://stackoverflow.com/questions/51161225/how-can-i-make-macos-frameworks-available-to-clang-in-a-nix-environment
  frameworks = nixpkgs.darwin.apple_sdk.frameworks;
in
with nixpkgs;
stdenv.mkDerivation rec {
  name = "holochain-tools-environment";

  buildInputs = [
    rust-build
    rustup

    hc-fmt
    hc-fmt-check

    # mac os x
    frameworks.Security
    frameworks.CoreFoundation
    frameworks.CoreServices
  ];

  # mac os x
  shellHook = ''
      export PS1="[$name] \[$txtgrn\]\u@\h\[$txtwht\]:\[$bldpur\]\w \[$txtcyn\]\$git_branch\[$txtred\]\$git_dirty \[$bldylw\]\$aws_env\[$txtrst\]\$ "
      export NIX_LDFLAGS="-F${frameworks.CoreFoundation}/Library/Frameworks -framework CoreFoundation $NIX_LDFLAGS";
  '';
}
