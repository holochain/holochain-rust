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

This `holochain-rust` repository implements three distinct yet overlapping aspects of the Holochain framework.

1. The core Holochain functionality that executes applications
2. A wrapper used to instantiate, manage, and run applications
3. A library and syntax for use in Rust based development of Zomes within applications

Let's elaborate on these three aspects.

### 1 - core
The [core](./core) folder contains the code that implements the core functionality of Holochain. It is the aspect that takes in an application DNA, and an agent, and securely runs peer-to-peer applications by exposing the API functions to Zomes. It draws on other top level definitions and functions such as [dna](./dna), [cas_implementations](./cas_implementations), [agent](./agent), and [core_types](./core_types).

### 2 - core api
The first aspect only implements the logic for the execution of a single application. Meanwhile, there is a strong need to be able to load and instantiate multiple applications, and possibly even multiple instances of the same application. The [core_api](./core_api) folder contains the code that implements this capability. Even then, this code must be employed within some other context to actually run it. Such a thing is called a Holochain "container".

A container is a Holochain utility or service that manages and runs Holochain applications. The first such containers are the GUI driven [holosqape](https://github.com/holochain/holosqape) and the CLI driven [hcshell](https://github.com/holochain/holosqape#hcshell) container for test running.

To implement a Rust based container, one could use the Rust crate exposed in [core_api](./core_api). To implement a container in a C based language, the [core_api_c_binding](./core_api_c_binding) code could be used, such as HoloSqape does.

### 3 - HDK Rust
Holochain applications have been designed to consist at the low-level of WebAssembly (WASM) running in a virtual machine environment. However, most languages will be easiest to use with some stub code to connect into the WASM runtime environment, because of some constraints with WASM. That is the main reason why a "Developer Kit" for a language exists. It is a library, and a syntax, for writing Zome code in that language.

[`hdk-rust`](./hdk-rust) is a solid reference implementation of this, that enables Zomes to be written in the Rust language (the same, somewhat confusingly, as Holochain Core).

Within this repository, some aspects cross over between `core` and `hdk-rust`, such as [core_types](./core_types), since they get stored into WASM memory in `core`, and then loaded from WASM memory, within `hdk-rust`. Related, [wasm_utils](./wasm_utils) is used on both sides to actually perform the storing, and loading, of values into and from WASM memory.

#### Other HDKs and language options
Any language that compiles to WASM and can serialize/deserialize JSON data can be available as an option for programmers to write Holochain applications.

An HDK for [Assemblyscript](https://github.com/Assemblyscript/assemblyscript) is under experimental development at [`hdk-assemblyscript`](https://github.com/holochain/hdk-assemblyscript).

We expect many more languages to be added by the community, and there is even an article on how to [write a kit for a new language](https://holochain.github.io/holochain-rust/writing_development_kit.html).

## Documentation: The Book on Holochain
There is a work-in-progress book of documentation being written about `holochain-rust`. See the published version at the associated GitHub Pages for this repo, [https://holochain.github.io/holochain-rust](https://holochain.github.io/holochain-rust). See instructions for how to contribute to the book at [doc/holochain_101/src/how_to_contribute.md](./doc/holochain_101/src/how_to_contribute.md).

## Installation & Usage

Please note that this repository, with its installation instructions, is not useful to install if you fall into one of the following two categories:

If you are developing Holochain applications, you will want to instead install both:
- the [`hc` command line tool](https://github.com/holochain/holochain-cmd)
- the [`hcshell` test runner](https://github.com/holochain/holosqape#hcshell)
These will help create and test Holochain DNA packages suitable for running in a Holochain service.
You may also wish to look into the [hdk-rust](./hdk-rust) folder within this repository, to read on about the use of the HDK for Zome development.

If you are a Holochain end-user, either you will install DNA packages into a Holochain hApp's service like [HoloSqape](https://github.com/holochain/holosqape), or your application will come with them built in.

**The following instructions are for Holochain Core & HDK Developers Only**

There are two approaches to building and testing Holochain, using `make` or using `docker`:

### Make (ubuntu and macOS only)

If you are running on ubuntu or macOS, and you have `make` installed, you can do local development by simply typing:

`make` which will:

1. install (or update to) the correct version of rust
2. build all the rust libraries from the source code in this repository.

### Docker

However, we mostly use [docker](https://www.docker.com/) because it's easier to count on things working the expected way across platforms.

The `docker` folder contains scripts to build and run docker images.

#### Running tests

Run:

```shell
. docker/run-test
```

#### Code style
There is a linter/formatter enforcing code style.

Run:

```shell
. docker/run-fmt
```

#### Updating the CI Environment

The continuous integration (CI) suite executes the same `. docker/run-test` command that developers are encouraged to run.

What happens if I need to change that environment? E.g. what if I need a new system library dependency installed?

- Step 1 - Add the dependency to `docker/Dockerfile.ubuntu`

```dockerfile
RUN apt-get update && apt-get install --yes\
  # ... snip ...
  my-new-lib-here
```

- Step 2 - Build it

```shell
. docker/build-ubuntu
```

- Step 3 - Test it out

```shell
. docker/run-test
```

- Step 4 - Wait a minute! The CI environment is still using the old Dockerfile!

If your changes do not break the current environment, you can submit a separate Pull Request first, and once it is merged, the CI environment should be up-to-date for your code change Pull Request.

Otherwise, you will need to speak to an admin who can force merge your full changes after testing locally.

### Building for Android
Note there is an article written on how to build Holochain for Android, read it [here](doc/holochain_101/src/holochain_across_platforms.md).

## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](https://github.com/holochain/org/blob/master/CONTRIBUTING.md) for our general practices and protocols on participating in the community.

### App Spec Driven Development
In adding significant changes and new features to Holochain, we follow a specific test-driven development protocol:
1. Start by creating a branch in the [app-spec-rust](https://github.com/holochain/app-spec-rust) repository which demonstrates an actual implementation of the use of the new feature in the sample application that lives in that repository, including tests that would pass if the feature were actually implemented here in the holochain-rust repo.
1. Create a pull request on that branch for the development team to talk about and discuss the suggested change.  The PR triggers Continuous Integration tests which will initially fail, because they try and run the proposed changes against the `develop` branch of this `holochain-rust` repo.
1. Do any development necessary in the `holochain-rust` repo to implement the feature demonstrated in `app-spec-rust`
1. Finally, when the feature is fully implemented, the CI tests should turn green on `app-spec-rust` and the branch can be merged.  This merge in `app-spec-rust` of the feature branch completes the test-driven development loop.

In this way [`app-spec-rust`](https://github.com/holochain/app-spec-rust) works as a living specification with example app to build against.

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
