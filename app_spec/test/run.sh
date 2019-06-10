#!/usr/bin/env bash
if [ -z $1];
then node index.js | tee test.out~ | faucet || ( cat test.out~; false );
else node $1;
fi;
