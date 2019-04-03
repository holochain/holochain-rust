#!/usr/bin/env bash

# Check if the script is run by the root user
if [ "$EUID" -ne 0 ]
then
	# Script is run by root, so we do not need `sudo`
	as_root=""
else
	# Check if `sudo` is installed
	which sudo > /dev/null
	if [ $? -eq 0 ]
	then
		# `sudo` is installed; use it
		as_root="sudo"
	else
		echo "This script requires `sudo` or root priviledges" 1>&2
		exit
	fi
fi

# basics
$as_root apt-get install -y curl git

# needed for rust_sodium-sys + neon
# https://github.com/holochain/holochain-rust/pull/1105
$as_root apt-get install -y build-essential

# needed for wabt-sys
# https://circleci.com/gh/holochain/holochain-rust/10614
$as_root apt-get install -y cmake

# needed for ubuntu xenial
# https://circleci.com/gh/holochain/holochain-rust/10569
# https://askubuntu.com/questions/104160/method-driver-usr-lib-apt-methods-https-could-not-be-found-update-error
$as_root apt-get install -y apt-transport-https

# needed for debian stretch
# https://circleci.com/gh/holochain/holochain-rust/10566
# https://stackoverflow.com/questions/50757647/e-gnupg-gnupg2-and-gnupg1-do-not-seem-to-be-installed-but-one-of-them-is-requ
$as_root apt-get install -y gnupg

$as_root apt-get install -y \
  libssl-dev \
  pkg-config \
  python2.7

# hc deps
$as_root apt-get install -y qt5-default

# nodejs_conductor deps
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | $as_root apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | $as_root tee /etc/apt/sources.list.d/yarn.list
curl -sL https://deb.nodesource.com/setup_11.x | $as_root bash
$as_root apt-get update && $as_root apt-get install -y nodejs yarn
$as_root npm install -g neon-cli

