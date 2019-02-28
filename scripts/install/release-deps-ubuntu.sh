#!/usr/bin/env bash

# basics
sudo apt-get update
sudo apt-get install -y cmake curl

# sodium deps
sudo apt-get install -y \
  libssl-dev \
  pkg-config \
  python2.7
