#!/usr/bin/env bash

# default nix-shell behaviour preserves some of the user's environment
# this can be problematic when debugging nix behaviour on dirty systems
#
# this is surprising if you expect `nix-shell` to provide complete isolation
# from the host environment and work exactly like a full nixos install
#
# try `nixos-container` which does provide complete isolation to see why you
# may not want that, e.g.:
#
# - no internet connection
# - no configured devices/users
# - no access to files on host system
#
# @see https://nixos.org/nixos/manual/#ch-containers
#
# that said, `nix-shell` can be configured to provide a lot more isolation than
# the default configuration
#
# @see https://nixos.org/nix/manual/#options-1
# @see https://github.com/NixOS/nix/issues/903
# @see https://github.com/NixOS/nix/issues/903#issuecomment-460331573
#
# if a nix command is breaking on your machine but working elsewhere try
# passing it to our "pod" script, e.g.:
#
# `./nix/pod.sh hc-rust-test`
#
# if the pod works where a normal `nix-shell` does not, chances are that you
# have some incompatible user config being sourced/executed by `nix-shell`
#
# common culprits:
#
# - hardcoded $PATH to existing rust installations or other libs
# - setting environment variables that cargo/npm read
# - other items in ~/.bashrc, /etc/bashrc, $HOME, $USER, $DISPLAY
#
PS1="" nix-shell --pure --keep PS1 --run "$1"
