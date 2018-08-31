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
