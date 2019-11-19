#!/usr/bin/env bash

set -euxo pipefail

docker run -h holochain \
  -e HOST_UID \
  -v `pwd`:/holochain \
  -v $CARGO_HOME/registry:/home/holochain/.cargo/registry \
  --rm -it holochain/holochain-rust:latest "$@"
