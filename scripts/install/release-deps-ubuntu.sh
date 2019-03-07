#!/usr/bin/env bash

# basics
echo "deb http://archive.ubuntu.com/ubuntu/ bionic main restricted" | sudo tee -a /etc/apt/sources.list
echo "deb http://archive.ubuntu.com/ubuntu/ bionic-updates main restricted" | sudo tee -a /etc/apt/sources.list
echo "deb http://archive.ubuntu.com/ubuntu/ bionic universe" | sudo tee -a /etc/apt/sources.list
echo "deb http://archive.ubuntu.com/ubuntu/ bionic-updates universe" | sudo tee -a /etc/apt/sources.list
echo "deb http://archive.ubuntu.com/ubuntu/ bionic multiverse" | sudo tee -a /etc/apt/sources.list
echo "deb http://archive.ubuntu.com/ubuntu/ bionic-updates multiverse" | sudo tee -a /etc/apt/sources.list
echo "deb http://archive.ubuntu.com/ubuntu/ bionic-backports main restricted universe multiverse" | sudo tee -a /etc/apt/sources.list
echo "deb http://security.ubuntu.com/ubuntu bionic-security main restricted" | sudo tee -a /etc/apt/sources.list
echo "deb http://security.ubuntu.com/ubuntu bionic-security universe" | sudo tee -a /etc/apt/sources.list
echo "deb http://security.ubuntu.com/ubuntu bionic-security multiverse" | sudo tee -a /etc/apt/sources.list
sudo apt-get update
sudo apt-get install -y libsodium23
sudo apt-get install -y cmake curl

# sodium deps
sudo apt-get install -y \
  libssl-dev \
  pkg-config \
  python2.7
