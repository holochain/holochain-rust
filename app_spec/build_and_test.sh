#! /bin/bash
mkdir dist
echo "===================================================================================="
echo "RUNNING cargo test for zomes"
echo "------------------------------------------------------------------------------------"
cargo test --manifest-path zomes/blog/code/Cargo.toml
cargo test --manifest-path zomes/summer/code/Cargo.toml
echo "===================================================================================="
echo "BUILDING genome with 'hc package --output dist/app_spec.hcpkg --strip-meta':"
echo "------------------------------------------------------------------------------------"
rm dist/app_spec.hcpkg
hc package --output dist/app_spec.hcpkg --strip-meta
echo "DONE."
echo "===================================================================================="
echo "Running test.js in node"
echo "------------------------------------------------------------------------------------"
cd test
npm install
npm test
