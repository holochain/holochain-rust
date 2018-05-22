# 3. Redux architecture pattern

Date: 2018-05-16

## Status

Accepted

## Context

We are doing a rewrite.

Holochain Go code shows many implicit dependencies between different modules and stateful objects. In conjunction with the complexity of a p2p network of agents, this leads to a level of overall complexity that feels too much to manage. A clean and fitting architecture for this Rust rebuild is needed.

Having a single global state within the agent feels appropriate and even balancing the distributed nature of the network of agents.

## Decision

The new holochain architecture will follow a redux architecture in order for an agent to have one global state.

We will apply nested state objects which represent a state tree, with sub states for each module.

We use reference counting smart pointers for the sub
states such that it is possible for each module's
reducer to decide if the sub state is to be mutated or reused.

## Consequences

Holochain refactor must fit this new model of having State objects and Actions objects.

Each module of the Holochain agent only needs to depend on the state and not on other modules, which helps decoupling and thus reduces code dependencies.

This will also make it easy to have time-machine debugging capabilities (by storing old states), and easy logging and (remote) monitoring of agents (by sending serialized state objects to monitor client).

The processing of redux-like actions through each modules reducer might cause a constant performance overhead.
