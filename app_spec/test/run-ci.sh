#!/usr/bin/env bash

mkdir -p diorama-storage

STORAGE="`pwd`/diorama-storage"
rm -fr $STORAGE
mkdir $STORAGE
if [ -z $1];
# We are directly pointing to the faucet executable because we can't use symlinks in vagrant on windows
then DIORAMA_STORAGE=$STORAGE node index.js | tee test.out~ | node_modules/faucet/bin/cmd.js || ( cat test.out~; false );
else DIORAMA_STORAGE=$STORAGE node $1;
fi;
