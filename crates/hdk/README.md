# Holochain Development Kit for Rust-based Zomes

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat-square)](https://waffle.io/holochain/org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.net)

## Overview
`hdk-rust` is a library for Rust-based hApps that makes it easier to develop Holochain Zomes. With Holochain, Zome functions and validation code are represented as WASM binaries. This library provides bindings for Rust.

## Usage
First, [Rust](https://www.rust-lang.org/en-US/install.html) must be installed on your computer.

Being a Rust library, `hdk-rust` can be added as a dependency to any Rust crate. When you generate Rust based Zomes with [hc](https://github.com/holochain/holochain-rust/tree/develop/cli) it will automatically be added as a dependency, and imported into your code.

To see the documentation for usage, check out [https://developer.holochain.org/api](https://developer.holochain.org/api)

### Specification for App Development
As new features, or changes to the HDK (and the API) are being designed, use cases will be added to an example app and put as changes to a pull request to the app_spec directory of this repo. The example app also integrates the feature set available in Holochain's main branch.

Please see the [Contribute section](https://github.com/holochain/holochain-rust/blob/develop/README.md#app-spec-driven-development) for our protocol on how we do this.

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
