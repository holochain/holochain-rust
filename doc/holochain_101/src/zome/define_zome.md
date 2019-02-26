# Intro to Zome Definition

The [adding a zome](./adding_a_zome.md) section explored the file structure of a Zome. [Intro to HDK](./intro_to_hdk.md) covered Holochain Development Kit libraries for languages that Zomes can be written in. Once a Zome has been generated, and the HDK imported, it is time to start adding definitions to it.

There are multiple aspects to defining a Zome. They will each be covered in detail in the following articles.

What are the characteristics of a Zome, that need defining?

A Zome has
- `name`: the name of the containing folder
- `description`: defined in JSON in the `zome.json` file within a Zome folder
- `config`: Not Implemented Yet
- validating entry types: definition may vary based on the language
- a `genesis` function: a callback that Holochain expects and requires, defined in the code itself
- `fn_declarations`: a collection of custom functions declarations,
- `traits`: sets of named function groups used for composability
- `code`: the core application logic of a Zome, written in a language that compiles to WASM, which Holochain interprets through that compiled WASM

To develop a Zome, you will have to become familiar with these different aspects, the most complex of which are the validating entry types, and the traits and function definition. Implementation details will differ depending on the language that you are developing a Zome in.

## Building in Rust: define_zome!

As discussed in the [intro to HDK](./intro_to_hdk.md) article, by setting the HDK as a dependency in the `Cargo.toml` file, and then referencing it in `src/lib.rs`, Zome code in Rust gains access to a host of features.

The first line in the following code snippet (from a `src/lib.rs`) is important: `#[macro_use]`. This imports access to custom Rust macros defined by the HDK.

What are Rust macros? Generally speaking, they are code that will actually generate other code, when compiled. They are shortcuts. Anywhere in Rust that you see an expression followed immediately (no space) by an exclamation mark (!) that is the use of a macro.

In the case of Zome development, it was discovered that much code could be saved from being written, by encapsulating it in a macro.

That is how `define_zome!` came about. It is a Rust macro imported from the HDK which must be used for every Zome (unless you read the source code for it yourself and write something that behaves the same way!)

The following is technically the most minimalistic Zome that could be implemented. It does nothing, but still conforms to the expectations Holochain has for a Zome.

```rust
#[macro_use]
extern crate hdk;

define_zome! {
    entries: []

    genesis: || {
        Ok(())
    }

    functions: []

    capabilitites: {}
}
```

`entries` represents the validating entry type definitions. Note that it is an array, because there can be many. What validating entry types are will be [explained next](./entry_type_definitions.md).

`genesis` represents the previously mentioned `genesis` callback that Holochain expects from every Zome. [Skip here for details.](./genesis.md)

`functions` is where the functions are defined. [Skip here for details.](./zome_functions.md)

These are the three *required* properties of `define_zome!`.
