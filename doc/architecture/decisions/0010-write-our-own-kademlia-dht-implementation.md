# 10. Write our own Kademlia DHT implementation

Date: 2018-05-16

## Status

Proposed

## Context

For the Go based prototype we used IPFS' Kademlia DHT implementations with edits.

Since we are switching over to Rust, we can't easily continue using that code base.

Also, it seems we have too many Holochain specific additions to a vanilla Kademlia DHT so that might not make sense to change existing implementations but instead roll our own.

## Decision

We decide to build our own Kademlia DHT implementation in Rust from scratch.

## Consequences

Support and maintain forever our Kademlia code.
