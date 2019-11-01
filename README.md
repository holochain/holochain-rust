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

## Application Developer

Setup a development environment with the latest release to build Holochain applications:

https://developer.holochain.org/start.html

## Core Developer

Setup a core development environment to work on Holochain itself:

[Core Developer Setup](./doc/CoreDevSetup.md)

## Documentation

### API Reference
Auto generated documentation for all of the code written in Rust is published online, with different versions to match the releases here: [https://developer.holochain.org/docs/api](https://developer.holochain.org/docs/api).

### The Holochain Guidebook
There is a guide for understanding and developing with Holochain. It is published online, with different versions to match the releases here: [https://developer.holochain.org/guide](https://developer.holochain.org/guide).

 See instructions for how to contribute to the book at [doc/holochain_101/src/how_to_contribute.md](./doc/holochain_101/src/how_to_contribute.md).


## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](/CONTRIBUTING.md) for our general practices and protocols on participating in the community, as well as specific expectations around things like code formatting, testing practices, continuous integration, etc.

Some helpful links:

* Chat with us on our [Chat Server](https://chat.holochain.org) or [Gitter](https://gitter.im/metacurrency/holochain)


## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2017 - 2019, Holochain Foundation

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
