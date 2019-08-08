# Zome Functions

It is time to address the core application logic of Zomes.

What Zome functions you write depends, of course, on what you are building your application to do. By exposing a number of native capacities of Holochain to Zomes, developers have been given access to a rich suite of tools that offer limitless ways they can be combined. Holochain achieves this by exposing core functions to the WASM code of Zomes.

A core feature of an HDK is that it will handle that native interface to Holochain, wrapping the underlying functions in easy to call, well defined and well documented functions that are native to the language that you're writing in!

So within Zome code, by calling functions of the HDK, you can do powerful things with Holochain, like:
  - Read and write data to and from the configured storage mechanism, and from the network
  - Transport messages directly between nodes of a network
  - Call functions in other Zomes, and even "bridged" DNA instances
  - Use cryptographic functions like signing and verification to handle data security and integrity
  - Emit "signals" containing data from a Zome, to a UI for example

How these different functions work and how to use them will be covered throughout the rest of this chapter in detail. This article will provide a general overview of what Zome functions themselves are and how they work.

An important thing to note before going further is that zome functions *cannot* be used to enforce data integrity. One reason for this is that the core holochain functionality (e.g. `commit_entry`, `link_entry`) can be called externally bypassing the zome function logic. All data validation must be done inside the entry validation callbacks. You can think of zome functions just as helpers that encode common workflows and expose them to the consuming code (e.g. a UI).

Recall that Zomes will be written in diverse programming languages, any one that compiles to WebAssembly. Towards the bottom of this article, "Building in Rust" gives examples of what writing functions in Rust will be like.

## DNA, Zomes, Functions, Traits, and Capabilities

When Holochain loads a DNA file, to start an instance from it, it expects the presence of one or more Zomes in the definition. Here is a skeletal (incomplete) DNA JSON file to illustrate this:

```json
{
    "name": "test",
    "zomes": {
        "test_zome": {
            "name": "test_zome",
            "traits": {
                "hc_public": {
                    "functions": [],
                }
            },
            "fn_declarations": [],
            "code": {
                "code": "AAECAw=="
            }
        }
    }
}
```

This theoretical DNA has one Zome, "test_zome". However, it has no functions. Note that the nested `fn_declarations` property is an empty array.

There are a few things to learn from this DNA JSON. The first, is that the code, Base64 encoded WASM, is actually embedded in the Zome's definition, nested under `code.code`. All the functions Holochain expects to be implemented need to be encapsulated within that WASM code.

The second is that even outside of the WASM code, Holochain expects a certain level of visibility into the functions contained within, at least the ones meant to be called via Holochain (as oppose to private/internal functions).

There are at least two reasons for this:
- to be able to reason about data inputs and outputs for those functions
- to group those functions semantically for composition

These will both be discussed below.

## Function Declarations

All of the Zome's functions are declared in the `fn_declarations` array. Here is an example of one:

```json
"fn_declarations": [
    {
        "name": "get_task_list",
        "inputs": [{"name": "username", "type": "string"}],
        "outputs": [{"name": "task_list", "type": "json"}]
    }
]
```

Each function declaration is an object that includes the `name`, and the `inputs` and `outputs` expected for the function. Since WebAssembly only compiles from code languages with a type system, the generation of these inputs and outputs can expected to be automated.

The `name` is the most important thing here, because when a function call to an instance is being performed, it will have to match a name which Holochain can find in the `functions`. If the function isn't declared, Holochain will treat it as if it doesn't exist, even if it is an exposed function in the WASM code.

## Traits
Traits provide a way to group functions by name.  The primary use of this feature is for creating a composibility space where DNA creators can implement different DNAs to emergent function interfaces and then compose with them in the conductor by matching on the function group names and signatures.  Additionally Holochain may reserve a few special trait names that have specific side-effects.  The first of such reserved names is `hc_public`.  Functions grouped in this name will have automatically added to a public capability grant that happens at init time, thus making them accessible to any caller.  For more details on the Holochain security model please see the [Capabilities](capabilities.md) section.

Here is an example of what a trait definition using the public reserved trait name might look like:

```json
"traits": {
    "hc_public": {
        "functions": ["get_task_list"]
    }
}
```

## Data Interchange - Inputs and Outputs

In order to support building zomes in a variety of languages, we decided to use a simple language agnostic function specification format, using JSON.  Other formats may be supported in the future.

This has two big implications: Holochain Conductor implementations must handle JSON serialization and deserialization on the "outside", and HDKs and Zomes must handle JSON serialization and deserialization on the "inside". Holochain agrees only to mediate between the two by passing a string (which should represent valid JSON data).

## How Zome Functions Are Called

Function calls are received by Holochain from client requests (which there are a variety of implementations of, discussed later).  When function calls are being made, they will need to include a complete enough set of arguments to know the following:
- which Zome?
- which function?
- what values should the function be called with?

Before making the function call, Holochain will check the validity of the request, and fail if necessary. If the request is deemed valid, Holochain will mount the WASM code for a Zome using its' WASM interpreter, and then make a function call into it, giving it the arguments given to it in the request. When it receives the response from the WASM, it will then pass that return value as the response to the request. This may sound complex, but that's just what's going on internally, actually using it with an HDK and a [Conductor](../conductors.md) (which is discussed later) is easy.


## Building in Rust: Zome Functions

So far, in [entry type definitions](./entry_type_definitions.md) and [init](./init.md), the most complex example of a zome module was still very simple, and didn't include any functions.

### Adding a Zome Function

Here is an example of a simplistic function, for illustration purposes. It centers on the use of a function call to an HDK function. Zome functions must be declared within a zome module (`#[zome]`).

```rust
#[zome_fn("hc_public", "my_trait")]
fn send_message(to_agent: Address, message: String) -> ZomeApiResult<String>  {
    hdk::send(to_agent, message, 60000.into())
}
```

The function signature is consumed by the macro and used to automatically generate a definition. Notice also the `#[zome_fn()]` annotation contains a list of the traits this function should be added to.

Within the function, `send_message` makes use of a Holochain/HDK function that [sends messages directly node-to-node](https://developer.holochain.org/api/0.0.26-alpha1/hdk/api/fn.send.html).

The available functions, their purpose, and how to use them is fully documented elsewhere, in the [API reference](https://developer.holochain.org/api/0.0.26-alpha1/hdk/api/index.html#functions) and the [List of API Functions](./api_functions.md).

In the example, `handle_send_message` simply forwards the result of calling `hdk::send` as its' own result.

To see plenty of examples of adding functions, check out a file used for [testing the many capacities of the HDK](https://github.com/holochain/holochain-rust/blob/v0.0.4/hdk-rust/wasm-test/src/lib.rs).
