# 3. Redux architecture pattern

Date: 2018-05-16

## Status

Accepted

## Context

Context here...

## Decision

The new holochain architecture will follow a redux architecture in order for an agent to have one global state.  This reduce the complexity of handling state management which is already complex for a distributed app.

## Consequences

Holochain refactor must fit this new model of having State objects and Actions objects.
