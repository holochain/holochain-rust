#!/usr/bin/env bash

# basics
apt-get update
apt-get install -y cmake curl sudo git

# needed for ubuntu xenial
# https://circleci.com/gh/holochain/holochain-rust/10569
# https://askubuntu.com/questions/104160/method-driver-usr-lib-apt-methods-https-could-not-be-found-update-error
sudo apt-get install -y apt-transport-https

# needed for debian stretch
# https://circleci.com/gh/holochain/holochain-rust/10566
# https://stackoverflow.com/questions/50757647/e-gnupg-gnupg2-and-gnupg1-do-not-seem-to-be-installed-but-one-of-them-is-requ
sudo apt-get install -y gnupg

apt-get install -y \
  libssl-dev \
  pkg-config \
  python2.7

# hc deps
apt-get install -y qt5-default;

# nodejs_conductor deps
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
curl -sL https://deb.nodesource.com/setup_11.x | bash
apt-get update && apt-get install -y nodejs yarn
npm install -g neon-cli
