# Holochain-Rust Architectural Overview

This `holochain-rust` repository implements a number of distinct yet overlapping aspects of the Holochain framework.

1. A library for the core Holochain functionality for defining and running DNA instances: [*core*](#core)
1. A library and syntax for use in Rust based development of Zomes within DNAs, called Holochain Development Kit: [*hdk-rust*](#hdk-rust)
1. A library for managing instances and connecting them to interfaces: [*conductor-api*](#conductor-api)
1. A Rust based Conductor that uses the conductor_api: [*conductor*](#rust-conductor)
1. A nodejs based Conductor for running tests: [*nodejs-conductor*](#nodejs-conductor)
1. A command line developer tool: [*hc*](#hc-command-line-developer-tool)
1. A sample application that we use to demonstrate the current functionality and drive development: [*app-spec*](#app-spec-driven-development)

## Core
The [core](/core) folder contains the code that implements the core functionality of Holochain. It is the aspect that takes in an application DNA, and an agent, and "securely" (NOT secure during alpha) runs peer-to-peer applications by exposing the API functions to Zomes. It draws on other top level definitions and functions such as [dna](/dna), [cas_implementations](/cas_implementations), [agent](/agent), and [core_types](/core_types).

## HDK Rust
Holochain applications have been designed to consist at the low-level of WebAssembly (WASM) running in a virtual machine environment. Most languages will be easiest to use with some stub code to connect into the WASM runtime environment due to the nature of WASM. That is the main reason why a "Developer Kit" for a language exists. It is a library, and a syntax, for writing Zome code in that language.

[`hdk-rust`](/hdk-rust) is a solid reference implementation of this, that enables Zomes to be written in the Rust language (the same, somewhat confusingly, as Holochain Core).

Within this repository, some aspects cross over between `core` and `hdk-rust`, such as [core_types](/core_types), since they get stored into WASM memory in `core`, and then loaded from WASM memory, within `hdk-rust`. Related, [wasm_utils](/wasm_utils) is used on both sides to actually perform the storing, and loading, of values into and from WASM memory.

### Other HDKs and language options
Any language that compiles to WASM and can serialize/deserialize JSON data can be available as an option for programmers to write Holochain applications.

An HDK for [Assemblyscript](https://github.com/Assemblyscript/assemblyscript) is under experimental development at [`hdk-assemblyscript`](https://github.com/holochain/hdk-assemblyscript).

We expect many more languages to be added by the community, and there is even an article on how to [write a kit for a new language](https://developer.holochain.org/guide/latest/writing_development_kit.html).

## Conductor API
*Core* only implements the logic for the execution of a single DNA.  Because the Holochain app ecosystem relies on DNA composibility, we need to be able to load and instantiate multiple DNAs.  We call an executable that can do this a *Conductor*.  The first such Conductors we implemented were the Qt C++ GUI driven [holosqape](https://github.com/holochain/holosqape) and the CLI driven [hcshell](https://github.com/holochain/holosqape#hcshell) Conductor which we used for running javascript based tests.

These gave us the experience from which we abstracted the [conductor_api](conductor_api) crate which specifies and implements a standard way for building conductors, including specifying the various interfaces that might be available for executing calls on a particular DNA, i.e. websockets, HTTP, Unix domain sockets, carrier pigeon network, etc...

If you need to implement your own conductor, [conductor_api](conductor_api) should provide you with the needed types and functions to do so easily.

To implement a conductor in a C based language, the [core_api_c_binding](/core_api_c_binding) [NEEDS UPDATING] code could be used, such as HoloSqape does.

## Rust Conductor
The [conductor crate](/conductor) uses the [conductor_api](conductor_api) to implement an executable which is intended to become the main, highly configurable and GUI less conductor implementation that can be run as a background system service.

## Nodejs Conductor
The [nodejs_conductor](/nodejs_conductor) directory implements a node package that creates a conductor that wraps the Holochain core Rust implementation so we can access it from node.  This is crucial especially for creating a test-driven development environment for developing Holochain DNA.  The `hc` command-line tool relies on it to run tests.

## HC Command-line developer tool.
The [cli crate](/cli) implements our command line developer tool which allows you to create DNA scaffold, run tests, and finally package your DNA for running in a Conductor.  For more details see the [crate README](/cli/README.md).


## App Spec Driven Development
We use a practice for coordinating additions and features that starts with adding a feature to a sample application so that we know we have a working example all the times.  You can read about [the details here](/CONTRIBUTING.md#app-spec-driven-development)