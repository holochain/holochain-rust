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
- a timestamp conveying the moment of creation of the header
- the hash of the "entry" being created
- the "entry type" of the "entry" being created

The meaning and role of entries is covered next.

### Entries

Entries are elements of data which get persisted to the same storage as Headers, yet have a different nature. They represent the substance of the data that is of interest to the end users of Holochain, storing records such as transactions, social media posts, or whatever else app developers come up with. The hash of each entry is stored and referenced in its corresponding header. Every local source chain should have the exact same number of Entries and Headers.

All Entries have a type. Entries can be "system" Entries, or "app" Entries. System Entries are Entries integral to the proper functioning of Holochain. Importantly, there are system Entries written to each local source chain, at the time of its initialization, or "genesis".

#### System Entries
These include firstly, the DNA itself. The DNA is written as the first entry into each local source chain. Importantly, this is what gives everyone totaly confidence in everyone elses chains: the fact that you all know you started out by running the same code (otherwise, the hash of the initial DNA entry would be different). There will be more discussion on this later.

The second Entry to be written to the chain is information about the agent using this source chain. That includes their "public key" which is also their address on the network.

There are also entries created relating to the "Capabilities" features of Holochain, but those will be covered elsewhere.

#### App Entries
Next, there are the "App" Entries. The definitions for these entry types, and how to validate them, are in the hands of developers. When end users are attempting actions which should author data, the data will first be checked against the validation rules supplied by the DNA developer. The possibilities for App Entry type definitions are endless. Defining App Entry types is how you setup the schemas, data structures, and data integrity of your DNA.

#### Entry Sharing
Not every Entry must be shared with other people running the same DNA. There are two (and may be more later) levels of sharing for App Entries defined: `public` and `private`.

With `public`, all Entries of that type will get propogated to the other peers running the same DNA, so that they will hold a copy as well, and make your data more resilient, and so that it is available even when you're offline.

With `private`, the Entry will never leave your device. In order to still be able to verify your local source chain, the HEADER for that Entry will be propogated to other peers, but the Entry itself will not.

## Data Propogation
After any chain-modifying functions are called, Holochain will attempt to notify other peers in the network of those actions. Note that it is not a requirement that the device be online or connected to any other networks or nodes for Holochain to be usable! Holochain will announce the new data over the network, by attempting to publish it to peers. As mentioned previously, if the entry is one of a `private` App Entry type, then only the Header will be published, not the entry itself. Publishing of the data is secondary to being able to author data in the first place, so this will be further elaborated in the [entry validation](./entry_validation.md) article.

## Writing Data

Broadly speaking, writing data is accomplished by writing DNA source code that makes API calls to Holochain to affect the chain. One should only modify chain data by calling Holochain API functions, as there is lots of internal logic that it is instrumental that Holochain perform, for each change.

### Creating Entries


### Updating Entries


### Removing Entries


## Reading Data


### Get an Entry


### Query Local Chain Entries



## Building in Rust: Read & Write Data


### hdk::commit_entry



### hdk::update_entry


### hdk::remove_entry


### hdk::get_entry


### hdk::query
