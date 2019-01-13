#!/usr/bin/env bash

/usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"

brew install zmq

export INSTALL_NODE_VERSION=8.4

rm -rf ~/.nvm && git clone https://github.com/creationix/nvm.git ~/.nvm && (cd ~/.nvm && git checkout `git describe --abbrev=0 --tags`) && source ~/.nvm/nvm.sh && nvm install $INSTALL_NODE_VERSION
curl -o- -L https://yarnpkg.com/install.sh | bash -s -- --version 1.10.1
export PATH=$HOME/.yarn/bin:$PATH
