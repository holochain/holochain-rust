# Zome implementation

## Zome API functions

Each zome API function is implemented under `nucleus::ribosome`.

To co-ordinate the execution of an API function across Rust and WASM we need to
implement several items:

- An integer index in the `nucleus::ribosome::HcApiFuncIndex` enum
- An invocation dispatch in `nucleus::ribosome::call` under `Externals for Runtime`
- The zome API function signature in `nucleus::ribosome::call` under `resolve_func`
- A ribosome module implementing the invocation logic as `invoke_*`
- An agent action if the zome API function has side effects
  - `do_action_*` function
  - dispatch from reducer
  - `ActionResult`
  - `Action` variant + method
- Add a gets hashmap to actaully store the state
- write a closure that is a sensor to detect the new get and push back to the calling scope

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

Where `FOO` and `invoke_foo` should replace `foo` with the canonical name.

### Zome API function signature

Define the WASMI function signature under `resolve_func`.

Read more about WASMI: https://github.com/paritytech/wasmi

Signatures are defined by WASMI: https://paritytech.github.io/wasmi/wasmi/struct.Signature.html
As are allowed value types: https://paritytech.github.io/wasmi/wasmi/enum.ValueType.html

Note that the only allowed value types are 32/64 bit integers and floats.

Passing "complex" data types between zome and rust code is handled by JSON
string serialization/deserialization and sending the bytes through WASMI.

### Zome API function ribosome module

Each zome API function should have its own module under `nucleus::ribosome::*`.

Implement a public function as `invoke_<canonical name>`. The function must take
two arguments, a `&mut nucleus::ribosome::Runtime` and a `&wasmi::RuntimeArgs`.

This function will be called by the invocation dispatch (see above).

### Zome API function agent action

If the zome API function will cause side effects to the agent state then it must
implement and dispatch an action.

Actions are covered in more detail in the agent chapter.

In summary, if a new agent action is needed:

- extend the `agent::Action` enum (with the canonical name if that makes sense)
- implement the new enum in `agent::reduce`
