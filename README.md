# Holochain-rust

<a href="http://holochain.org"><img align="right" width="200" src="https://github.com/holochain/org/blob/master/logo/holochain_logo.png?raw=true" alt="holochain logo" /></a>

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.org)

[![Twitter Follow](https://img.shields.io/twitter/follow/holochain.svg?style=social&label=Follow)](https://twitter.com/holochain)

Travis: [![Build Status](https://travis-ci.com/holochain/holochain-rust.svg?branch=master)](https://travis-ci.com/holochain/holochain-rust)
Circle CI: [![CircleCI](https://circleci.com/gh/holochain/holochain-rust.svg?style=svg)](https://circleci.com/gh/holochain/holochain-rust)
Codecov: [![Codecov](https://img.shields.io/codecov/c/github/holochain/holochain-rust.svg)](https://codecov.io/gh/holochain/holochain-rust/branch/master)
License: [![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

This is the home of the Holochain Rust libraries.

This code is loosely based on the [Golang prototype](https://github.com/holochain/holochain-proto).

**Code Status:** Rust version is alpha. Not for production use. The code is guaranteed NOT secure. We will aggressively restructure code APIs and data chains until Beta.

[Releases](https://github.com/holochain/holochain-rust/releases) happen weekly.
<br/>

| Holochain Links: | [FAQ](https://developer.holochain.org/guide/latest/faq.html) | [Developer Docs](https://developer.holochain.org) | [White Paper](https://github.com/holochain/holochain-proto/blob/whitepaper/holochain.pdf) |
|---|---|---|---|

## Overview

[Holochain-Rust Architectural Overview](./doc/architecture/README.md)

## Documentation: The Book on Holochain
There is a work-in-progress book of documentation being written about `holochain-rust`. See the published version at the associated GitHub Pages for this repo, [https://developer.holochain.org/guide/latest](https://developer.holochain.org/guide/latest). See instructions for how to contribute to the book at [doc/holochain_101/src/how_to_contribute.md](./doc/holochain_101/src/how_to_contribute.md).

## Installation & Usage

**Important:** the instructions in this readme are for developers intending work on Holochain code-base itself, not Holochain application developers.  If you want to use Holochain, please proceed to the instructions on the quick start installation guide: **https://developer.holochain.org/start.html**

**The following instructions are for developing Holochain Core or the HDK itself**

There are two components needed currently to run Holochain applications, the core (what's in this repo) and also [the networking engine](https://github.com/holochain/n3h).  You can install and work on core using the built-in mock network following the instructions below, but if you want to actually test out your apps using the real networking, you will have to install [the networking component](https://github.com/holochain/n3h) following the instructions in the README there.  (Note: please see the instructions in the guide book for [`hc`](https://developer.holochain.org/guide/latest/hc_configuring_networking.html) or the [production Conductor](https://developer.holochain.org/guide/latest/conductor_networking.html) for how to configure the tools to use and activate the networking component.

There are three approaches to building and testing Holochain: using `nix-shell`, `make`, `docker`:

### Nix Shell (Supported: Ubuntu, Debian, Mac OS X & Nix OS)

The `nix-shell` command from the nixos team is the preferred way to work with Holochain.

NixOS is an entire operating system but the `nix-shell` is simply a tool to manage dependencies for an individual shell session.

To install `nix-shell`:

```shell
# basic deps needed on ubuntu/debian
apt-get update && apt-get install -y curl bzip2

# this installs on all (non-windows) systems
curl https://nixos.org/nix/install | sh
```

Follow any further instructions output to the terminal during installation.

[The website](https://nixos.org/nix/download.html) has more details.

Running the `nix-shell` command from inside the root of the repository will detect and use the `shell.nix` file.

The `nix-shell` approach offers unique benefits:

- Dependencies are injected into a single shell session only
  - Minimal modifications to the host environment
  - No need to maintain/rerun/troubleshoot installation scripts
  - Further isolation from host environment can be achieved with `nix-shell --pure`
- Dependencies are hashed
  - "Dependency hell" is avoided
  - Nice parallels with the hashed zomes model
  - Security + reliability benefits
- Dependencies can be garbage collected with the `nix-collect-garbage` command
- Single "package manager" across most operating systems
- Ability to ship utility scripts in the `shell.nix` file
- Access to the nix functional programming language for dependencies/script management
- NixOS runs on HoloPorts so `nix-shell` provides similar behaviour/environment

If you have a nix friendly system, this is the fastest and most reliable way to develop and test.

Once in a `nix-shell` the `hc-*` prefixed bash commands support tab completion.

Note: The `hc-test-all` command builds and tests _everything_ in core.

Note: The `hc-install-*` commands may write to the current user's home directory.
Other commands that call `cargo` and `npm` may also write to the home directory.
This is how `cargo` and `npm` work unfortunately.
`hc-**-flush` commands delete relevant development artifacts.

#### Troubleshooting

Default `nix-shell` behaviour preserves some of the user's environment, simply
_adding_ to it rather than _isolating_ from it.

This can cause problems if your user has cruft that conflicts with what nix is
doing, e.g. existing `cargo` or `npm` installations/environment variables.

If you are seeing an issue in `nix-shell` that others are not seeing, try using
our isolation script `./scripts/nix/pod.sh` to debug the command.

For example:

```shell
./scripts/nix/pod.sh 'hc-build-wasm && hc-test'
```

or even:

```shell
./scripts/nix/pod.sh hc-test-all
```

#### Future deployments

In the future we plan to distribute binaries through nixpkgs.
This would to enable the following:

```shell
# doesn't work yet... watch this space!
nix-shell -p holochain --run holochain ...
```

### Make (Supported: Ubuntu, Debian & Mac OS X)

For Linux/OSX you can install the prerequisites directly into the host environment with:

``` shell
cd path/to/holochain
. ./scripts/install/auto.sh
```

**Note**: the script will install [homebrew](https://brew.sh/) on mac os x
**Note**: the script will install dependencies with `apt-get` on linux

After the install script completes successfully, you can start local development using `make`

Running the `make` command will:

1. install (or update to) the correct version of rust
2. build all the rust libraries from the source code in this repository.
3. build and install the command-line tools.

**Note**: it's very important to use the rust version specified in the Makefile! Since we are using nightly rust builds, the language is changing rapidly and sometimes introduces breaking changes that we haven't adapted to yet. Don't just use the latest nightly.

**Note**: The installation script evolves over time alongside core.
The installation script is idempotent.
Rerun the script after each upgrade/downgrade.

### Docker (Supported: Ubuntu, Debian, Mac OS X, Nix OS, Windows)

We support [docker](https://www.docker.com/).
The `docker` folder contains scripts to build and run docker images.

The `holochain/holochain-rust:latest` docker image is an alpine NixOS rebuilt nightly.
The build process warms nix and incrementally compiles cargo/wasm/neon for faster feedback.

### Windows

You will need to install rust manually.

Rustup `https://rustup.rs/#` is likely the best option.

The rust language moves very fast on the nightly channel.

It is very important to be using the correct nightly version.

Currently this is:

`nightly-2019-01-24-x86_64-pc-windows-msvc`

The nightly version we test/develop against can always be found in the .travis.yml file.

#### Running tests

Run:

```shell
. docker/run-test
```
or

``` shell
make test
```

or

``` shell
nix-shell --run hc-test
```

Note that there are also make commands for running the tests of just core, or the command-line line tools or app_spec separately:

``` shell
make test_cli
make test_holochain
make test_app_spec
make build_nodejs_conductor
```

### Building for Android
Note there is an article written on how to build Holochain for Android, read it [here](doc/holochain_101/src/building_for_android.md).

## Upgrading

Upgrading to a new tagged release of Holochain may include new/changed system dependencies.

__If not using `nix-shell` we strongly recommend rerunning `./scripts/install/auto.sh` when upgrading core.__

The script is designed to be idempotent. This means there is no harm re-running it.

## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](/CONTRIBUTING.md) for our general practices and protocols on participating in the community, as well as specific expectations around things like code formatting, testing practices, continuous integration, etc.

Some helpful links:

* Chat with us on our [Chat Server](https://chat.holochain.org) or [Gitter](https://gitter.im/metacurrency/holochain)


## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2018, Holochain Foundation

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
