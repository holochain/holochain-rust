# Holochain Rust Conductor

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat-square)](https://waffle.io/holochain/org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.net)

This crate implements a reference Conductor for serving Holochain DNAs.

## Install

Our recommended pattern for the installation of the conductor is to download the binary for your platform from our [releases](https://github.com/holochain/holochain-rust/releases) page. Otherwise, you can proceed with the more complex instructions for building from source, below.  Note, on Mac and Linux as well as installing the binaries you will need to install the `zmq` dependency e.g.:

On MacOS:

```
brew install zmq
```

On Ubuntu:

```
apt-get install libzmq3-dev
```


### Building From Source

These dependencies need to be installed in order to compile, and use `holochain`:

- [Rust](https://www.rust-lang.org/en-US/install.html)
  - needs to be the `nightly-2019-01-24` build, so use the following commands, once you have first installed Rust
  - `rustup toolchain install nightly-2019-01-24`
  - `rustup default nightly-2019-01-24`
  - Also, if you are going to be developing Zomes in Rust, install the WASM build target for Rust, by running:
  - `rustup target add wasm32-unknown-unknown --toolchain nightly-2019-01-24`
- [Zmq](http://zeromq.org/intro:get-the-software)
  - zeromq is a "distributed messaging" software package utilized in the networking stack of Holochain
  - the link above has common platform installation instructions
  - without ZMQ the installation command that follows will fail


To install the cutting edge version of the Holochain conductor, run the following command in a terminal
```shell
$ cargo install holochain --force --git https://github.com/holochain/holochain-rust.git --branch develop
```

To install the latest released version of the Holochain conductor, run the following command in a terminal
```shell
$ cargo install holochain --force --git https://github.com/holochain/holochain-rust.git --tag v0.0.5-alpha
```

The Conductor should then be available from your command line using the `holochain` command.

Run `holochain --version` to confirm that it built.

## Usage
To learn about `holochain` and how to use it, check out the chapter in the guidebook all about it.

[https://developer.holochain.org/guide/latest/production_conductor.html](https://developer.holochain.org/guide/latest/production_conductor.html)

## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](../CONTRIBUTING.md) for our general practices and protocols on participating in the community.

## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2018, Holochain Foundation

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
