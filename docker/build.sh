#!/usr/bin/env bash

set -euxo pipefail

docker build ./docker -f ./docker/Dockerfile.${1} -t "holochain/holochain-rust:${1}" --no-cache
