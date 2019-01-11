# Holochain Rust Container

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat-square)](https://waffle.io/holochain/org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.net)

This crate implements a reference container for serving Holochain DNAs.

## Install

Our recommended pattern for the installation of the container is to download the binary for your platform from our [releases](https://github.com/holochain/holochain-rust/releases) page. Otherwise, you can proceed with the more complex instructions for building from source, below.

### Building From Source

These dependencies need to be installed in order to compile, and use `holochain_container`:

- [Rust](https://www.rust-lang.org/en-US/install.html)
  - needs to be the `nightly` build, so use the following commands, once you have first installed Rust
  - `rustup toolchain install nightly`
  - `rustup default nightly`
  - Also, if you are going to be developing Zomes in Rust, install the WASM build target for Rust, by running:
  - `rustup target add wasm32-unknown-unknown --toolchain nightly`
- [Zmq](http://zeromq.org/intro:get-the-software)
  - zeromq is a "distributed messaging" software package utilized in the networking stack of Holochain
  - the link above has common platform installation instructions
  - without ZMQ the installation command that follows will fail


To install the latest version of the Holochain container, run the following command in a terminal
```shell
$ cargo install holochain_container --force --git https://github.com/holochain/holochain-rust.git --branch develop
```

The container should then be available from your command line using the `holochain_container` command.

Run `holochain_container --version` to confirm that it built.

## Usage

The container requires a configuration file to run, you can see a [sample here](https://github.com/holochain/holochain-rust/blob/develop/container/example-config/basic.toml)

You can put your configuration file in `~/.holochain/container_config.toml` or run `holochain_container` explicitly with the `-c` to specify where to find it.

### Using real networking
The container currently uses mock networking by default. To use real networking you have to install the [n3h networking component](https://github.com/holochain/n3h) and add a configuration block into the config file to tell the container where it can find n3h.  It should look something like this:

```
[network]
n3h_path = "/home/eric/holochain/n3h"
n3h_persistence_path = "/tmp"
bootstrap_nodes = []
```

## Configuration File Spec

TBD (for now you just have infer from the example!)

## Testing HTTP interface using cURL

Currently the container supports the `websocket` and `http` interfaces.
Assuming the container http interface is running on port 4000 it can be tested by running:
`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":"0","method":"info/instances"}' http://localhost:4000`


## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](../CONTRIBUTING.md) for our general practices and protocols on participating in the community.

## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2018, Holochain Trust

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
