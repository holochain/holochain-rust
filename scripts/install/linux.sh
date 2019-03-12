#!/usr/bin/env bash

# basics
apt-get update
apt-get install -y sudo
sudo apt-get install -y curl git

# needed for rust_sodium-sys + neon
# https://github.com/holochain/holochain-rust/pull/1105
sudo apt-get install -y build-essential

# needed for wabt-sys
# https://circleci.com/gh/holochain/holochain-rust/10614
sudo apt-get install -y cmake

# needed for ubuntu xenial
# https://circleci.com/gh/holochain/holochain-rust/10569
# https://askubuntu.com/questions/104160/method-driver-usr-lib-apt-methods-https-could-not-be-found-update-error
sudo apt-get install -y apt-transport-https

# needed for debian stretch
# https://circleci.com/gh/holochain/holochain-rust/10566
# https://stackoverflow.com/questions/50757647/e-gnupg-gnupg2-and-gnupg1-do-not-seem-to-be-installed-but-one-of-them-is-requ
sudo apt-get install -y gnupg

sudo apt-get install -y \
  libssl-dev \
  pkg-config \
  python2.7

# hc deps
sudo apt-get install -y qt5-default;

# nodejs_conductor deps
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
curl -sL https://deb.nodesource.com/setup_11.x | bash
sudo apt-get update && sudo apt-get install -y nodejs yarn
npm install -g neon-cli
