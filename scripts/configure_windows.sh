choco install python2
choco install nodist
choco upgrade yarn
export NODE_PATH="/c/Program Files (x86)\Nodist\bin\node_modules;$NODE_PATH"
export NODIST_PREFIX="/c/Program Files (x86)\Nodist"
export NODIST_X64=1
nodist add $TRAVIS_NODE_VERSION
nodist $TRAVIS_NODE_VERSION
export PATH="/c/Program Files (x86)/Yarn/bin:$PATH"
export PATH="/c/Program Files (x86)/Nodist/bin:$PATH"
export PATH="/c/Python27:/c/Python27/Scripts:$PATH"
export PATH="/c/Users/travis/build/holochain/holochain-rust/vendor/zmq/bin:$PATH"
