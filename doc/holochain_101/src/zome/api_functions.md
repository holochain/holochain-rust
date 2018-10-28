# Zome API Functions

## Overview

A Zome API Function is any Holochain core functionality that is exposed as a
callable function within Zome code.

Compare this to a Zome Callback Function, which is implemented by the Zome code 
and called by Holochain.

So, Zome functions (functions in the Zome code) are called by Holochain, 
which can optionally call Zome API Functions, and then finally return a
value back to Holochain.

```
Holochain blocks
  -> calls Zome function
  -> executes WASM logic compiled from Zome language
  -> Zome logic calls zome API function
    -> Holochain natively executes Zome API function
    -> Holochain returns value to Zome function
  -> Zome function returns some value
  -> Holochain receives final value of Zome function
```

Each Zome API Function has a canonical name used internally by Holochain.

Zome code can be written in any language that compiles to WASM. This means the
canonical function name and the function name in the Zome language might be
different. The Zome language will closely mirror the canonical names, but naming
conventions such as capitalisation of the zome language are also respected.

For example, the canonical `verify_signature` might become `verifySignature` in
AssemblyScript.

When a Zome API function is called from within Zome code a corresponding Rust
function is called. The Rust function is passed the current Zome runtime and the
arguments that the zome API function was called with. The Rust function connects
Zome logic to Holochain core functionality and often has side effects. The
return value of the Rust function is passed back to the Zome code as the return
of the Zome API function.

### Property

Canonical name: `property`

Returns an application property, which are defined by the developer in the DNA.
It returns values from the DNA file that you set as properties of your application (e.g. Name, Language, Description, Author, etc.).

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.property.html)

### Hash Entry

Canonical name: `hash_entry`

Reconstructs an address of the given entry data. This is the same value that would be returned if an entry type and entry value were passed to the `commit_entry` function and by which it would be retrievable from the DHT using `get_entry`. It is often used to reconstruct an address of a base argument when calling `get_links`. It was renamed `hash_entry` in this version from `make_hash` in the Go version of Holochain.

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.hash_entry.html)

### Debug

Canonical name: `debug`

Debug sends the passed arguments to the log that was given to the Holochain instance and returns `None`.

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.debug.html)

### Call

Canonical name: `call`

Perform a function call to an exposed function from another Zome.

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.call.html)

### Sign

Canonical name: `sign`

Not yet available, but you will see updates here:

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.sign.html)

### Verify Signature

Canonical name: `verify_signature`

Not yet available, but you will see updates here:

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.verify_signature.html)

### Commit Entry

Canonical name: `commit_entry`

Attempts to commit an entry to your local source chain. The entry will have to pass the defined validation rules for that entry type. If the entry type is defined as public, will also publish the entry to the DHT. Returns either an address of the committed entry as a string, or an error.

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.commit_entry.html)

### Update Entry

Canonical name: `update_entry`

Not yet available, but you will see updates here:

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.update_entry.html)

### Update Agent

Canonical name: `update_agent`

Not yet available, but you will see updates here:

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.update_agent.html)

### Remove Entry

Canonical name: `remove_entry`

Not yet available, but you will see updates here:

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.remove_entry.html)

### Get Entry

Canonical name: `get_entry`

Given an entry hash, returns the entry from the DHT if that entry exists.

Entry lookup is done in the following order:
- The local source chain
- The local hash table
- The distributed hash table

Caller can request additional metadata on the entry such as type or sources
(hashes of the agents that committed the entry).

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.get_entry.html)

### Get Links

Canonical name: `get_links`

Not yet available, but you will see
updates here: [LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.get_links.html)

### Remove Entry

Canonical name: `remove_entry`

Not yet available, but you will see updates here:

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.remove_entry.html)

### Query

Canonical name: `query`

Returns a list of addresses of entries from your local source chain, that match a given type. You can optionally limit the number of results.

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.query.html)

### Send

Canonical name: `send`

Not yet available, but you will see updates here:

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.send.html)

### Start Bundle

Canonical name: `start_bundle`

Not yet available, but you will see updates here:

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.start_bundle.html)

### Close Bundle

Canonical name: `close_bundle`

Not yet available, but you will see updates here:

[LINK](https://holochain.github.io/rust-api/0.0.1/hdk/fn.close_bundle.html)

