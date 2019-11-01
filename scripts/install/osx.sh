#!/usr/bin/env bash

/usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"

brew install cmake
brew install qt5
# https://superuser.com/questions/256232/how-can-i-get-qmake-on-mac-os-x#comment1880535_422785
# https://superuser.com/a/1153338
brew link qt5 --force

export INSTALL_NODE_VERSION=8.14.0

rm -rf ~/.nvm && git clone https://github.com/creationix/nvm.git ~/.nvm && (cd ~/.nvm && git checkout `git describe --abbrev=0 --tags`) && source ~/.nvm/nvm.sh && nvm install $INSTALL_NODE_VERSION
curl -o- -L https://yarnpkg.com/install.sh | bash -s -- --version 1.10.1
export PATH=$HOME/.yarn/bin:$PATH
npm install -g neon-cli

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly-2019-07-14 -y
source $HOME/.cargo/env
