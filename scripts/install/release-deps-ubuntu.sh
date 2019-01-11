#!/usr/bin/env bash

# basics
apt-get update
apt-get install -y cmake curl sudo

# sodium deps
apt-get install -y \
  libssl-dev \
  pkg-config \
  python2.7

# libzmq
apt-get install -y \
  libzmq3-dev
