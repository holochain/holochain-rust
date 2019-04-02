# Writing a Development Kit

The end goal of a Development Kit is to simplify the experience of writing Zomes that compile to WASM for Holochain apps.

At the time of writing, there is currently one active Developer Kit being written, for the Rust language. While it is possible to look at the [Rust language HDK](https://github.com/holochain/hdk-rust) as a reference, this article is a more general guide that outlines what it takes to build a Development Kit.

If you are interested in supporting developers to write Zomes in an unsupported language, you will want to first of all check whether that language can be compiled to WebAssembly, as that is a requirement.

### Why Development Kits

Development Kits are important because the WASM interface between Zomes and Holochain is constrained to singular 64 bit integers.

The WASM spec allows for multiple function arguments and defines integers as neither signed nor unsigned, but Holochain only supports a single `u64` input and output for all zome functions.

WASM implements a single linear memory of bytes accessible by offset and length.

Holochain sends and receives allocated bytes of memory to zomes by treating the 64 bit integer as two 32 bit integers (high bits as offset and low bits as length).

If no bytes of memory are allocated (i.e. the 32 bit length is 0) the high bits map to an internal enum. This enum is contextual to the zome but typically represents errors:

```rust
pub enum RibosomeErrorCode {
    Unspecified                     = 1 << 32,
    ArgumentDeserializationFailed   = 2 << 32,
    OutOfMemory                     = 3 << 32,
    ReceivedWrongActionResult       = 4 << 32,
    CallbackFailed                  = 5 << 32,
    RecursiveCallForbidden          = 6 << 32,
    ResponseSerializationFailed     = 7 << 32,
    NotAnAllocation                 = 8 << 32,
    ZeroSizedAllocation             = 9 << 32,
    UnknownEntryType                = 10 << 32,
}
```

Each development kit should abstract memory handling in some contextually idiomatic way.

### The Rust Development Kit WASM Solution

The standard development kit implements a simple memory stack.

The `WasmAllocation` struct represents a pair of offset/length `u32` values.

The `WasmStack` struct is a single "top" `u32` value that tracks the current end of linear memory that can be written to (either allocation or deallocation).

Use of these structs is optional inside zome WASM, Holochain core will always write/read according to the input/output position represented by the `u64` arg/return values.

Reads and write methods are provided for both primitive Rust UTF-8 strings and `JsonString` structs.

Write new data to `WasmStack` as `stack.write_string(String)` and `stack.write_json(Into<JsonString>)`.

If the allocation is successful a `WasmAllocation` will be returned else an `AllocationError` will result.

Allocation to the stack can be handled manually as `stack.allocate(allocation)` and the next allocation can be built with `stack.next_allocation(length)`.

Allocation on the stack will fail if the offset of the new allocation does not match the current stack top value.

To read a previous write call `let s = allocation.read_to_string()` and standard `let foo: Result<Foo, HolochainError> = JsonString::try_from(s)` for JSON deserialization.

To write a deallocation call `stack.deallocate(allocation)`.

Deallocation does not clear out WASM memory, it simply moves the top of the stack back to the start of the passed allocation ready to be overwritten by the next allocation.

Deallocation will fail if the allocation offset + length does not equal the current stack top.

Holochain compatible encodings of allocations for the return value of zome functions can be generated with `allocation.as_ribosome_encoding()`.

The development kit:

- Implements the simple stack/allocation structs and methods
- Manages a static stack for consistent writing
- Exposes convenience functions for the Holochain API to handle relevant allocation/deallocations
- Maps `u64` values to/from encoded error values and `u32` offset/length values for memory allocations

For more details review the unit/integration test suites in `hdk-rust` and `wasm_utils`.

### Crafting the API

Using its WASM interpreter, Holochain exposes its callable Zome API functions by making them available as "imports" in Zome WASM modules. Per the memory discussion above, each of the Zome API functions have the same explicit function signature, but different implicit function signatures. The native functions have each been given a prefix so that Development Kit wrappers can expose a regular function name. Here is a complete list:

- hc_debug
- hc_call
- hc_sign
- hc_verify_signature
- hc_commit_entry
- hc_update_entry
- hc_update_agent
- hc_remove_entry
- hc_get_entry
- hc_link_entries
- hc_query
- hc_send
- hc_start_bundle
- hc_close_bundle

There is a special additional one called `hc_init_globals` which we will discuss further.

The Development Kit should implement and export one function per each native function from the list. The function should be called the same as its native form, but without the prefix. E.g. `hc_update_agent` should be called `update_agent` or `updateAgent`. That function should internally call the native function and handle the additional complexity around that.

In order to call these "external" functions, you will need to import them and provide their signature, but in a WASM import compatible way. In Rust, for example, this is simply:
```rust
extern {
  fn hc_commit_entry(encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits;
}
```

TODO: define or link to meaningful function signatures

### Working with WASM Memory

The goal of the Development Kit is to expose a meaningful and easy to use version of the API functions, with meaningful arguments and return values. There is a bit of flexibility around how this is done, as coding languages differ. However, the internal process will be similar in nature. Here it is, generalized:
1. declare, or use a passed, single page 64 KiB memory stack
2. join whatever inputs are given into a single serializable structure
3. serialize the given data structure as an array of bytes
4. determine byte array length
5. ensure it is not oversized for the stack
6. allocate the memory
7. write the byte array to memory
8. create an allocation pointer for the memory
  a. use a 16 bit integer for the pointers `offset`
  b. use a 16 bit integer for the pointers `length`
9. join the pointers into a single 32 bit integer
  a. high bits are `offset`
  b. low bits are `length`
10. call the native function with that 32 bit integer and assign the result to another 32 bit integer
  a. e.g. `encoded_alloc_of_result = hc_commit_entry(encoded_alloc_of_input)`
11. deconstruct that 32 bit integer into two variables
  a. use a 16 bit integer for the pointers `offset`
  b. use a 16 bit integer for the pointers `length`
12. read string data from memory at the `offset` address
13. deallocate the memory
14. deserialize the string to JSON if JSON is expected

That looks like a lot of steps, but most of this code can be shared for the various functions throughout the Development Kit, leaving implementations to be as little as 5 lines long. Basically, the process inverts at the point of the native function call.

#### WASM Single Page Stack

TODO

### App Globals

When writing Zome code, it is common to need to reference aspects of the context it runs in, such as the active user/agent, or the DNA address of the app. Holochain exposes certain values through to the Zome, though it does so natively by way of the `hc_init_globals` function mentioned. Taking care to expose these values as constants will simplify the developer experience.

This is done by calling `hc_init_globals` with an input value of 0. The result of calling the function is a 32 bit integer which represents the memory location of a serialized JSON object containing all the app global values. Fetch the result from memory, and deserialize the result back into an object. If appropriate, set those values as exports for the Development Kit. For example, in Rust, values become accessible in Zomes using `hdk::DNA_NAME`. It's recommended to use all capital letters for the export of the constants, but as they are returned as keys on an object from `hc_init_globals` they are in lower case. The object has the following values:
- dna_name
- dna_address
- agent_id_str
- agent_address
- agent_initial_hash
- agent_latest_hash
- public_token

See the [API global variables](/zome/api_globals.html) page for details on what these are.

### Publish It and Get In Touch

If you've made it through the process so far, good work. The community is an important part of the success of any project, and Holochain is no different. If you're really proud of your work, get in touch with the development team on the [chat server](https://chat.holochain.net/appsup/channels/hc-core), mention you're working on it, and request help if necessary. This book could be updated to include links to other HDKs. Whether you would like to, or you'd like the team to, the HDK could be published to the primary package manager in use for the language, to be used by developers around the world. For example, RubyGems for Ruby or npm for nodejs.
