# Zome Functions

Finally, it is time to address the core application logic of Zomes.

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

## DNA, Zomes, Capabilities, and Functions

When Holochain loads a DNA file, to start an instance from it, it expects the presence of one or more Zomes in the definition. Here is a skeletal (incomplete) DNA JSON file that can illustrate this:

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

There are few things to learn from this DNA JSON. The first, that is simple to explain, is that the code, Base64 encoded WASM, is actually embedded in the Zome's definition, nested under `code.code`. All the functions Holochain expects to be implemented need to be encapsulated within that WASM code.

The second is that even outside of the WASM code, Holochain expects a certain level of visibility into the functions contained within, at least the ones meant to be called via Holochain (as oppose to private/internal functions).

There are at least two reasons for this:
- to define a permission based system for those functions
- to be able to reason about data inputs and outputs for those functions

These will both be discussed below.

## Capabilities
## TODO: FIXME (convert to traits)

In order to operate securely, but still be full featured, Holochain has a permissions system for function calls. This is being called "Capabilities".

A Zome can have multiple Capabilities, and each Capability has one CapabilityType, from a defined set of options, as well as list of functions that are accessible using that capability. The point of selecting a CapabilityType for a set of functions is that it will allow granular control of who can call which functions of a Zome.

In the example, the name of the trait was "hc_public", which is a reserved trait name.

```json
"traits": {
    "hc_public": {
        "functions": ["get_task_list"]
    }
}
```

The CapabilityType, or just "type" in the JSON, for the Capability is set to "public". The current options for a CapabilityType are `public`, `transferable` and `assigned`.

At this moment, Holochain's capability system is still under development, so these values aren't final. More documentation for Capabilities will be released as the implementation evolves within Holochain.

Important notes for the current use of Capabilities:
- Holochain does not yet check the identity of the user making function calls
- Capability names are ALSO needed when function calls are being made

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

In order to maintain compabitility with a variety of languages, it was decided to use a language agnostic data interchange format for inputs and ouputs. JSON, the modern web format was selected. Other formats may be supported in the future.

Put simply, this has two big implications: Holochain Container implementations must handle JSON serialization and deserialization on the "outside", and HDKs and Zomes must handle JSON serialization and deserialization on the "inside". Holochain agrees only to mediate between the two by passing a string (which should represent valid JSON data).

## Introducing "Containers"

To discuss the functions developers will build within Zomes, it is useful to zoom out for a moment, to the level of how Holochain runs on devices. Because there was an intention to make Holochain highly platform and system compatible, the core logic was written in such a way that it could be included into many different codebases. Think MacOSX, Linux, Windows, Android, iOS, and more. Thus Holochain core is actually simply a library that needs to be included in another project which mounts, executes and manages it. Because filling this new need is becoming such a foundational aspect of Holochain, it has its' own name: *Container*.

Containers install and uninstall, start and stop instances of DNA on devices. There is one more important function of Containers: *they create a channel to securely make function calls into the Zome functions of DNA instances*.

Imagine that there are many DNA instances running within one Container, and each DNA can have multiple Zomes. Clearly, function calls will need to include a complete enough set of arguments to know the following:
- which instance?
- which Zome?
- which Capability token?
- which function?
- what arguments?

Containers can implement whatever interfaces to perform these function calls they wish to, opening a wealth of opportunity. Holochain provides two reference Containers, one for [Nodejs](https://www.npmjs.com/package/@holochain/holochain-nodejs), and the other a [Rust built binary executable](https://github.com/holochain/holochain-rust/tree/develop/container). With the Rust built binary Container, interfaces for making function calls already includes HTTP and WebSockets. More details about Containers can be found in [another chapter](../containers.md), it is simply important context for proceeding.

When a call to a Zome function is being made from the Container, it first passes the arguments to Holochain. Before making the function call, Holochain will check the validity of the request, and fail if necessary. If the request is deemed valid, Holochain will mount the WASM code for a Zome using its' WASM interpreter, and then make a function call into it, giving it the arguments given to it in the request. When it receives the response from the WASM, it will then pass that return value as the response to the request. This may sound complex, but that's just what's going on internally, actually using it with an HDK and a Container is easy.


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

`functions` is where the Capabilities, and function declarations will be made.

### Adding a Capability

A Zome can have multiple Capabilities within it. This is what adding some Capabilities might look like:

```rust
...

define_zome! {
    ...
    capabilities: {
        public (Public) [read_post]
        authoring (Assigned) [create_post, update_post]
        }
    }
}
```

In this example, `public` is the name of a capability which grants `Public` Capbility-type access to the `read_post` function, and `authoring` is the name of a capability which for which token grants can be assigned to specific agents for access to the `create_post` and `update_post` functions.  The implication of `Public` is that from your local device, any request to Holochain to make a function call to this Capability of this Zome will succeed, without needing authorization.

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
