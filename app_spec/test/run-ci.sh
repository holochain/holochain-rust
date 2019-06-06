#!/usr/bin/env bash

mkdir diorama-storage

if [ -z $1];
then DIORAMA_STORAGE="`pwd`/diorama-storage" node index.js | tee test.out~ | faucet || ( cat test.out~; false );
else DIORAMA_STORAGE="`pwd`/diorama-storage" node $1;
fi;

# rm -fr diorama-storage