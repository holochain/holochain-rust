# Logging

Idiomatic Rust logging implementation.


This PR is related to [holochain/lib3h#29](https://github.com/holochain/lib3h/issues/29) and implement a more idiomatic logging.

Requirements can be found [here](https://hackmd.io/MP5F3UhSTp2iPk37Cwa-fw).

It still misses:

* documentation: will be updated once I get some review from Core
* toml deserialization: will add it once I'm sure this implementation meets the requirements of bot HC Core & Networking.
* configurable output channel: this is more tricky than is sounds because Write is not safe to share between thread, but I have some ideas how to implement at least logging to file.


## Example ##

To have a generale view of the capability of this crate, you can run this command:

```bash
cargo run --example simple
```
