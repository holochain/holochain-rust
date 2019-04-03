#!/usr/bin/env bash

if [[ "$OSTYPE" == "linux-gnu" ]];
then
 `pwd`/scripts/install/linux.sh
elif [[ "$OSTYPE" == darwin* ]];
then
 `pwd`/scripts/install/osx.sh
else
 echo "auto install script not supported (does nothing) on $OSTYPE";
fi
