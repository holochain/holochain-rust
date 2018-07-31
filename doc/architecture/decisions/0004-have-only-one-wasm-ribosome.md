# 4. Have only one WASM Ribosome

Date: 2018-05-16

## Status

Accepted

## Context

There is a WASM intepretor in Rust.
There seems to be a lot of momentum behind WASM: 
* all big browser support WASM since of Feb 2018
* recent blockchain projects jump on WASM

We will get more languages for Holochain for free just by those communities building WASM support
JS, C++ and Rust already with stable support.

By writing one WASM Ribosome we support more languages than sticking to a JS and Zygo Ribosome each.

## Decision

Implement only one WASM ribosome for holochain and have it working for 2 different toolchains/languages (Javascript and Rust or C++ - to be decided)


## Consequences

Non WASM-able languages like Python or Ruby will not be supported.
We will need to write toolchains to support more languages that are compiled into WASM.
