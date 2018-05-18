# 7. Abstraction for network layer

Date: 2018-05-16

## Status

Proposed

## Context

In switching to Rust, we can no longer use libp2p directly, and we want our own Kademlia implementation anyways (see ADR: 0010)
What we want from a network library:
* Public Key cryptography for end-to-end encryption.
* Hash of public key as network address which wraps the transport layer for use by DHT.
* Possibility of multiple transport layers.
* Built in support for NAT/firewall traversal for case of IP transport. However: This may need to be handled by us explicitly as a Topology issue, i.e. having some holochain nodes act as routers (volunteering, or special nodes).

## Decision

Write our own abstraction layer for the networking library which describes the API of our needs, and thus makes it easier to choose/switch a networking stack.
Investigate Promether and see if it fits all our needs, including being network protocol agnostic (not just IP) [we think it is].

## Consequences

* More flexibility in replacing low-level network layer
* Work overhead of defining and implementing layer
* More clarity about what we really need
