choco install nodist
choco upgrade yarn
export PATH="/c/Program Files (x86)/Yarn/bin:/c/Program Files (x86)/Nodist/bin:$PATH"
export NODE_PATH="/c/Program Files (x86)\Nodist\bin\node_modules;$NODE_PATH"
export NODIST_PREFIX="/c/Program Files (x86)\Nodist"
export NODIST_X64=1
nodist add $TRAVIS_NODE_VERSION
nodist $TRAVIS_NODE_VERSION
node -e "console.log(process.argv[0], process.arch, process.versions)"
export PATH=$PATH:/c/Users/travis/build/holochain/holochain-rust/vendor/zmq/bin
# deps for neon, found at https://guides.neon-bindings.com/getting-started/
npm install --scripts-prepend-node-path=true --global --vs2015 --production windows-build-tools
yarn config set python python2.7
npm config set msvs_version 2015
yarn config set msvs_version 2015 --global
rustup target add wasm32-unknown-unknown --toolchain nightly-2018-10-12-x86_64-pc-windows-msvc