# 10. Write our own Kademlia DHT implementation

Date: 2018-05-16

## Status

Proposed

## Context

For the Go based prototype we used IPFS' Kademlia DHT implementations with edits.

Since we are switching over to Rust, we can't easily continue using that code base.

Also, it seems we have too many Holochain specific additions to a vanilla Kademlia DHT  (especially around the forthcoming World Model for resilience, CRDT handling, and our graph properties via linking) so that might not make sense to change existing implementations but instead roll our own.

## Decision

We will to build our own Kademlia DHT implementation in Rust from scratch.

## Consequences

- We will have to support and maintain forever our Kademlia code.
- This may make it easier to adopt other distance metrics for the DHT, so we should think of generalizing the Distance metric from the start.
