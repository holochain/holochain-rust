# 4. Have only one WASM Ribosome

Date: 2018-05-16

## Status

Accepted

## Context

There is a WASM intepretor in Rust which can enable hApps in many different languages. WASM is more performant than any other interpretor.


## Decision

Implement only one WASM ribosome for holochain and have it working for 2 different toolchains (Javascript and Rust or C++)

## Consequences

Might do another ribosome non-wasm compatible languages like Closurescript, Python or Ruby
