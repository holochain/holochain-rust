# 6. Splitting agent into front-house / back-house - OR NOT

Date: 2018-05-16

## Status

Pending

## Context

We need to make Holo work with having keys and source chain on the client machine and the rest of Holochain be held by a HoloPorts

### Problem:
- every interaction the "back-house" has with other agents needs to be signed, i.e. including continued gossip and world-model interactions
- if key held in front-house (by definition of this partitioning), this means:
  - if front-house lives in Holo-front-end in the browser, HoloPort would need to tunnel everything through client for all signing interactions.
  - doesn't work for mobile because node wants to shut down for memory and bandwith reasons

### Possible Solution:

user signs agreement that another agent can act on his behalf (at least for gossip related actions)

- we basically introduce an identity that consist of several agents, of which one is the HoloPort
- might also solve the multiple-device problem

## Decision

Use Front-House/Back-House distinction with some system for tunneling signing
or
Assume no Front-House/Back-House architectural split and handle passing of signing duty to proxy as part of compound-identity.

## Consequences

-> we need to check-in with @artbrock
