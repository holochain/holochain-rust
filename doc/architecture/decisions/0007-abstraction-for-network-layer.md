# 7. Abstraction for network layer

Date: 2018-05-16

## Status

Accepted

## Context

libp2p has issues. DHT is too tied to network.
What we want from a network library: Public Key cryptography: Hash public key that corresponds to identity on the network. Seemless port use.
Handles routing tables?

## Decision

Have an abstraction layer for the networking library so we are not dependent on a specific one.
Investigate Promether and see if it fits all our needs, including beeing network protocol agnostic (not just IP).

## Consequences

Consequences here...
