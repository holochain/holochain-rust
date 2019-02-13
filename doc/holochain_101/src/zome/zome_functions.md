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

Recall that Zomes will be written in diverse programming languages, any one that compiles to WebAssembly. Towards the bottom of this article, "Building in Rust" gives examples of what writing functions in Rust will be like. It is difficult to show what a function in WASM looks like, since even the "human-readable" version of WASM, WAT, is not highly readable.

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

## Data Interchange - Inputs and Outputs

In order to support building zomes in a variety of languages, we decided to use a simple language agnostic function specification format, using JSON.  Other formats may be supported in the future.

This has two big implications: Holochain Conductor implementations must handle JSON serialization and deserialization on the "outside", and HDKs and Zomes must handle JSON serialization and deserialization on the "inside". Holochain agrees only to mediate between the two by passing a string (which should represent valid JSON data).

## Traits
Traits provide a way to group functions by name.  The primary use of this feature is for creating a composibility space where DNA creators can implement different DNAs to emergent function interfaces and then compose with them in the conductor by matching on the function group names and signatures.  Additionally Holochain may reserve a few special trait names that have specific side-effects.  The first of such reserved names is `hc_public`.  Functions grouped in this name will have automatically added to a public capability grant that happens at genesis time, thus making them accessible to any caller.  For more details on the Holochain security model please see the [Capbilities](capabilities.md) section.

Here is an example of what a trait definition using the public reserved trait name might look like:

```json
"traits": {
    "hc_public": {
        "functions": ["get_task_list"]
    }
}
```

## How Zome Functions Are Called

Function calls are received by Holochain from client requests (which there are a variety of implementations of, discussed later).  When function calls are being made, they will need to include a complete enough set of arguments to know the following:
- which Zome?
- which Capability token?
- which function?
- what values should the function be called with?

Before making the function call, Holochain will check the validity of the request, and fail if necessary. If the request is deemed valid, Holochain will mount the WASM code for a Zome using its' WASM interpreter, and then make a function call into it, giving it the arguments given to it in the request. When it receives the response from the WASM, it will then pass that return value as the response to the request. This may sound complex, but that's just what's going on internally, actually using it with an HDK and a [Conductor](../conductors.md) (which is discussed later) is easy.


## Building in Rust: Zome Functions

So far, in [entry type definitions](./entry_type_definitions.md) and [genesis](./genesis.md), the most complex example of `define_zome!` was still very simple, and didn't include any functions:

```rust
...

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct Post {
    content: String,
    date_created: String,
}

define_zome! {
    entries: [
        entry!(
            name: "post",
            description: "A blog post entry which has an author",
            sharing: Sharing::Public,
            native_type: Post,
            validation_package: || {
                ValidationPackageDefinition::Entry
            },
            validation: |_post: Post, _validation_data: ValidationData| {
                Ok(())
            }
        )
    ]

    genesis: || {
        Ok(())
    }

    functions: []
}
```

`functions` is where the function declarations will be made.

### Adding Traits:

Here are some sample traits

```rust
...

define_zome! {
    ...
    traits: {
        hc_public [read_post]
        authoring [create_post, update_post]
        }
    }
}
```

In this example, `hc_public` is the reserved trait name which create a `Public` Capbility-type grant at genesis time for access to the `read_post` function.  Additionally it names an `authoring` trait the `create_post` and `update_post` functions.

### Adding a Zome Function

In order to add a Zome function, there are two primary steps that are involved.
1. declare your function in `define_zome!`
2. write the Rust code for the handler of that function, calling any HDK functions you need

__Step 1__

The `functions` section looks a bit like an array of key-value pairs:

```rust
...

define_zome! {
    ...
    functions: [
        send_message: {
            inputs: |to_agent: Address, message: String|,
            outputs: |response: ZomeApiResult<String>|,
            handler: handle_send_message
        }
    ]
}
```

In this example, `send_message` is the given name of this function, by which it will be referenced and called elsewhere. There are three properties necessary to provide `send_message`, and any function declaration: `inputs`, `outputs`, and `handler`.

`inputs` expects a list or argument names, and types, for the `send_message` function to be called with.

`outputs` expects a single declaration of a return type. The name (which in the example is `response`) is arbitrary, call it anything.

`handler` expects the name of a function which will handle this function call, and which matches the function signature of `inputs` and `outputs`. In this case, `handle_send_message`, which has yet to be defined.

__Step 2__

Here is an example of a simplistic function, for illustration purposes. It centers on the use of a function call to an HDK function.

```rust
fn handle_send_message(to_agent: Address, message: String) -> ZomeApiResult<String>  {
    hdk::send(to_agent, message, 60000.into())
}
```

Notice right away how the arguments match perfectly with the `inputs: |...|` section of the function declaration. Any differences will cause issues. This is also true of the return type of the output. Note the pairing of `ZomeApiResult<String>` as the return type.

The name of the function, `handle_send_message` is the same as the name given as the `handler` in the `define_zome!` function declaration.

Within the function, `handle_send_message` makes use of a Holochain/HDK function that [sends messages directly node-to-node](https://developer.holochain.org/api/0.0.3/hdk/api/fn.send.html).

The available functions, their purpose, and how to use them is fully documented elsewhere, in the [API reference](https://developer.holochain.org/api/0.0.3/hdk/api/index.html#functions) and the [List of API Functions](./api_functions.md).

In the example, `handle_send_message` simply forwards the result of calling `hdk::send` as its' own result.

Here are the above two steps combined:
```rust
...

fn handle_send_message(to_agent: Address, message: String) -> ZomeApiResult<String>  {
    hdk::send(to_agent, message, 60000.into())
}

define_zome! {
    ...
    functions: [
        send_message: {
            inputs: |to_agent: Address, message: String|,
            outputs: |response: ZomeApiResult<String>|,
            handler: handle_send_message
        }
    ]
}
```

To see plenty of examples of adding functions, check out a file used for [testing the many capacities of the HDK](https://github.com/holochain/holochain-rust/blob/v0.0.3/hdk-rust/wasm-test/src/lib.rs).

Otherwise, continue reading to learn all about the API Functions and examples of how to use them.
