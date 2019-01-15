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
Holochain "blocks" (meaning it pauses further execution in processor threads)
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

Not Yet Available.

Returns an application property, which are defined by the developer in the DNA.
It returns values from the DNA file that you set as properties of your application (e.g. Name, Language, Description, Author, etc.).

### Entry Address

Canonical name: `entry_address`

Returns the address that a given entry will hash into. Often used for reconstructing an address for a "base" when calling [get_links](#get-links).

[View it in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.entry_address.html)

### Debug

Canonical name: `debug`

Debug sends the passed arguments to the log that was given to the Holochain instance and returns `None`.

[View it in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.debug.html)

### Call

Canonical name: `call`

Enables making function calls to an exposed function from another app instance via bridging, or simply another Zome within the same instance.

[View it in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.call.html)

### Sign

Canonical name: `sign`

Not yet available.

Enables the signing of some piece of data, with the private keys associated with the acting agent.

### Verify Signature

Canonical name: `verify_signature`

Not yet available.

A "signature" is a piece of data which claims to be signed by the holder of a private key associated with a public key.
This function allows that claim to be verified, when given a "signature" and a public key.

### Commit Entry

Canonical name: `commit_entry`

Attempts to commit an entry to your local source chain. The entry will have to pass the defined validation rules for that entry type. If the entry type is defined as public, it will also publish the entry to the DHT. Returns either an address of the committed entry as a string, or an error.

[View it in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.commit_entry.html)

### Update Entry

Canonical name: `update_entry`

Commit an entry to your local source chain that "updates" a previous entry, meaning when getting the previous entry, the updated entry will be returned. update_entry sets the previous entry's status metadata to Modified and adds the updated entry's address in the previous entry's metadata. The updated entry will hold the previous entry's address in its header, which will be used by validation routes.

[View it in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.update_entry.html)

### Update Agent

Canonical name: `update_agent`

Not yet available.

### Remove Entry

Canonical name: `remove_entry`

Enables an entry, referred to by its address, to be marked in the chain as 'deleted'. This is done by adding a new entry
which indicates the deleted status of the old one. This will change which types of results that entry would then show up in,
according to its new 'deleted' status. It can still be retrieved, but only if specifically asked for.

[View it in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.remove_entry.html)

### Get Entry

Canonical name: `get_entry`

Given an entry hash, returns the entry from a chain or DHT if that entry exists.

Entry lookup is done in the following order:
- The local source chain
- The local hash table
- The distributed hash table

Caller can request additional metadata on the entry such as type or sources
(hashes of the agents that committed the entry).

- [View get_entry in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.get_entry.html)
- [View get_entry_initial in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.get_entry_initial.html)
- [View get_entry_history in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.get_entry_history.html)
- [View get_entry_result in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.get_entry_result.html)


### Get Links

Canonical name: `get_links`

Consumes two values, the first of which is the address of an entry, base, and the second of which is a string, tag, used to describe the relationship between the base and other entries you wish to lookup. Returns a list of addresses of other entries which matched as being linked by the given tag. Links are created in the first place using the Zome API function [link_entries](#link-entries). Once you have the addresses, there is a good likelihood that you will wish to call [get_entry](#get-entry) for each of them.

- [View get_links in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.get_links.html)
- [View get_links_and_load in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.get_links_and_load.html)
- [View get_links_result in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.get_links_result.html)
- [View get_links_with_options in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.get_links_with_options.html)


### Link Entries

Canonical name: `link_entries`

Consumes three values, two of which are the addresses of entries, and one of which is a string that defines a relationship between them, called a `tag`. Later, lists of entries can be looked up by using `get_links`. Entries can only be looked up in the direction from the `base`, which is the first argument, to the `target`, which is the second.

[View it in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.link_entries.html)

### Query

Canonical name: `query`

Returns a list of addresses of entries from your local source chain, that match a given entry type name, or a vector of names. You can optionally limit the number of results, and you can use "glob" patterns such as "prefix/*" to specify the entry type names desired.

[View it in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.query.html)

### Send

Canonical name: `send`

Sends a node-to-node message to the given agent. This works in conjunction with the receive callback, which is where the response behaviour to receiving a message should be defined. This function returns the result from the receive callback on the other side.

[View it in the Rust HDK](https://developer.holochain.org/api/0.0.3/hdk/api/fn.send.html)

### Start Bundle

Canonical name: `start_bundle`

Not yet available.

### Close Bundle

Canonical name: `close_bundle`

Not yet available.

