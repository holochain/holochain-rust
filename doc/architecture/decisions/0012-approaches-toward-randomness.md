# 12. Generalized Randomness for Holochain applications
Date: 2018-06-5

## Status
Accepted

## Context
Distributed applications, like centralized ones, often need a source of randomness.  Having confidence and verifiability of that randomness poses particular challenges in the the distributed context. Specifically, we need a source of randomness with some of the following properties:
 1. It is outside their control or influence so it can be used for a dice roll, card deck shuffle, or something where they may have an interest in skewing the results.
 2. It is predictably reproducable, so that other nodes, whether players in a game, or validating nodes later, can reproduce the SAME random seed to reach the same random output for purposes of validation.
 3. It is generalizable. Ideally, every application won't have to build their own approach to randomness, but can take advantage of a useful underlying convention. There are pitfalls we want to help them avoid in a distributed, eventually consistent system.
 4. It doesn't require specific parties to be online to access the randomness data, so that later validators can confirm it even if parties with private entries are not online or no longer part of the network.

In the case of multiple parties wanting to generate randomness together, [cointoss](https://github.com/holochain/cointoss) provides an example of sharing of the hash of a secret which when later revealed can be combined to create a random number seed.  This method can be generalized into storing a bunch of private secrets, and publishing the hashes of those secrets, and then later revealing the secret to be combined with another party doing the same.  In cointoss this revelation happens via node-to-node communication, but in the general case it doesn't have to work that way.

Our application environment includes interactions (gossip and validation)  the combination of which are highly unpredicable (they include things like network latency, and timestamps) but verifiable after the fact. So for example, using the "first" four validation signatures on your opponent's last move as a random seed, could be one approach.

## Decision
We will:
1. Implement mixins to provide randomness generation for some usecases using the cointoss combined secret method (both n2n for single events, and dht pushed for multiple events)
2. Provide app level access to unpredictable gossip/validation events for explicit use as seeds for random number generators.

## Consequences

- We will want to have our approach reviewed as proof of the validity of these approaches  (i.e. this should get included in the security review(s))
- We need to add protocol/calls/callbacks in the network abstraction layer (See ADR 7) to gain access to this randomness.
