# Holochain-rust

<h1 align="center">
  <a href="http://holochain.org"><img width="250" src="https://github.com/holochain/org/blob/master/logo/holochain_logo.png?raw=true" alt="holochain logo" /></a>
</h1>

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat)](https://waffle.io/holochain/holochain-rust)
[![Twitter Follow](https://img.shields.io/twitter/follow/holochain.svg?style=social&label=Follow)](https://twitter.com/holochain)

[![Travis](https://img.shields.io/travis/holochain/holochain-rust/develop.svg)](https://travis-ci.org/holochain/holochain-rust/branches)
[![Codecov](https://img.shields.io/codecov/c/github/holochain/holochain-rust.svg)](https://codecov.io/gh/holochain/holochain-rust/branch/develop)
[![In Progress](https://img.shields.io/waffle/label/holochain/holochain-rust/in%20progress.svg)](http://waffle.io/holochain/holochain-rust)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

This is the home of the Holochain Rust library, being rewritten from [Go](https://github.com/holochain/holochain-proto) into Rust.

**[Code Status:](https://github.com/holochain/holochain-rust/milestones?direction=asc&sort=completeness&state=all)** Rust version is currently Pre-Alpha. Not for production use. The code has not yet undergone a security audit. We expect to destructively restructure code APIs and data chains until Beta. Prototype go version was unveiled at our first hackathon (March 2017), with go version Alpha 0 was released October 2017.  Alpha 1 was released May 2018.  We expect a developer pre-release of this Rust re-write in mid October 2018.
<br/>

| Holochain Links: | [FAQ](https://github.com/holochain/holochain-proto/wiki/FAQ) | [Developer Docs](https://holochain.github.io/holochain-rust/) | [White Paper](https://github.com/holochain/holochain-proto/blob/whitepaper/holochain.pdf) |
|---|---|---|---|

## Overview

This `holochain-rust` repo does not contain any end-user executables, rather it delivers the holochain-core libraries in the form of a number of rust cargo crates which other repos use for building utilities or Holochain services that actually run Holochain applications:

- `holochain_core_api`: the primary client wrapper crate used to instantiate and run a Holochain genome.
- `hoclocian_core`: the main crate that implements the core Holochain functionality.
- `holochain_dna`: a crate for working with holochain genome from a package file.  Used by both holochain_core the [packager utility](https://github.com/holochain/holochain-cmd)
- `holochain_agent`: a crate for managing holochain agent info, including identities, keys etc..  Used by both holochain_core and other utilities.

We have designed Holochain applications to consist at the low-level of WebAssembly running in a virtual machine environment.  This allows us to robustly make any language that compiles to WASM available as an option for programmers to write their Holochain applications.  However each language requires a small bit of stub code to connect into the WASM runtime environment.  `[hdk-rust]`(https://github.com/holochain/hdk-rust) and `[hdk-assemblyscript]`(https://github.com/holochain/hdk-assemblyscript) implement the code for Rust and TypeScript compatibility.  We expect many more languages to be added by the community.

## Documentation: The Book on Holochain

There is a work-in-progress book being written about `holochain-rust`. See the published version at the associated GitHub Pages for this repo, [https://holochain.github.io/holochain-rust](https://holochain.github.io/holochain-rust). See instructions for how to contribute to the book at [./doc/holochain_101/src/how_to_contribute.md](./doc/holochain_101/src/how_to_contribute.md).

## Installation & Usage
**Core Developers Only:**  These instructions are for developers of Holochain Core itself.  If you are developing Holochain applications, you will want to install the [`hcdev` command line tool](https://github.com/holochain/holochain-cmd) to help create Holochain Genome packages suitable for running in a Holochain service.  If you are a Holochain end-user, either you will install Genome packages into a Holochain hApp's service like [HoloSqape](https://github.com/holochain/holosqape), or you application will come with them built in.

There are two approaches to building and testing Holochain, using `make` or using `docker`:

### Make

If you are running on ubuntu or Mac OS X, and you have `make` installed, you can do local development by simply typing:

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
Note there is an article written for how to build Holochain for Android, read it [here](doc/holochain_101/src/holochain_across_platforms.md).

## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](https://github.com/holochain/org/master/CONTRIBUTING.md) for our general practices and protocols on participating in the community.

In adding significant changes and new features to Holochain, we follow a specific test-driven development protocol:
1. Start by creating a branch in the [app-spec-rust](https://github.com/holochain/app-spec-rust) repository which demonstrates an actual implementation of the use of the new feature in the sample application that lives in that repository, including tests that would pass if the feature were actually implemented here in the holochain-rust repo.
1. Create a pull request on that branch for the development team to talk about and discuss the suggested change.  The PR triggers Continuous Integration tests which will initially fail, because they try and run the proposed changes against the `develop` branch of this `holochain-rust` repo.
1. Do any development necessary to on here on `holochain-rust` and `hdk-rust` to implement the feature demonstrated in `app-spec-rust`
1. Finally, when the feature is fully implemented, the CI tests should turn green on `app-spec-rust` and the branch can be merged indicating that that feature.

In this way [`app-spec-rust`](https://github.com/holochain/app-spec-rust) works as a living specification with example app to build against.

[![In Progress](https://img.shields.io/waffle/label/holochain/holochain-rust/in%20progress.svg)](http://waffle.io/holochain/holochain-rust)

Some helpful links:

* View our [Kanban on Waffle](https://waffle.io/holochain/holochain-org).
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
