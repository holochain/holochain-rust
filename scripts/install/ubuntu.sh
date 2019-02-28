#!/usr/bin/env bash

# basics
sudo apt-get update
sudo apt-get install -y cmake curl sudo git

# sodium deps
sudo apt-get install -y \
  libssl-dev \
  pkg-config \
  python2.7

# libzmq
sudo apt-get install -y \
  libzmq3-dev

# hc deps
sudo apt-get install -y qt5-default;

# nodejs_conductor deps
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
curl -sL https://deb.nodesource.com/setup_11.x | bash
sudo apt-get update && apt-get install -y nodejs yarn
npm install -g neon-cli
