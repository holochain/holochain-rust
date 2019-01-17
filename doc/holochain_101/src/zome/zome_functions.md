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
            "capabilities": {
                "test_capability": {
                    "type": "public",
                    "fn_declarations": []
                }
            },
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

In order to operate securely, but still be full featured, Holochain has a permissions system for function calls. This is being called "Capabilities".

A Zome can have multiple Capabilities, and each Capability has one CapabilityType, from a defined set of options. The point of selecting a CapabilityType for a set of functions is that it will allow granular control of who can call which functions of a Zome.

In the example, the name of the capability was "test_capability".

```json
"capabilities": {
    "test_capability": {
        "type": "public",
        "fn_declarations": []
    }
}
```

The CapabilityType, or just "type" in the JSON, for the Capability is set to "public". The current options for a CapabilityType are `public`, `transferable` and `assigned`.

At this moment, Holochain's capability system is still under development, so these values aren't final. More documentation for Capabilities will be released as the implementation evolves within Holochain.

Important notes for the current use of Capabilities:
- Holochain does not yet check the identity of the user making function calls
- Capability names are ALSO needed when function calls are being made

## Function Declarations

It's time to illustrate what a `fn_declaration` looks like. Here is an example of one, added to "test_capability":

```json
"capabilities": {
    "test_capability": {
        "type": "public",
        "fn_declarations": [
            {
                "name": "get_task_list",
                "inputs": [{"name": "username", "type": "string"}],
                "outputs": [{"name": "task_list", "type": "json"}]
            }
        ]
    }
}
```

Each function declaration is an object that includes the `name`, and the `inputs` and `outputs` expected for the function. Since WebAssembly only compiles from code languages with a type system, the generation of these inputs and outputs can expected to be automated.

The `name` is the most important thing here, because when a function call to an instance is being performed, it will have to match a name which Holochain can find in the `fn_declarations` specification for the Capability. If the function isn't declared, Holochain will treat it as if it doesn't exist, even if it is an exposed function in the WASM code.

## Data Interchange - Inputs and Outputs

In order to maintain compabitility with a variety of languages, it was decided to use a language agnostic data interchange format for inputs and ouputs. JSON, the modern web format was selected. Other formats may be supported in the future.

Put simply, this has two big implications: Holochain Container implementations must handle JSON serialization and deserialization on the "outside", and HDKs and Zomes must handle JSON serialization and deserialization on the "inside". Holochain agrees only to mediate between the two by passing a string (which should represent valid JSON data).

## Introducing "Containers"

To discuss the functions developers will build within Zomes, it is useful to zoom out for a moment, to the level of how Holochain runs on devices. Because there was an intention to make Holochain highly platform and system compatible, the core logic was written in such a way that it could be included into many different codebases. Think MacOSX, Linux, Windows, Android, iOS, and more. Thus Holochain core is actually simply a library that needs to be included in another project which mounts, executes and manages it. Because filling this new need is becoming such a foundational aspect of Holochain, it has its' own name: *Container*.

Containers install and uninstall, start and stop instances of DNA on devices. There is one more important function of Containers: *they create a channel to securely make function calls into the Zome functions of DNA instances*.

Imagine that there are many DNA instances running within one Container, and each DNA can have multiple Zomes. Clearly, function calls will need to include a complete enough set of arguments to know the following:
- which instance?
- which Zome?
- which Capability?
- which function?
- what arguments?

Containers can implement whatever interfaces to perform these function calls they wish to, opening a wealth of opportunity. Holochain provides two reference Containers, one for [Nodejs](https://www.npmjs.com/package/@holochain/holochain-nodejs), and the other a [Rust built binary executable](https://github.com/holochain/holochain-rust/tree/develop/container). With the Rust built binary Container, interfaces for making function calls already includes HTTP and WebSockets. More details about Containers can be found in [another chapter](../container.md), it is simply important context for proceeding.

When a call to a Zome function is being made from the Container, it first passes the arguments to Holochain. Before making the function call, Holochain will check the validity of the request, and fail if necessary. If the request is deemed valid, Holochain will mount the WASM code for a Zome using its' WASM interpreter, and then make a function call into it, giving it the arguments given to it in the request. When it receives the response from the WASM, it will then pass that return value as the response to the request. This may sound complex, but that's just what's going on internally, actually using it with an HDK and a Container is easy.


## Building in Rust: Zome Functions

```rust
use hdk...

fn handle_send_message(to_agent: Address, message: String) -> ZomeApiResult<String>  {
    hdk::send(to_agent, message)
}

test (Public) {
        send_message: {
            inputs: |to_agent: Address, message: String|,
            outputs: |response: ZomeApiResult<String>|,
            handler: handle_send_message
        }
    }
}


```

[https://developer.holochain.org/api/0.0.3/holochain_core_types/dna/capabilities/enum.CapabilityType.html#variants](https://developer.holochain.org/api/0.0.3/holochain_core_types/dna/capabilities/enum.CapabilityType.html#variants)
```
Public
Transferable
Assigned
```
