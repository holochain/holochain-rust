#!/usr/bin/env bash
# apt-get update && apt-get install -y \
#   cmake \
#   curl \
#   sudo \
#   pkg-config \
#   libssl1.0-dev \
#   libzmq3-dev \
#   python2.7 \
#   qt5-default \

apt-get update && \
apt-get install -y \
  git \
  build-essential \
  libssl-dev \
  curl
