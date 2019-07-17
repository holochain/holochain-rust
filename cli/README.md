# Holochain Command Line Tools

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat-square)](https://waffle.io/holochain/org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.org)

This crate implements a set of development tools for building and running Holochain DNA from the command line.

## Install

The recommended pattern for usage of the command line tools is to follow the installation instructions found in the Quick Start guide on the developer documentation.

[https://developer.holochain.org/start.html](https://developer.holochain.org/start.html)

## Usage

Run `hc --version` to check your version number.

Run `hc help` for help.

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
