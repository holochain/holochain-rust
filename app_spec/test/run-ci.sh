#!/usr/bin/env bash

mkdir -p diorama-storage

STORAGE="`pwd`/diorama-storage"
rm -fr $STORAGE
mkdir $STORAGE
if [ -z $1];
# We are directly pointing to the faucet executable because we can't use symlinks in vagrant on windows
then TRYORAMA_STORAGE=$STORAGE TRYORAMA_STRICT_CONDUCTOR_TIMEOUT=1 node index.js | tee test.out~ | node_modules/faucet/bin/cmd.js || ( cat test.out~; false );
else TRYORAMA_STORAGE=$STORAGE TRYORAMA_STRICT_CONDUCTOR_TIMEOUT=1 node $1;
fi;
