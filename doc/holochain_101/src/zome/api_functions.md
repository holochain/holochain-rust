# Zome API functions

## Overview

A Zome API Function is any Holochain core functionality that is exposed as a
callable function within zome code.

Compare this to a Zome Callback Function, which is implemented by the zome code 
and called by Holochain.

So, zome functions (functions in the zome code) are called by Holochain, 
which can optionally call Zome API Functions, and then finally return a
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

Each Zome API Function has a canonical name used internally by Holochain.

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

Note: Full reference is available in language-specific API Reference documentation.
(TODO add links)

### Property

Canonical name: `property`

Returns an application property, which are defined by the app developer in the DNA.
It returns values from the DNA file that you set as properties of your application (e.g. Name, Language, Description, Author, etc.).

[LINK](https://holochain.github.io/rust-api/hdk/fn.property.html)

### Make Hash

Canonical name: `make_hash`

Not yet available, but you will see
updates here:
[LINK](https://holochain.github.io/rust-api/hdk/fn.make_hash.html)

### Debug

Canonical name: `debug`

Debug sends the passed arguments to the log that was given to the Holochain instance and returns `None`.

[LINK](https://holochain.github.io/rust-api/hdk/fn.debug.html)

### Call

Canonical name: `call`

Not yet available, but you will see
updates here:
[LINK](https://holochain.github.io/rust-api/hdk/fn.call.html)

### Sign

Canonical name: `sign`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.sign.html)

### Verify Signature

Canonical name: `verify_signature`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.verify_signature.html)

### Commit Entry

Canonical name: `commit_entry`

Given an entry type and content, commits an entry to the local source chain.
On success, returns the hash of the entry.

[LINK](https://holochain.github.io/rust-api/hdk/fn.commit_entry.html)

### Update Entry

Canonical name: `update_entry`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.update_entry.html)

### Update Agent

Canonical name: `update_agent`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.update_agent.html)

### Remove Entry

Canonical name: `remove_entry`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.remove_entry.html)

### Get Entry

Canonical name: `get_entry`

Given an entry hash, returns the entry from the DHT if that entry exists.

Entry lookup is done in the following order:
- The local source chain
- The local hash table
- The distributed hash table

Caller can request additional metadata on the entry such as type or sources
(hashes of the agents that committed the entry).

[LINK](https://holochain.github.io/rust-api/hdk/fn.get_entry.html)

### Get Links

Canonical name: `get_links`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.get_links.html)

### Remove Entry

Canonical name: `remove_entry`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.remove_entry.html)

### Query

Canonical name: `query`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.query.html)

### Send

Canonical name: `send`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.send.html)

### Start Bundle

Canonical name: `start_bundle`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.start_bundle.html)

### Close Bundle

Canonical name: `close_bundle`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/hdk/fn.close_bundle.html)

