# Capabilities

## Overview
Holochain uses a modified version of the [capabilities](https://en.wikipedia.org/wiki/Capability-based_security) security model.  Holochain DNA instances will grant revokable, cryptographic capability tokens which are shared as access credentials. Appropriate access credentials must be used to access functions and private data.

This enables us to use a single security pattern for:

- connecting end-user UIs,
- calls across zomes within a DNA,
- bridging calls between different DNAs,
- and providing selective users of a DNA the ability to query private entries on the local chain via send/receive.

Each capability grant gets recorded as a private entry on the grantor’s chain, and are validated against for every zome function call.

## Using Capabilities

As of version 0.0.6-alpha capabilities are not fully implemented. In this version, however you must declare all functions as public using the special `hc_public` marker trait in your `define_zome!` call.  Functions in that trait will be added to the public capability grant which gets auto-committed during genesis, and thus, because other capability grants aren't yet available in 0.0.6-alpha, all Zome functions must be made public.

```
define_zome! {

...

   traits: {
       hc_public [read_post, write_post]
   }

...

}
```
