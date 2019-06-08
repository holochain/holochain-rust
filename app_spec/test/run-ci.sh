#!/usr/bin/env bash

mkdir -p diorama-storage

STORAGE="`pwd`/diorama-storage"
rm -fr $STORAGE
mkdir $STORAGE
if [ -z $1];
then DIORAMA_STORAGE=$STORAGE node index.js | tee test.out~ | faucet || ( cat test.out~; false );
else DIORAMA_STORAGE=$STORAGE node $1;
fi;
