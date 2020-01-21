#!/bin/bash
CRATE=$1
REPO=$2
BRANCH=$3
ROOT=`pwd`
cd $ROOT/crates
dirs=`ls`
for d in $dirs
do 
    cd $d 
if grep "$CRATE" Cargo.toml > /dev/null; then
    cargo-add add $CRATE --git $REPO --branch $BRANCH
fi
cd ..
done
cd $ROOT
