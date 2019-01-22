#!/usr/bin/env bash

if [[ "$OSTYPE" == "linux-gnu" ]];
then
 . `pwd`/scripts/install/release-deps-ubuntu.sh
elif [[ "$OSTYPE" == darwin* ]];
then
 . `pwd`/scripts/install/release-deps-osx.sh
fi
