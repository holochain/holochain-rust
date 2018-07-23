# Zome implementation

## Zome API functions

Each zome API function is implemented under `nucleus::ribosome`.

To co-ordinate the execution of an API function across Rust and WASM we need to
implement several items:

- An integer index in the `nucleus::ribosome::HcApiFuncIndex` enum
- An invocation dispatch in `nucleus::ribosome::call` under `Externals for Runtime`
- The zome API function signature in `nucleus::ribosome::call` under `resolve_func`
- A ribosome module implementing the invocation logic as `invoke_*`

### Zome API function index

Simply add the name of the new zome API function to the end of the enum.

DO add a doc comment summarising what the zome function does and sketching the
function signature.

Do NOT add to the start or middle of the enum as that will renumber the other
zome functions.

### Invocation dispatch

Add the match arm for the new enum under `invoke_index`.

It should look something like this:

```rust
index if index == HcApiFuncIndex::FOO as usize => invoke_foo(self, &args),
```

### Zome API function signature

Define the WASMI function signature under `resolve_func`.

Read more about WASMI: https://github.com/paritytech/wasmi

Signatures are defined by WASMI: https://paritytech.github.io/wasmi/wasmi/struct.Signature.html
As are allowed value types: https://paritytech.github.io/wasmi/wasmi/enum.ValueType.html

Note that the only allowed value types are 32/64 bit integers and floats.

Passing "complex" data types between zome and rust code is handled by JSON
string serialization/deserialization and sending the bytes through WASMI.
