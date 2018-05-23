# 5. Alpha1 is last major Go release

Date: 2018-05-16

## Status

Accepted

## Context

A complete Rust rewrite is planned (see ADR #0002)

Go code has code debt -> substantial effort to refactor
Go implementation would not be reusable in Holo front-end (whereas portions of rust compilable into WASM could be)
Go code has dependencies that make it hard to compile for mobile

## Decision

- Alpha1 go release is last major go release of holochain because energy will be focused on the new Rust version.  One team, one code base for now, may revisit this later
- We will call the rust release Alpha2, and will have at least the functionality of Alpha 1 plus World-model & new network-transport

## Consequences

There will still be minor releases for bug fixing.
Move from [holochain-proto waffle](https://waffle.io/Holochain/holochain-proto) -> [org waffle](https://waffle.io/holochain/org)

We realized that this really isn't an architecture decision, but we'll leave it here as documenting our learning of that.  We Probably need another venue for non-architecture i.e software-engineering decisions, i.e. in OrgBook.
