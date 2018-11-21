# Holochain app specification in Rust

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat-square)](https://waffle.io/holochain/org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.net)

This repository contains a Holochain app that functions as a living specification of Holochain and its Rust [HDK](https://github.com/holochain/holochain-rust/tree/develop/hdk-rust) (Holochain Development Toolkit).

As new features, or changes to the HDK (and the API) are being designed, they will be made concrete by adding a use case to this example app and putting those changes as a pull-request to this repository. As soon as the current master branch in https://github.com/holochain/holochain-rust implements all used features, the PR gets merged here so that this repository's master branch always reflects the feature set available in Holochain's master branch.

Please see the [Contribute section](https://github.com/holochain/holochain-rust/blob/develop/README.md#app-spec-driven-development) for our protocol on how we do this.

### Dependencies

The primary dependency is on the Holochain command line tools. These can be installed here:
* https://github.com/holochain/holochain-cmd

The command line tools also have dependencies, needed only for the testing commands. 

Nodejs and NPM, for compiling Javascript tests, Version 8 or higher
* https://nodejs.org/en/download/

`hcshell`, the holochain-rust container bindings for Javascript, for executing tests
* https://github.com/holochain/holosqape#hcshell

Make sure that all three of these dependencies are available as binary executables in your terminal before continuing. For this, they must all be available in your PATH environment variable. The instructions to accomplish that can be found separately at the above links. To verify it is all working, run each of the following three commands. 

`hc`

If you see the available commands listed, they're successfully installed. 

`node -v`

If you see `vA.B.C` where A, B, and C are numbers, you've got `node` installed.

`hcshell -v`

If you see `hcshell X.Y`, where X, and Y are numbers, you've got `hcshell` installed.

### Run the tests

Make sure that you fully completed the installation of dependencies. Then, within the directory of this repo, cloned to your computer, run the following:

`./build_and_test.sh`

You should see the tests all passing successfully.

### Detailed Description

In order to get from the source directory tree to a Holochain DNA package, and then test that, several steps need to be taken which are all automated.

This includes compiling any Rust code projects to WASM, and then assembling a DNA file (.hcpkg) with all configuration snippets and the WASM in it.

It will error at this point if for some reason it can't successfully build the WASM, or the DNA file.

Once this packaging is complete, there are a few more steps.

Unless they have already been installed, it will install node_modules to your `test` folder.

Next, it will use `webpack` to compile your tests into a single executable file, in `test/dist/bundle.js`. 

Finally, it uses `hcshell` to run those tests, giving you the feedback you really want to test code, and develop new functionality.

## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](https://github.com/holochain/org/blob/master/CONTRIBUTING.md) for our general practices and protocols on participating in the community.

## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2018, Holochain Trust

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
