# Holochain-rust
This is the home of the Holochain Rust library, being rewritten from [Go](https://github.com/holochain/holochain-proto) into Rust. See https://holochain.org.

[![Code Status](https://img.shields.io/badge/Code-Pre_Alpha-red.svg)](https://github.com/holochain/holochain-rust/milestones?direction=asc&sort=completeness&state=all)
[![Travis](https://img.shields.io/travis/holochain/holochain-rust/develop.svg)](https://travis-ci.org/holochain/holochain-rust/branches)
[![Codecov](https://img.shields.io/codecov/c/github/holochain/holochain-rust.svg)](https://codecov.io/gh/holochain/holochain-rust/branch/develop)
[![In Progress](https://img.shields.io/waffle/label/holochain/holochain-rust/in%20progress.svg)](http://waffle.io/holochain/holochain-rust)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)
[![Twitter Follow](https://img.shields.io/twitter/follow/holochain.svg?style=social&label=Follow)](https://twitter.com/holochain)

**Holographic storage for distributed applications.** A holochain is a monotonic distributed hash table (DHT) where every node enforces validation rules on data before publishing that data against the signed chains where the data originated.

In other words, a holochain functions very much **like a blockchain without bottlenecks** when it comes to enforcing validation rules, but is designed to  be fully distributed with each node only needing to hold a small portion of the data instead of everything needing a full copy of a global ledger. This makes it feasible to run blockchain-like applications on devices as lightweight as mobile phones.

**[Code Status:](https://github.com/holochain/holochain-rust/milestones?direction=asc&sort=completeness&state=all)** Rust version is currently Pre-Alpha. Not for production use. The code has not yet undergone a security audit. We expect to destructively restructure code APIs and data chains until Beta. Prototype go version was unveiled at our first hackathon (March 2017), with go version Alpha 0 was released October 2017.  Alpha 1 was released May 2018.  We expect a developer pre-release of this Rust re-write in mid October 2018.
<br/>

| Holochain Links: | [FAQ](https://github.com/holochain/holochain-proto/wiki/FAQ) | [Developer Docs](https://holochain.github.io/holochain-rust/) | [White Paper](https://github.com/holochain/holochain-proto/blob/whitepaper/holochain.pdf) |
|---|---|---|---|


## Documentation: The Book on Holochain

There is a work-in-progress book being written about `holochain-rust`. See the published version at the associated GitHub Pages for this repo, [https://holochain.github.io/holochain-rust](https://holochain.github.io/holochain-rust). See instructions for how to contribute to the book at [./doc/holochain_101/src/how_to_contribute.md](./doc/holochain_101/src/how_to_contribute.md).

## Installation
**Core Developers Only:** This `holochain-rust` repo delivers the holochain-core rust based cargo libraries, not any end-user executables.  These installation instructions are for developers of Holochain Core itself.  If you are developing Holochain applications, you will want to install the [`hcdev` command line tool](https://github.com/holochain/holochain-cmd) to create Holochain Genome packages suitable for running in a Holochain service.  If you are a Holochain end-user, either you will install Genome packages into a Holochain hApp's service like [HoloSqape](https://github.com/holochain/holosqape), or you application will come with them built in.

### Local development & testing

#### Make

If you are running on ubuntu or Mac OS X, and you have `make` installed, you can do local development by simply typing:

`make` which will:

1. install (or update to) the correct version of rust
2. build all the rust libraries

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

### Contribute
We accept Pull Requests and welcome your participation. Please make sure to include the issue number your branch names and use descriptive commit messages.


[![In Progress](https://img.shields.io/waffle/label/holochain/holochain-rust/in%20progress.svg)](http://waffle.io/holochain/holochain-rust)

Some helpful links:

* View our [Kanban on Waffle](https://waffle.io/holochain/holochain-org).
* Chat with us on our [Chat Server](https://chat.holochain.org) or [Gitter](https://gitter.im/metacurrency/holochain)

Current Throughput graph:

[![Throughput Graph](http://graphs.waffle.io/holochain/holochain-rust/throughput.svg)](https://waffle.io/holochain/holochain-rust/metrics)

Contributors to this project are expected to follow our [development protocols & practices](https://github.com/holochain/holochain-rust/wiki/Development-Protocols).


## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2018, Holochain Trust

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
