# Holochain
This is the home of the Holochain library, being rewritten from [Go](https://github.com/holochain/holochain-proto) into Rust. See https://holochain.org.

### Building for Android
Note there is an article written for how to build Holochain for Android, read it [here](doc/holochain_101/src/holochain_across_platforms.md).

#### State, Reducers, and Actions: The Redux-style Architecture
To read about the redux like architecture, see [here](doc/holochain_101/src/state_actions.md).

## Local development & testing

**NEVER RUN `make` ON ITS OWN UNLESS YOU KNOW WHAT YOU'RE DOING**

CI builds are happening on circle CI.

### Docker

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

### The Book on Holochain

There is a work-in-progress book being written about `holochain-rust`. See the published version at the associated GitHub Pages for this repo, [https://holochain.github.io/holochain-rust](https://holochain.github.io/holochain-rust). See instructions for how to contribute to the book at [./doc/holochain_101/src/how_to_contribute.md](./doc/holochain_101/src/how_to_contribute.md).



## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2018, Holochain Trust

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.
