# 9. Support 64bit architectures only

Date: 2018-05-16

## Status

Accepted

## Context

Going forward with the rust implementation, we recognize that some 32bit architectures exist that people may want to run Holochain on.  Supporting 32bit architectures may have particular consequences in the realm of cryptography.  We have limited resources.

## Decision

For now we will assume availability 64bit CPUs and not use our resources testing against 32bit targets.

## Consequences

This may limit us to certain low-power/low-cost environments where we would ideally like to see Holochain available.
