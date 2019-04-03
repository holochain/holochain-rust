# Holochain Command Line Tools

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat-square)](https://waffle.io/holochain/org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.org)

This crate implements a set of development tools for building and running Holochain DNA from the command line.

## Install

### Scripted installation

The install scripts provided by holochain core (see [README](../README.md)) install all the dependencies required for holochain cli.

### Manual installation

An option for the installation of `hc` command line tools is to download the binary for your platform from our [releases](https://github.com/holochain/holochain-rust/releases) page. Otherwise, you can proceed with the more complex instructions for building from source, below.

### Building From Source

These dependencies need to be installed in order to compile, and use `hc`:

- [Rust](https://www.rust-lang.org/en-US/install.html)
  - needs to be the `nightly-2019-01-24` build, so use the following commands, once you have first installed Rust
  - `rustup toolchain install nightly-2019-01-24`
  - `rustup default nightly-2019-01-24`
  - (the specific nightly build we use will change over time)
  - Also, if you are going to be developing Zomes in Rust, install the WASM build target for Rust, by running:
  - `rustup target add wasm32-unknown-unknown --toolchain nightly-2019-01-24`
- [Node.js](https://nodejs.org) version 8 or higher
  - Tests for Holochain apps are now written in Javascript and executed in Nodejs
  - To read further, check out [the holochain-nodejs module](https://www.npmjs.com/package/@holochain/holochain-nodejs)


To install the cutting edge version of the Holochain conductor, run the following command in a terminal
```shell
$ cargo install hc --force --git https://github.com/holochain/holochain-rust.git --branch develop
```

To install the latest released version of the Holochain conductor, run the following command in a terminal
```shell
$ cargo install hc --force --git https://github.com/holochain/holochain-rust.git --tag v0.0.9-alpha
```

The command line tools are now available in your command line using the `hc` command.

Run `hc --version` to confirm.

Run `hc help` for help.

### Networking

If you want to use `hc run` with real (as opposed to mock) networking, you will also need to install [n3h](https://github.com/holochain/n3h).

## Usage

`(u)` means the command is not yet implemented.

| Command   | Use                                                                 |
|-----------|---------------------------------------------------------------------|
| init      | Initializes a new Holochain app at the given directory              |
| generate  | Generates a new Zome                                                |
| package   | Builds the current Holochain app into a `.dna.json` file            |
| unpack    | Unpacks a Holochain bundle into its original file system structure  |
| test      | Runs tests written in the test folder                               |
| run       | Starts a websocket server for the current Holochain app             |
| keygen    | Creates a new passphrase encrypted agent key bundle                 |

### hc init & hc generate: How To Get Started Building An App

To read about `hc init`, used for starting a new Holochain project, see [https://developer.holochain.org/guide/latest/new_project.html](https://developer.holochain.org/guide/latest/new_project.html).

To read about `hc generate`, used for generating boilerplate code and files for a new Zome, see [https://developer.holochain.org/guide/latest/zome/adding_a_zome.html](https://developer.holochain.org/guide/latest/zome/adding_a_zome.html).

### hc package: Using Built-in Compilation

To read about `hc package`, used for bundling your source files into a single file runnable by Holochain, see [https://developer.holochain.org/guide/latest/packaging.html](https://developer.holochain.org/guide/latest/packaging.html).

### hc test: Writing and Running Tests

To read about `hc test`, used for running tests over your source code, see [https://developer.holochain.org/guide/latest/intro_to_testing.html](https://developer.holochain.org/guide/latest/intro_to_testing.html).

### hc run: Running your application

To read about `hc run`, used for spinning up a quick development version of your app with an HTTP or Websocket interface, that you can connect to from a UI, or any client, see [https://developer.holochain.org/guide/latest/development_conductor.html](https://developer.holochain.org/guide/latest/development_conductor.html).

### hc keygen: Create agent key pair

Every agent is represented by a private/public key pair, which are used to author source chains.
This command creates a new key pair by asking for a passphrase and writing a key bundle file that a Holochain Conductor
can read when starting up an instance.

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
