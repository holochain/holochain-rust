FROM nixorg/nix:circleci

ENV NIX_PATH nixpkgs=https://github.com/NixOS/nixpkgs/archive/47f008676fe3b77b9c8edda54db621a0dc16dd8e.tar.gz
ENV HC_TARGET_PREFIX /tmp/holochain/

WORKDIR /holochain-rust/build
COPY . .
