# 14. P2P - IPC Abstraction

Date: 2018-07-19

## Status

Accepted

## Context

Rust as a language, in my opinion, does not lend itself to rapid prototyping. Writing efficient Rust code, while absolutely possible, is a problem domain unto itself, and takes time. E.g. Is it worth returning borrowed references in this case to save memory? Am I able to craft the lifetimes appropriately given all the other api usages of my struct? What about synchronization? These questions are implementation details, not architectural. If we are still working out the details with the underlying architecture, we don't want to be spending brainpower on * how * we accomplish our experiments.

In this prototyping / proof-of-concept phase, I don't want to take the time to write efficient rust code, but I don't want to leave us with a bunch of unmaintainable inefficient spaghetti if we *do* decide to go forward with the prototype.

## Decision

Abstract the p2p library at the process level.

- The P2P process will host a [ZeroMQ](http://zeromq.org/) ROUTER socket. This process can be in any language that supports zmq, and using any of the transports it supports. (Likely start with unix domain sockets for their high throughput).
- The holochain rust code will connect to the P2P process with a ROUTER socket using the [zmq](https://crates.io/crates/zmq) crate.
- The holochain rust code will access this ipc abstraction through the [network-abstraction](0007-abstraction-for-network-layer.md) framework allowing the option to implement, for example, the rust version of libp2p both internally, and as a separate process.

## Consequences

### Pros
* Ability to prototype and experiment with p2p solutions in other languages (including the more mature nodejs version of [libp2p](https://github.com/libp2p/js-libp2p)).
* Separation of concerns in terms of security and stability.
* Multiple Holochain processes can share the overhead (cpu, memory, bandwidth) of a single p2p network connection.

### Cons
* Work overhead for creation of additional abstraction layer.
* Additional complexity for system setup / maintanence for both developers and end-users. That is, they will have to start up / configure two processes instead of one.
