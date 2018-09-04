# Naming things

> There are only two hard things in Computer Science: cache invalidation and naming things.

## Rust naming conventions

If in doubt refer to the Rust conventions.

https://doc.rust-lang.org/1.0.0/style/style/naming/README.html

## Holochain naming conventions

There are gaps where the Rust conventions are either silent or following them
would make things too ambiguous.

### Actions & reducers

- `Action` is `VerbNoun` e.g. `GetEntry`
- `ActionResponse` is `ActionName` e.g. `Action::GetEntry` results in `ActionResponse::GetEntry`
- reducer name is `reduce_action_name` e.g. `reduce_get_entry`

### Actors

- Actor `Protocol` is `VerbNoun` e.g. `PutEntry`
- Result of a `Protocol` is `VerbNounResult` e.g. `PutEntryResult`

### Method names

- method names that get something are the name of the thing being got, e.g. `entry()`
- method names that have side effect other than get are `verb_noun()` e.g. `put_entry()`

### Short names

avoid micro names like `t`, `e`, `h` when `table`, `entry`, `header` would be clearer.

avoid shorthand names like `table` when `table_actor` would be clearer.

it is almost always clearer to use the long name.

in the long run the legibility and unambiguity saves OOMs more time than the typing costs.

one notable exception where short names may be appropriate is when destructuring inline for a very simple test.

in this case there may not be a meaningful name, other than to re-iterate the literal value immediately inline with the variable.

Example:

```rust
for (a, b) in vec![("foo", "bar"), ("foo", "baz")] {
  assert_eq!(
    a,
    some_fn(b),
  );
}
```
