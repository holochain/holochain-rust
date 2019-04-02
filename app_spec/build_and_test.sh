#! /bin/bash
mkdir -p dist
echo "===================================================================================="
echo "RUNNING cargo test for zomes"
echo "------------------------------------------------------------------------------------"
cargo test --manifest-path zomes/blog/code/Cargo.toml
cargo test --manifest-path zomes/summer/code/Cargo.toml
echo "===================================================================================="
echo "BUILDING genome with 'hc package --output dist/app_spec.dna.json --strip-meta':"
echo "------------------------------------------------------------------------------------"
rm -f dist/app_spec.dna.json
hc package --output dist/app_spec.dna.json --strip-meta
echo "DONE."
echo "===================================================================================="
echo "Running test.js in node"
echo "------------------------------------------------------------------------------------"
cd test
npm install
npm test
