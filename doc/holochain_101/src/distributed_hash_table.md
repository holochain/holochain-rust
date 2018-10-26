# Distributed Hash Table

## Local Hash Table

### implementation details

First, read about [state actors](/state/actors.html).

The 1:1 API implementation between actors and their inner table is achieved by
internally blocking on an `ask` from riker patterns.

https://github.com/riker-rs/riker-patterns

The actor ref methods implementing `HashTable` sends messages to itself.

Calling `table_actor_ref.commit(entry)` looks like this:

0. the actor ref constructs a `Protocol::PutPair` message including the entry
0. the actor ref calls its own `ask` method, which builds a future using riker's `ask`
0. the actor ref blocks on its internal future
0. the referenced actor receives the `Commit` message and matches/destructures this into the entry
0. the entry is passed to the `commit()` method of the inner table
0. the actor's inner table, implementing `HashTable`, does something with commit (e.g. MemTable inserts into a standard Rust, in-memory `HashMap`)
0. the return value of the inner table `commit` is inserted into a `CommitResult` message
0. the `CommitResult` message is sent by the actor back to the actor ref's internal future
0. the actor ref stops blocking
0. the `CommitResult` message is destructured by the actor ref so that the return of `commit` satisfies the `HashTable` trait implementation

Riker `ask` returns a future from the futures `0.2.2` crate.

`table_actor.block_on_ask()` calls `block_on` and `unwrap` against this ask.

Both the block and the unwrap should be handled better in the future.
