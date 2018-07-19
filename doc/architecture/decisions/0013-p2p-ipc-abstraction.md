# 13. P2P - IPC Abstraction

Date: 2018-07-19

## Status

Proposed

## Context

It's frustrating to admit, but I'm still pretty slow at writing rust code. Oh, I can hack something together quickly (gogo `unwrap()/clone()`), but when I sit down to write efficient production code, I get bogged down in cyclic lifetime reference issues or any number of other arcane rustisisms.

In this prototyping / proof-of-concept phase, I don't want to take the time to write efficient rust code, but I don't want to leave us with a bunch of unmaintainable inefficient spaghetti if we *do* decide to go forward with the prototype.

## Decision

Abstract the p2p library at the process level.

- Host a [ZeroMQ](http://zeromq.org/) REP socket for the p2p process. This process can be in any language that supports zmq, and using any of the transports it supports. (Likely start with unix domain sockets for their high throughput).
- Connect to the Host with a REQ socket from within the holochain rust code using the [zmq](https://crates.io/crates/zmq) crate.
- Create a Holochain IPC transport type that fits within the proposed [network-abstraction](0007-abstraction-for-network-layer.md) framework.

## Consequences

### Pros
* Ability to prototype and experiment with p2p solutions in other languages (including the more mature nodejs version of [libp2p](https://github.com/libp2p/js-libp2p)).
* Separation of concerns in terms of security and stability.
* Multiple Holochain processes can share the overhead (cpu, memory, bandwidth) of a single p2p network connection.

### Cons
* Work overhead for creation of additional abstraction layer.
* Additional complexity for system setup / maintanence for both developers and end-users.
