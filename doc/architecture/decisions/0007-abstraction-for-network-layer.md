# 7. Abstraction for network layer

Date: 2018-05-16

## Status

Accepted

## Context

In switching to Rust, we can no longer use libp2p directly, and we want our own Kademlia implementation anyways (see ADR: 0010)
What we want from a network library:
* Public Key cryptography for end-to-end encryption.
* Hash of public key as network address which wraps the transport layer for use by DHT.
* Possibility of multiple transport layers.
* Built in support for NAT/firewall traversal for case of IP transport.

Updated 7/20 context:

* Conversations with the IPFS folks about libp2p as well as its wider adoption by hyperledger and parity/polkadot as a P2P transport layer, along with the progress of libp2p-rust, and thinking about strategic alignment with other efforts and stewarding of our dev resources make it a more interesting candidate to use as a transport layer to handle the above requirements.
* ZeroMQ looks very interesting.
* neonphog has written lib3 as a prototyping transport.
* Promether questions.

## Decision

1. Write our own abstraction layer for the networking library with an API that allows us to build against our needs, and thus makes it easier to choose/switch a networking stack.
2. Assume that this allows connection to a directly compiled-in p2p layer or connection via thin layer to a p2p transport system service (see ADR #14)
3. Assume transport layer handles all multiplexing across different physical transports as well as topological problems (NAT relay etc)

## Consequences

* This abstraction is to allow us to switch between transport solutions, and is not explicitly designed for bridging between them, i.e. assumes that routing is handled.
* The kademlia re-implementation we will do (ADR #10) is about updating an maintaining app world model for data resiliency, not routing.
