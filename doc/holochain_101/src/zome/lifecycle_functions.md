# Lifecycle functions

## Overview

A lifecycle function is implemented in the zome language and called by
Holochain.

Contrast this to a zome API function that is implemented by Holochain and called
by the zome.

Lifecycle functions are passed some parameters and expected to return one of
three possible `LifecycleFunctionParams` variants:

- `Pass`: The lifecycle function has executed/validated successfully
- `NotImplemented`: The lifecycle function was not found in the current zome
- `Fail(String)`: The lifecycle function failed for the given reason

As per Zome API functions, the names of the lifecycle functions may be slightly
different depending on the language. The canonical name follows Rust naming
conventions but other languages may vary these (e.g. camel casing).

To implement a lifecycle function in a zome simply define it and Holochain will
call it automatically during standard internal workflows.

Return `true` or an empty string from the zome function to `Pass`.

Return `false` or a non-empty string from the zome function to `Fail`. The
string will be used as the `Fail` reason.

## Reference

### Genesis

Canonical name: `genesis`
Parameters: none

Called the first time a dApp initializes.

`Pass`: the dApp will initialize
`NotImplemented`: the dApp will initialize
`Fail` (any reason): the dApp will NOT initialize

### ValidateCommit

Canonical name: `validate_commit`
Parameters: The candidate entry to be committed

Called internally by the `commit` Zome API function.

`Pass`: the entry will be committed
`NotImplemented`: the entry will be committed
`Fail` (any reason): the entry will NOT be committed and `commit` will return a
`HcApiReturnCode::ErrorLifecycleResult` error code.
