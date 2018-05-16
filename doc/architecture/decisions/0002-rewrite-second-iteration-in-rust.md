# 2. Rewrite second iteration in Rust

Date: 2018-05-16

## Status

Accepted

## Context

We want to have holochain run in the browser (for Holo).

Rust can compile to WASM. Rust is language for experts. Rust is trending.

There is a WASM interpretor in Rust (WASMI).

## Decision

Refactor Holochain in the Rust programming language

## Consequences

We have to recode all holochain in Rust which will delay the next version release.

It is an opportunity to refactor the architecture.

We must have clear coding practice on how we manage memory ownership in Rust.
