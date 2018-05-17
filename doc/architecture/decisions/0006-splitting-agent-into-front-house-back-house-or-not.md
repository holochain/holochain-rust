# 6. Splitting agent into front-house / back-house - OR NOT

Date: 2018-05-16

## Status

proposed

## Context

For Holo, we need to have user agent's keys and source chain on the client machine and the rest of the Holochain agent (the DHT shard etc.) be held by HoloPorts.

In February 2018, Arthur, Eric and Nico discussed this during a co-creative session in London and made the assumption to have the Holochain agent be split up into two pieces, called front-house and back-house (prior left and right hemisphere). The front-house was meant to entail the source chain and private key management as well as a ribosome to run the app and provide interfaces for the UI and in the non-Holo case for bridging - everyhting that is user/agent facing. The back-house should have been the DHT shard, routing table, metrics, etc. Basically everything that is network facing.

With this separation, the reasoning in February was to rewrite (only) the front-house in JS to be able to run this piece in a web browser, as needed for Holo.

Eric and Nico continued to map out the specifics of how these two modules would interface. In that process it became apparent that dividing an agent in these two pieces and have them run on different remote machines has a huge disadvantage:
  * every network communication that the back-house is doing has to be signed with the agent's keys
  * the agent's keys are by definition part of the front-house
  * -> the back-house can't live up to its main accountability without communicating with the front-house and requesting a signature for every packet of communication

## Decision

We decide to not separate the Holochain agent into two pieces - at least not with the notion of them being a distributed agent.

In other words: we decide to not try to implement distributed agents - every Holochain agent must be complete locally.

Instead, we solve the initial Holo front-end problem by establishing an identity that transcends agents and basically treat both the slim Holo browser front-house and the agent run on the HoloPort as full agents, that both act under the same identity.

The Holo user uses their local key to sign a statement that grants another agent (the HoloPort) to act on their behalf for all the cases needed to have the HoloPort carry the DHT weight for this agent. But technically, it is another agent with its own key.

## Consequences

* The Holo user stays in full control over their own keys and source chain
* We don't need to implement the back-house DHT handling in the Holo browser part
* We need to implement these authorization statements and their handling in Holochain itself -> it affects routing, metrics and the DHT lookup process (the user's entries can be retrieved from the HoloPorts address -> the lookup mechanism needs to know this somehow)
