#!/usr/bin/env bash

cd nodejs_conductor
yarn install --ignore-scripts
RUST_SODIUM_DISABLE_PIE=1 node ./publish.js
