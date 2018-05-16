# 6. Splitting agent into front-house / back-house - OR NOT

Date: 2018-05-16

## Status

Accepted

## Context

We need to make Holo work with having keys and source chain on the client machine and the rest of Holochain be held by a HoloPorts

## Problem:
every interaction the back-house is having with other agents needs to be signed
key is hold in front-house (by definition of this partitioning)
if front-house lives in Holo-front-end in the browser, HoloPort would need to tunnel everything through client
-> meh
-> we can't to this
-> we need to check-in with @artbrock
solution: user signs agreement that another agent can act on his behalf
-> we basically introduce an identity that consist of several agents, of which one is the HoloPort
-> might also solve the multiple-device problem

## Decision

For Holo, seperating the back end networking part of the agent from the front-end UI & logic part of the agent.

## Consequences

Consequences here...
