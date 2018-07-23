# Zome API functions

## Overview

A zome API function is any Holochain core functionality that is exposed as a
callable function within zome code.

Compare this to a zome function, which is a function implemented in the zome
language and called by Holochain.

So, zome functions are called by Holochain, which execute logic in the zome
language and can optionally call zome API functions, which finally return a
value back to Holochain.

```
Holochain blocks
  -> calls zome function
  -> executes WASM logic compiled from zome language
  -> zome logic calls zome API function
    -> Holochain natively executes zome API function
    -> Holochain returns value to zome function
  -> zome function returns some value
  -> Holochain receives final value of zome function
```

Each zome API function has a canonical name used internally by Holochain.

Zome code can be written in any language that compiles to WASM. This means the
canonical function name and the function name in the zome language might be
different. The zome language will closely mirror the canonical names, but naming
conventions such as capitalisation of the zome language are also respected.

For example, the canonical `verify_signature` might become `verifySignature` in
JavaScript.

When a zome API function is called from within zome code a corresponding Rust
function is called. The Rust function is passed the current zome runtime and the
arguments that the zome API function was called with. The Rust function connects
zome logic to Holochain core functionality and often has side effects. The
return value of the Rust function is passed back to the zome code as the return
of the zome API function.

## Reference

### Print

Canonical name: `print`

Print pushes the passed arguments to the `print_output` of the runtime and
returns `None`.

### Commit

Canonical name: `commit`

Given an entry type and content, commits to the local source chain and returns
the hash key if successful. The hash key is the hash of the header associated
with this commit in the source chain.

### Get

Canonical name: `get`
