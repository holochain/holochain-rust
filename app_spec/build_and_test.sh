#!/usr/bin/env bash
set -euo pipefail
mkdir -p dist

echo "===================================================================================="
echo "BUILDING genome with 'hc package --output dist/app_spec.dna.json --strip-meta':"
echo "------------------------------------------------------------------------------------"

rm -rf dist
mkdir dist
hc package --output dist/app_spec.dna.json --strip-meta

echo "DONE."
echo "===================================================================================="
echo "Running test.js in node"
echo "------------------------------------------------------------------------------------"

cd test
# --no-bin-links is required for windows vagrant support
# more precisely symlinks are not supported without additional work on the host
# e.g. https://superuser.com/questions/1115329/vagrant-shared-folder-and-symbolic-links-under-windows-10
npm install --no-bin-links
npm run test-ci
