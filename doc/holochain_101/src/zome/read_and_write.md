# Read & Write Data Operations

## The Local Source Chain: Headers and Entries

At the core of how Holochain is designed is the "local source chain".
It is the data structure that puts Holochain in the family of "distributed ledger technology".
It functions as an "append-only log", which is a fancy way of saying that data can only ever be written to the log,
and never modified in place, nor removed, without breaking the integrity of the data that follows it in the log. As has been found for many other projects, this is a critical design choice for a distributed technology.

> Note that across the world of crytocurrency and distributed ledgers, these logs are sometimes given different names, despite usually refering to a very similar data structure: feed, log, ledger, and chain are only four such examples. What they share is the append-only nature, as well as the use of cryptographic signatures for securing data integrity.

As a consequence of this log-like data structure, Holochain is a departure from traditional relational
(SQL) and also non-relational (noSQL) databases. There are no tables, and no foreign keys, at least on the formal level of Holochain.
Despite this, one must of course still be able to persist data, and read back persisted data, and all of that happens in relation to the local source chain.

With Holochain, there are two aspects that combine to form each "element" in this chain-like log: headers, and entries.

### Headers

The fundamental chain-like aspect of Holochain is driven by "headers". They are a minimal data structure containing only a handful of values, but are sufficient to supply Holochain with verifiable data integrity.

Fundamental to creating the "chain", each header contains a reference to the "last" header. There are a couple aspects to this:
1. The cryptographic hash (its data "fingerprint") for a header is determined
2. The hash of the previously written header is used as a fixed-length input for the next header, thus influencing the hash of that new header (since any change to contents change a hash)
3. The only time this does not apply is for the very first header, for which there is no previous header hash, and a value of `0` is used

Other data contained in the headers includes (but isn't limited to):
- the public key/address, and the cryptographically signed signature of the author
- a timestamp conveying the moment of creation of the header, from the perspective of the author's system clock
- the hash of the "entry" being created
- the "entry type" of the "entry" being created

The meaning and role of entries is covered next.

### Entries

Entries are elements of data which get persisted to the same storage as Headers, yet have a different nature. They represent the substance of the data that is of interest to the end users of Holochain, storing records such as transactions, social media posts, or whatever else app developers come up with. The hash of each entry is stored and referenced in its corresponding header. Every local source chain should have exactly one Entry for every Header.

All Entries have a type. Entries can be "system" Entries, or "app" Entries. System Entries are Entries integral to the proper functioning of Holochain. Importantly, there are system Entries written to each local source chain, at the time of its initialization.

#### System Entries
These include firstly, the DNA itself. The DNA is written as the first entry into each local source chain. Importantly, this is the foundation for establishing confidence in the data of everyone's chains: the fact that you all know you started out by running the same code (otherwise, the hash of the initial DNA entry would be different). There will be more discussion on this later.

The second Entry to be written to the chain is information about the agent using this source chain. That includes their "public key" which is also their address on the network.

There are also other system entry types which are automatically created to record the occurrence of certain actions such as the creation or removal of a link, the marking of an existing entry as deleted, or the granting or revoking of a capability token. These will be covered in more detail in later chapters.

#### App Entries
Next, there are the "App" Entries. The definitions for these entry types, and how to validate them, are in the hands of developers. When end users are attempting actions which should author data, the data will first be checked against the validation rules supplied by the DNA developer. The possibilities for App Entry type definitions are endless. Defining App Entry types is how you setup the schemas, data structures, and data integrity of your DNA.

#### Entry Sharing
Not every Entry must be shared with other people running the same DNA. There are two (and may be more later) levels of sharing for App Entries defined: `public` and `private`.

With `public`, all Entries of that type will get propagated to the other peers running the same DNA, so that they will hold a copy as well, and make your data more resilient, and so that it is available even when you're offline.

With `private`, the Entry will never leave your device. In order to still be able to verify your local source chain, the HEADER for that Entry will be propagated to other peers, but the Entry itself will not.

## Data Propagation
After any chain-modifying functions are called, Holochain will attempt to notify other peers in the network of those actions. Note that it is not a requirement that the device be online or connected to any other networks or nodes for Holochain to be usable! Holochain will persist the changes locally to be gossiped to connected peers at a later date. As mentioned previously, if the entry is one of a `private` App Entry type, then only the Header will be published, not the entry itself. Publishing of the data is secondary to being able to author data in the first place, so this will be further elaborated in the [entry validation](./entry_validation.md) article.

## Writing Data
Broadly speaking, writing data is accomplished by writing Zome source code that makes API calls to Holochain to affect the local source chain. One should only modify chain data by calling Holochain API functions, as there is lots of internal logic that it is instrumental that Holochain perform, for each change. When an API function to alter the chain from Zome source code is invoked, a series of steps is performed internally. It is important to know, at least roughly, what those steps are so that you can use Holochain effectively.

Apart from the following three functions, there is only one additional way that data can be written to the local source chain, which is written about in [linking](./linking.md).

### Creating Entries, or "Committing"
Of course, first and foremost, there is the simple action of writing a new Entry. This is actually known as "committing" an Entry. It is known as "committing" an Entry in Holochain for precisely the reason that once you write it, you can't "unwrite" it, or delete it, without corrupting the integrity of your local source chain. It is there for good, and can only be "marked" as updated or deleted by writing additional entries in the future.

Because the Entry will be permanent in your local source chain, it must first pass validation, in order to be written in the first place. Validation of Entries is core to the distributed data integrity model of Holochain.

Invoking the commit entry API function will not always return "success". The call can fail if the Entry fails to pass validation, or something else goes wrong internally while Holochain is attempting to write the Entry to the local source chain. Like in any code, failure tolerant code is far better, and Zomes should be written to handle possible error cases.

So in general, the process that Holochain follows while trying to write an Entry during a "commit" call is this:

1. Collects data necessary for use in validation (a "validation package")
2. Checks whether the new entry is valid by passing it through the validation callback specified in the Zome for its given entry type. If this fails, it will exit here and return either a user defined, or system level error message
3. Creates a new Header for the Entry, and writes it to storage
4. Writes the Entry itself to storage
5. Announces over the network module this new Header and Entry, requesting that peers validate and hold a copy of it. This is known internally as "publishing".
6. Returns the "address" of the new Entry, which is the crytographic hash of the Entry data combined with its entry type string. This address can be used later to retrieve the Entry.


### Updating Entries

The act of updating an Entry is technically speaking just the same "committing an Entry", plus storing a bit of metadata indicating that this new entry is an "update" to the entry at an old address which is also given. This has the effect that when an attempt to retrieve an Entry by its address, it will forward the request along to the latest version in a potential series of updates, and return that instead. There is always the option to return the results for an Entry according to whether it's the "initial" version, the "latest", or getting the entire history too.

So to update an Entry, use the address of that entry, and provide the new Entry to replace it. You can use the address of the Entry at any point in its update history as the address to update (as long as it hasn't been marked deleted), and it will still work, technically updating the very latest version of that Entry instead of whatever you pass in.

So in general, the process that Holochain follows while trying to update an Entry during an "update_entry" call is this:

1. Retrieves the very latest version of the Entry at the given address
2. Collects data necessary for use in validation (a "validation package")
3. Checks whether the new entry is valid by passing it through the validation callback specified in the Zome for its given entry type. If this fails, it will exit here and return either a user defined, or system level error message
4. Creates a new Header for the Entry, and writes it to storage
5. Writes the Entry itself to storage
6. Announces over the network module this new Header and Entry, requesting that peers validate and hold a copy of it. This is known internally as "publishing". **This step also involves updating the metadata for the Entry at the old address such that default requests for it will forward to the new Entry.**
7. Returns the "address" of the new Entry, which is the crytographic hash of the Entry data combined with its entry type string. This address can be used later to retrieve the Entry.


### Removing Entries

The goal of removing an Entry is so that it will not automatically show up as a result when attempts to retrieve it are made, it is NOT to delete it entirely. Removing an Entry will not prevent someone who wishes to retrieve it from retrieving it, they will just have to pass a special option to do so. This makes sense due to the "append-only" nature of Holochain: the original Entry is never gone.

To remove an Entry, Holochain actually commits an Entry of a special type: `DeletionEntry`.

So in general, the process that Holochain follows while trying to remove an Entry during an "remove_entry" call is this:

1. Retrieves the very latest version of the Entry at the given address
2. Collects data necessary for use in validation (a "validation package")
3. Creates a new `DeletionEntry`, containing the address of the very latest version of the given entry
4. Checks whether the `DeletionEntry` is valid by passing it through the validation callback specified in the Zome for its given entry type. If this fails, it will exit here and return either a user defined, or system level error message
5. Creates a new Header for the Entry, and writes it to storage
6. Writes the Entry itself to storage
7. Announces over the network module this new Header and Entry, requesting that peers validate and hold a copy of it. This is known internally as "publishing". **This step also involves updating the metadata for the Entry at the old address such that default requests for it will return no Entry.**
8. Returns a null value, as there is no need to retrieve the new `DeletionEntry` at any point for any reason.



## Reading Data

Reading data is a lot more straightforward than writing data. Data is either read from your own device, if it lives there, or is fetched from peers over network connections. Throughout the writing data section, it was mentioned that Entries and even Headers have "addresses". These addresses are linked to the content itself, following a pattern known as ["content addressable storage"](https://en.wikipedia.org/wiki/Content-addressable_storage).

This means that to determine the address, the content is passed through a cryptographic hash function, which will deterministically give the same hash whenever the same content is given, and can take an input of any length, and return a result of fixed length. The content is then deposited in a location on a computer, and a network, where it can be retrieved given its address/hash.

Only Entries may be retrieved by their address, not Headers.


### Get an Entry


### Query Local Chain Entries



## Building in Rust: Read & Write Data


### hdk::commit_entry


### hdk::update_entry


### hdk::remove_entry


### hdk::get_entry


### hdk::query
