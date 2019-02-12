#!/usr/bin/env bash

# basics
sudo apt-get update
sudo apt-get install -y cmake curl

# sodium deps
sudo apt-get install -y \
  libssl-dev \
  pkg-config \
  python2.7

# libzmq
sudo apt-get install -y \
  libzmq3-dev