# Writing a Development Kit

The end goal of a Development Kit is to simplify the experience of writing Zomes that compile to WASM for Holochain apps.

At the time of writing, there is currently one active Developer Kit being written, for the Rust language. While it is possible to look at the [Rust language HDK](https://github.com/holochain/hdk-rust) as a reference, this article is a more general guide that outlines what it takes to build a Development Kit.

If you are interested in supporting developers to write Zomes in an unsupported language, you will want to first of all check whether that language can be compiled to WebAssembly, as that is a requirement.

### Why Development Kits

Development Kits are important because the WASM interface between Zomes and Holochain is really constrained. Because of WASMs design, WASM functions may only be called with 32 bit integers. Holochain implements a solution to this, but if app developers were to always have to interact with this solution directly, it would feel very complex. A Development Kit for each language should ideally be developed so that it gets so much simpler!

### The Development Kit WASM Solution

To enable passing arguments more complex than 32 bit integers between Zomes and Holochain, a pattern of utilizing WASM memory is used. When it is running the WASM code for a Zome, Holochain has access to both read and write from the WASM memory.

The pattern defines that Holochain Zome API functions expect to both give and receive 32 bit integers which actually represent a WASM memory location. So to provide a Holochain Zome API function a complex argument, one must first write it into memory, and then call the function, giving it the memory location. Holochain will pull the argument from memory, execute its behaviour, store the result in memory, and return the memory location of the result. The Zome code then has to *also* lookup the result by its location in memory.

Technically, an app developer can do all of these things if they have a reason to, but most won't want to handle the extra step involving memory. A Development Kit, then, should handle the extra step of writing to memory, and calling the native API function, and reading the result from memory, and returning that instead. Plus a few other sprinkles on top.

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
  fn hc_commit_entry(encoded_allocation_of_input: u32) -> u32;
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

See the [API global variables](/zome/api_globals.html) page for details on what these are.

### Publish It and Get In Touch

If you've made it through the process so far, good work. The community is an important part of the success of any project, and Holochain is no different. If you're really proud of your work, get in touch with the development team on the [chat server](https://chat.holochain.net/appsup/channels/hc-core), mention you're working on it, and request help if necessary. This book could be updated to include links to other HDKs. Whether you would like to, or you'd like the team to, the HDK could be published to the primary package manager in use for the language, to be used by developers around the world. For example, RubyGems for Ruby or npm for nodejs.
