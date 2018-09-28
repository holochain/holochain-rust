# 10. Write our own DHT implementation

Date: 2018-05-16

## Status

Accepted

## Context

For the Go based prototype we used IPFS' Kademlia DHT implementations with edits.

Since we are switching over to Rust, we can't easily continue using that code base.

More importantly, there are too many Holochain specific additions to a vanilla Kademlia DHT, as well as other possible implementations of achieving entry resilience, routing, our forthcoming World Model, CRDT handling, gossip and our graph properties via linking, so it does not make sense to change existing implementations but instead roll our own.

## Decision

We will build our own DHT implementation in Rust from scratch.

## Consequences

- This allows us to separate some networking concerns.
- We will have to support and maintain forever our DHT code.
- This may make it easier to adopt other distance metrics for the DHT, so we should think of generalizing the Distance metric from the start.
