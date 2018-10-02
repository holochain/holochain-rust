# 6. Splitting agent into front-house / back-house - OR NOT

Date: 2018-05-16

## Status

Accepted

## Context

For Holo, we need to have user agent's keys and source chain on the client machine and the rest of the Holochain agent (the DHT shard etc.) be held by HoloPorts.

In February 2018, Arthur, Eric and Nico discussed this during a co-creative session in London and made the assumption to have the Holochain agent be split up into two pieces, called front-house and back-house (prior left and right hemisphere). The front-house was meant to entail the source chain and private key management as well as a ribosome to run the app and provide interfaces for the UI and in the non-Holo case for bridging - everything that is user/agent facing. The back-house should have been the DHT shard, routing table, metrics, etc. Basically everything that is network facing.

With this separation, the reasoning in February was to rewrite (only) the front-house in JS to be able to run this piece in a web browser, as needed for Holo.

Eric and Nico continued to map out the specifics of how these two modules would interface. In that process it became apparent that dividing an agent in these two pieces and have them run on different remote machines has a huge problem:
  * every network communication (including the world model which happens on an ongoing basis) that the back-house is doing has to be signed with the agent's keys
  * the agent's keys are by definition part of the front-house
  * **-> the back-house can't live up to its main accountability without communicating with the front-house and requesting a signature for every packet of communication that might be even triggered from the network/DHT while the user is offline and keys not accessible**

Further conversation including Arthur on May 17th 2018 makes it clear that thinking in terms of two different kinds of agency seems appropriate. We discussed separating the authoring or creative agency from the part that runs validations and holds DHT shards and world model information, and allows the later to proxy for the former, **with separate keys**.

## Decision

We decide to **not** emulate a single agency (as represented by one key) across two remote devices. In other words: we decide to not try to implement distributed agents.

Instead, we solve the initial Holo front-end problem by establishing **two sub-agencies with distinct keys**, where the Holo client's authoring agency explicitly grants proxy rights to a HoloPort's DHT agency.

In other words: the Holo user uses their local key to sign a statement that grants another agent (the HoloPort) to act on their behalf for all the cases needed to have the HoloPort carry the DHT weight for this agent. But technically, it is another agent with its own key.

## Consequences

* The Holo user stays in full control over their own keys and source chain
* We don't need to implement the back-house DHT handling in the Holo browser part
* We don't need to think about how these two pieces interface and interact
* We need to **implement these proxy authorization statements and their handling in Holochain itself** -> it affects routing, metrics and the DHT lookup process (the user's entries can be retrieved from the HoloPorts address -> the lookup mechanism needs to know this somehow)
* A similar mechanism may work for other cases as well, especially **solving the multi-device problem** (a user with laptop, phone and tablet that all are different agents but that all should be treated as the same identity)
