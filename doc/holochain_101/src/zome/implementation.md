# Zome implementation

## Zome API functions

Each zome API function is implemented under `nucleus::ribosome`.

There is a fair bit of boilerplate at the moment, sorry!

To co-ordinate the execution of an API function across Rust and WASM we need to
implement several items:

- An integer index in the `nucleus::ribosome::HcApiFuncIndex` enum
- Map the index to the canonical name for the API function under `nucleus::ribosome::index_canonical_name`
- An invocation dispatch in `nucleus::ribosome::call` under `Externals for Runtime`
- A ribosome module implementing the invocation logic as `invoke_*`
- An action if the zome API function has side effects

### Zome API function index

Simply add the name of the new zome API function to the end of the enum.

DO add a doc comment summarising what the zome function does and sketching the
function signature.

Do NOT add to the start or middle of the enum as that will renumber the other
zome functions.

### Map the canonical name index

Add a mapping from the canonical name to the enum variant in `nucleus::ribosome::index_canonical_name`.

`nucleus::ribosome::call` will automatically resolve the correct function name
once the enum mapping is set.

### Invocation dispatch

Add the match arm for the new enum under `invoke_index`.

It should look something like this:

```rust
index if index == HcApiFuncIndex::Foo as usize => invoke_foo(self, &args),
```

Where `Foo` and `invoke_foo` should replace foo with the canonical name.

### Zome API function ribosome module

Each zome API function should have its own module under `nucleus::ribosome::*`.

Implement a public function as `invoke_<canonical name>`. The function must take
two arguments, a `&mut nucleus::ribosome::Runtime` and a `&wasmi::RuntimeArgs`.

This function will be called by the invocation dispatch (see above).

#### Zome API function arguments

The `wasmi::RuntimeArgs` passed to the Zome API function contains only a single
`u32` value. This is an encoded representation of a single page of memory
supported by the memory manager. The 16 high bits are the memory offset and the
16 low bits are the memory length. See the `wasm_utils` crate for more
implementation details.

You don't have to work with the memory manager directly, simply pass the runtime
and runtime args to `nucleus::runtime_args_to_utf8` to get a utf-8 string from
memory.

You DO have to handle serialization round trips if you want to pass anything
other than a single utf-8 string to a zome API function.

The simplest way to do this is implement a struct that derives `Serialize` and
`Deserialize` from serde, then use serde and `.into_bytes()` co-ordinate the
round trip.

For an example implementation of a struct with several fields see:

- `nucleus::ribosome::commit::CommitArgs` for the input args struct
- `nucleus::ribosome::commit::tests::test_args_bytes` serializing the struct as bytes
- `nucleus::ribosome::commit::invoke_commit` deserializing the struct from the runtime

#### Zome API function action dispatch

If the function has a side effect it must send an action to the state reduction
layer.

Actions are covered in more detail in the state chapter.

In summary, if you want to send an action and wait for a return value:

- create an outer channel in the scope of your invoke function that will receive the return value
- call `::instance::dispatch_action_with_observer` with:
  - the runtime's channels
  - the action the reducer will dispatch on
  - an observer sensor, which is a closure that polls for the action result and sends to your outer channel
- block the outer channel until you receive the action result

#### Zome API function return values

The zome API function returns a value to wasm representing success or a wasm trap.

The success value can only be a single `i32`.

Traps are a low level wasm concern and are unlikely to be directly useful to a
zome API function implementation.

See https://github.com/WebAssembly/design/blob/master/Semantics.md#traps

To get complex values out of wasm we use the memory manager, much like the input
argument serialization (see above).

The util function `nucleus::runtime_allocate_encode_str` takes a string,
allocates memory and returns the value that the zome API function must return.

To return an error relevant to holochain, return `Ok` with an `HcApiReturnCode`
error enum variant.

For an example implementation returning a complex struct see:

- `agent::state::ActionResult::Get` defining a result containing a `Pair` struct
- `nucleus::ribosome::get::invoke_get`
  - match the action result against the correct enum variant
  - serialize the pair using serde
  - return the result of `runtime_allocate_encode_str`
  - if the action result variant does NOT match then return `HcApiReturnCode::ErrorActionResult`

### Zome API function agent action

If the zome API function will cause side effects to the agent state then it must
implement and dispatch an action.

Actions are covered in more detail in the state chapter.

In summary, if a new agent action (for example) is needed:

- extend the `agent::state::Action` enum
  - use the canonical name if that makes sense
  - implement a constructor method in the enum impl
  - include a snowflake ID or there will be key collisions in the state history
- extend the `agent::state::ActionResult` enum if the action has a return value
- implement a function dispatch off the new action in `agent::state::reduce`
- implement the dispatched `agent::state::do_action_*` function
