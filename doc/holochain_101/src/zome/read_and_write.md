# Read & Write Data Operations

## The Local Source Chain: Headers and Entries

At the core of how Holochain is designed is the "local source chain". 
It is the data structure that puts Holochain in the family of "distributed ledger technology".
It functions as an "append-only log", which is a fancy way of saying that data can only ever be written to the log,
and never modified in place, nor removed, without breaking the integrity of the data that follows it in the log. As has been found for many other projects, this is a critical design choice for a distributed technology.

> Note that across the world of crytocurrency and distributed ledgers, these logs are sometimes given different names, despite usually refering to a very similar data structure: feed, log, ledger, and chain are only four such examples. What they share is the append-only nature, as well as the use of cryptographic signatures for checking data integrity.

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

Other data contained in the headers includes:
- the public key/address, and the cryptographically signed signature of the author
- a timestamp conveying the moment of creation of the header
- the hash of the "entry" being created
- the "entry type" of the "entry" being created

The meaning and role of entries is covered next.

### Entries



## Data Propogation



## Writing Data


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
