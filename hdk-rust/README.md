# Holochain Development Kit for Rust-based Apps

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat-square)](https://waffle.io/holochain/org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.net)

## Overview
`hdk-rust` is a library for Rust-based holochain dApps that makes it easier to develop Holochain zomes. With Holochain, zome functions and validation code are represented as WASM binaries. This library provides bindings for Rust.

## Usage
First, [Rust](https://www.rust-lang.org/en-US/install.html) must be installed on your computer.

Being a Rust library, `hdk-rust` can be added as a dependency to any Rust crate. When you generate Rust based Zomes with [holochain-cmd](https://github.com/holochain/holochain-cmd) it will automatically be added as a dependency, and imported into your code.

```rust
[package]
name = "yourappname"
version = "versionnumber"
authors = ["Your Name Here"]

[dependencies]
hdk = { git = "https://github.com/holochain/hdk-rust"}
```

`hdk-rust` includes a macro which should be used for writing your application logic into Zome functions. To use it looks something like this:
```
#[macro_use]
extern crate hdk;
extern crate holochain_wasm_utils;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct CreatePostResponse {
    author: String,
}

zome_functions! {
    create_post: |author: String, content: String| {

        // ..snip..

        CreatePostResponse { author: author }
    }
}
```

### Specification for App Development
As new features, or changes to the HDK (and the API) are being designed, use cases will be added to an example app and put as changes to a pull request to its [repository](https://github.com/holochain/app-spec-rust). The repository also integrates the feature set available in Holochain's main branch.

Please see the [Contribute section](https://github.com/holochain/holochain-rust/blob/develop/README.md#app-spec-driven-development) for our protocol on how we do this.


### Availability of API Functions
Functions will continue to move from incomplete to complete as this library matures.

The following functions are **complete**:
- debug
- commit_entry

The following functions are **incomplete**:
- property
- make_hash
- call
- sign
- verify_signature
- update_entry
- update_agent
- remove_entry
- get_entry
- link_entries
- get_links
- query
- send
- start_bundle
- close_bundle

## Organization of Code
`global.rs` holds all internal or private globals used by the zome API library, and contains internal global for memory usage, internal global for retrieving all app globals, and invokable functions in the ribosome

`lib.rs` holds the public zome API where all API reference documentation is (app global variables, system consts, and API functions)

`macro.rs` is a macro for easily writing zome functions

`init_globals.rs` holds the internal/private zome API function that retrieves all the public global values from the ribosome

`Cargo.toml` manifest files describe dependencies. They introduce two metadata files with bits of projection information, fetch and build dependencies, and invokes Holochain Rust with the correct parameters.

## Tests

To test you can either use `make` with:

`make test`

or, if you want to do this manually remember that you will need to build the wasm file that the tests need to run.  Build it like this:
```bash
$ cd wasm-test
$ cargo build --target wasm32-unknown-unknown
```
Then, run:
```bash
$ cd ..
$ cargo test
```

### Integration test
A test that sets up and runs a holochain instance, then calls the exposed WASM function that calls the Commit API function.

### WASM test
Tests WASM utilities.


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
