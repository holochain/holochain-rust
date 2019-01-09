# Holochain-rust

  <a href="http://holochain.org"><img align="right" width="200" src="https://github.com/holochain/org/blob/master/logo/holochain_logo.png?raw=true" alt="holochain logo" /></a>

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat-square)](https://waffle.io/holochain/org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.net)

[![Twitter Follow](https://img.shields.io/twitter/follow/holochain.svg?style=social&label=Follow)](https://twitter.com/holochain)

[![Travis](https://img.shields.io/travis/holochain/holochain-rust/develop.svg)](https://travis-ci.org/holochain/holochain-rust/branches)
[![Codecov](https://img.shields.io/codecov/c/github/holochain/holochain-rust.svg)](https://codecov.io/gh/holochain/holochain-rust/branch/develop)
[![In Progress](https://img.shields.io/waffle/label/holochain/holochain-rust/in%20progress.svg)](http://waffle.io/holochain/holochain-rust)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

This is the home of the Holochain Rust libraries, being rewritten from [Go](https://github.com/holochain/holochain-proto) into Rust, and extended.

**[Code Status:](https://github.com/holochain/holochain-rust/milestones?direction=asc&sort=completeness&state=all)** Rust version is currently Pre-Alpha. Not for production use. The code has not yet undergone a security audit. We expect to destructively restructure code APIs and data chains until Beta. Prototype go version was unveiled at our first hackathon (March 2017), with go version Alpha 0 released October 2017.  Alpha 1 was released May 2018.  We expect a developer pre-release of this Rust re-write in mid October 2018.
<br/>

| Holochain Links: | [FAQ](https://holochain.github.io/holochain-rust/faq.html) | [Developer Docs](https://holochain.github.io/holochain-rust/) | [White Paper](https://github.com/holochain/holochain-proto/blob/whitepaper/holochain.pdf) |
|---|---|---|---|

## Overview

This `holochain-rust` repository implements a number of distinct yet overlapping aspects of the Holochain framework.

1. A library for the core Holochain functionality for defining and running DNA instances: [*core*](#core)
1. A library and syntax for use in Rust based development of Zomes within DNAs, called Holochain Development Kit: [*hdk-rust*](#hdk-rust)
1. A library for managing instances and connecting them to interfaces: [*container-api*](#container-api)
1. A Rust based container that uses the container_api: [*container*](#rust-container)
1. A nodejs based container for running tests: [*nodejs-container*](#nodejs-container)
1. A command line developer tool: [*hc*](#hc-command-line-developer-tool)
1. A sample application that we use to demonstrate the current functionality and drive development: [*app-spec*](#app-spec-driven-development)

### Core
The [core](./core) folder contains the code that implements the core functionality of Holochain. It is the aspect that takes in an application DNA, and an agent, and securely runs peer-to-peer applications by exposing the API functions to Zomes. It draws on other top level definitions and functions such as [dna](./dna), [cas_implementations](./cas_implementations), [agent](./agent), and [core_types](./core_types).

### HDK Rust
Holochain applications have been designed to consist at the low-level of WebAssembly (WASM) running in a virtual machine environment. However, most languages will be easiest to use with some stub code to connect into the WASM runtime environment, because of some constraints with WASM. That is the main reason why a "Developer Kit" for a language exists. It is a library, and a syntax, for writing Zome code in that language.

[`hdk-rust`](./hdk-rust) is a solid reference implementation of this, that enables Zomes to be written in the Rust language (the same, somewhat confusingly, as Holochain Core).

Within this repository, some aspects cross over between `core` and `hdk-rust`, such as [core_types](./core_types), since they get stored into WASM memory in `core`, and then loaded from WASM memory, within `hdk-rust`. Related, [wasm_utils](./wasm_utils) is used on both sides to actually perform the storing, and loading, of values into and from WASM memory.

#### Other HDKs and language options
Any language that compiles to WASM and can serialize/deserialize JSON data can be available as an option for programmers to write Holochain applications.

An HDK for [Assemblyscript](https://github.com/Assemblyscript/assemblyscript) is under experimental development at [`hdk-assemblyscript`](https://github.com/holochain/hdk-assemblyscript).

We expect many more languages to be added by the community, and there is even an article on how to [write a kit for a new language](https://holochain.github.io/holochain-rust/writing_development_kit.html).

### Container API
*Core* only implements the logic for the execution of a single application. Because the Holochain app ecosystem relies on DNA composibility, we need to be able to load and instantiate multiple DNAs.  We call an executable that can do this an *container*.  The first such containers we implemented were the GUI driven [holosqape](https://github.com/holochain/holosqape) and the CLI driven [hcshell](https://github.com/holochain/holosqape#hcshell) container which we used for running javascript based tests.

These gave us the experience from which we abstracted the [container_api](container_api) crate which specifies and implements a standard way for building containers, including specifying the various interfaces that might be available for executing calls on a particular DNA, i.e. websockets, HTTP, Unix domain sockets, carrier pigeon network, etc...

If you need to implement your own container, [container_api](container_api) should provide you with the needed types and functions to do so easily.

To implement a container in a C based language, the [core_api_c_binding](./core_api_c_binding) [NEEDS UPDATING] code could be used, such as HoloSqape does.

### Rust Container
The [container crate](container) uses the [container_api](container_api) to implement an executable which is intended to become the main, highly configurable and GUI less container implementation that can be run as a background system service.

### Nodejs Container
The [nodejs_container](nodejs_container) directory implements a node package that creates a container that wraps the Holochain core Rust implementation so we can access it from node.  This is crucial especially for creating a test-driven development environment for developing Holochain DNA.  The `hc` command-line tool relies on it to run tests.

### HC Command-line developer tool.
The [cmd crate](cmd) implements our command line developer tool which allows you to create DNA scaffold, run tests, and finally package your DNA for running in a containter.  For more details see the [crate README](cmd/README.md).

## App Spec Driven Development
We use a practice for coordinating additions and features that starts with adding a feature to a sample application so that we know we have a working example all the times.  You can read about [the details here](/CONTRIBUTING.md#app-spec-driven-development)

## Documentation: The Book on Holochain
There is a work-in-progress book of documentation being written about `holochain-rust`. See the published version at the associated GitHub Pages for this repo, [https://developer.holochain.org/guide/latest](https://developer.holochain.org/guide/latest). See instructions for how to contribute to the book at [doc/holochain_101/src/how_to_contribute.md](./doc/holochain_101/src/how_to_contribute.md).

## Installation & Usage

**Important:** the instructions in this readme are for developers intending work on Holochain code-base itself, not Holochain application developers.  If you want to use Holochain, please proceed to the instructions on the quick start installation guide: **https://developer.holochain.org/start.html**

**The following instructions are for developing Holochain Core or the HDK itself**

There are two components needed currently to run Holochain applications, the core (what's in this repo) and also [the networking engine](https://github.com/holochain/n3h).  You can install and work on core using the built-in mock network following the instructions below, but if you want to actually test out your apps using the real networking, you will have to install [the networking component](https://github.com/holochain/n3h) following the instructions in the readme there.  (Note: please see the instructions in the [`hc` command-line tool readme](./cmd/README.md#using-real-networking) or the [container readme](./container/README.md#using-real-networking) for how to configure the tools to use the find and activate the networking component.

There are three approaches to building and testing Holochain: using `make`, `docker` or `nix`:

### Make (ubuntu and macOS only)

For Ubuntu you can install the prerequisites with :

``` shell
sudo apt-get install git build-essential libssl-dev curl
```

If you are running on ubuntu or macOS, and you have `make` installed, you can do local development by simply typing:

`make` which will:

1. install (or update to) the correct version of rust
2. build all the rust libraries from the source code in this repository.
3. build and install the command-line tools.

### Docker

We also use [docker](https://www.docker.com/).  The `docker` folder contains scripts to build and run docker images.

### NixOS

If you have `nix-shell` then feel free to use our `.nix` files.

`shell.core.nix` and `shell.tools.nix` are split to mirror the versioning behaviour in the makefile.

Not everything in the Makefile is implemented in nix, and a lot of things don't need to be. Notably the cross-platform and defensive installation of dependencies is not included.

If you have a nix friendly system, this is probably the fastest way to develop and test.

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
make test_cmd
make test_holochain
make test_app_spec
make build_nodejs_container
```

### Building for Android
Note there is an article written on how to build Holochain for Android, read it [here](doc/holochain_101/src/building_for_android.md).

## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](/CONTRIBUTING.md) for our general practices and protocols on participating in the community, as well as specific expectations around things like code formatting, testing practices, continuous integration, etc.


Some helpful links:

* View our [Kanban on Waffle](https://waffle.io/holochain/org) [![In Progress](https://img.shields.io/waffle/label/holochain/holochain-rust/in%20progress.svg)](http://waffle.io/holochain/holochain-rust)
* Chat with us on our [Chat Server](https://chat.holochain.org) or [Gitter](https://gitter.im/metacurrency/holochain)

Current Throughput graph:

[![Throughput Graph](http://graphs.waffle.io/holochain/holochain-rust/throughput.svg)](https://waffle.io/holochain/holochain-rust/metrics)


## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2018, Holochain Trust

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
