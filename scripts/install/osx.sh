#!/usr/bin/env bash

/usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"

brew install cmake
brew install zmq
brew upgrade zeromq
brew install qt
# https://superuser.com/questions/256232/how-can-i-get-qmake-on-mac-os-x#comment1880535_422785
brew link qt

export INSTALL_NODE_VERSION=8.14

rm -rf ~/.nvm && git clone https://github.com/creationix/nvm.git ~/.nvm && (cd ~/.nvm && git checkout `git describe --abbrev=0 --tags`) && source ~/.nvm/nvm.sh && nvm install $INSTALL_NODE_VERSION
curl -o- -L https://yarnpkg.com/install.sh | bash -s -- --version 1.10.1
export PATH=$HOME/.yarn/bin:$PATH
