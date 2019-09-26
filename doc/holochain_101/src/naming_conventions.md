# Naming things

> There are only two hard things in Computer Science: cache invalidation and naming things.

## Rust naming conventions

If in doubt refer to the Rust conventions.

https://doc.rust-lang.org/1.0.0/style/style/naming/README.html

## Holochain naming conventions

There are gaps where the Rust conventions are either silent or following them
would make things too ambiguous.

### Actions & reducers

- `Action` is `VerbNoun` or `Verb` if there is no available noun and matches the underlying function e.g. `GetEntry`
- `ActionResponse` is `ActionName` e.g. `Action::QueryEntry` results in `ActionResponse::GetEntry`
- reducer name is `reduce_action_name` e.g. `reduce_get_entry`

### Actors & protocols

- Actor `Protocol` is `VerbNoun` or `Verb` if there is no available noun and matches the underlying function e.g. `PutEntry` or `Setup`
- Result of a `Protocol` is `VerbNounResult` or `VerbResult` e.g. `PutEntryResult` or `SetupResult`

### Method names

- method names that access something directly "for free" are the name of the thing being accessed, e.g. `entry()`
- method names that have side effects or an expensive lookup are `verb_noun()` e.g. `put_entry()`

### Short names

avoid micro names like `t`, `e`, `h` when `table`, `entry`, `header` is clearer.

avoid shorthand names like `table` when `table_actor` is clearer.

in the long run the legibility and unambiguity saves orders of magnitude more time than the typing costs.
