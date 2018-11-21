#! /bin/bash
mkdir dist
echo "===================================================================================="
echo "BUILDING genome with 'hc package --output dist/app-spec-rust.hcpkg --strip-meta':"
echo "------------------------------------------------------------------------------------"
hc package --output dist/app-spec-rust.hcpkg --strip-meta
echo "DONE."
echo "===================================================================================="
echo "Running test.js in node"
echo "------------------------------------------------------------------------------------"
cd test
npm install
node test.js | node_modules/faucet/bin/cmd.js
