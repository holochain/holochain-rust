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

### Entry Address

Canonical name: `entry_address`

Returns the address that a given entry will hash into.

### Debug

Canonical name: `debug`

Debug sends the passed arguments to the log that was given to the Holochain instance and returns `None`.


### Call

Canonical name: `call`

TODO

### Sign

Canonical name: `sign`

TODO

### Verify Signature

Canonical name: `verify_signature`

TODO

### Commit Entry

Canonical name: `commit_entry`

Given an entry type and content, commits an entry to the local source chain.
On success, returns the hash of the entry.

### Update Entry

Canonical name: `update_entry`

TODO

### Update Agent

Canonical name: `update_agent`

TODO

### Remove Entry

Canonical name: `remove_entry`

TODO

### Get Entry

Canonical name: `get_entry`

Given an entry hash, returns the entry from the DHT if that entry exists.

Entry lookup is done in the following order:
- The local source chain
- The local hash table
- The distributed hash table

Caller can request additional metadata on the entry such as type or sources
(hashes of the agents that committed the entry).

### Get Links

Canonical name: `get_links`

TODO

### Remove Entry

Canonical name: `remove_entry`

TODO

### Query

Canonical name: `query`

TODO

### Send

Canonical name: `send`

TODO

### Start Bundle

Canonical name: `start_bundle`

TODO

### Close Bundle

Canonical name: `close_bundle`

TODO
