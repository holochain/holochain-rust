# App Entry Type Definitions

An "entry" is a data element that an agent authors to their source-chain (stored on their local device), which is then propagated to peers. The entry is backed by a "chain header", which is the data element used for verification of the integrity of itself, as well as the entry.

Entries are a fundamental, primitive type within Holochain. Entries are an abstraction, they can technically be persisted to a device in a variety of ways, using a variety of databases, which can be as simple as files in the file system.

There are types of entries which cannot be written to the chain by users of an application. These are generally called system entries. They include DNA, and initial Agent entries, which are always the first two entries written to a chain.

There are a special type of entries called App Entries. These are entries which are created through the active use of an application by a user. They must have an entry type which, rather than being system defined, is defined by the Zome developer.

## Defining App Entry Types

Creating a Zome for a hApp will almost always involve defining app entry types for that Zome. This means looking closely at the data model.

What types of data will the Zome be designed to handle? Is it dealing in "users", "transactions", "friendships", "tasks", or what? These will be the entry types needing definition in a Zome.

Broadly speaking, when defining the entry type, the developer of a Zome is designing the behaviour and generic properties of data of that type. This includes these important aspects:
- how that data is shared, or not, with peers
- the schema for entries of the type
- custom validation behaviour for entries of the type
- types of relationships (links) that can exist between entry types

An entry type is given a name that is used when an agent is attempting to write an entry to the chain. That's how Holochain knows what to do with the data for the entry that it has been given.

An entry type should also be given a basic description so that other people reading it understand the use of the entry type.

A third important property is `sharing`. The primary options for this at this time are 'Private' and 'Public'. Private means entries of this type will stay only the device of the author. Public means entries of this type will be gossiped to other peers sharing copies of the DNA. Public does NOT mean that it will be shared publicly on the internet.

Examining a `.dna.json` file closely, nested within the JSON configuration for a Zome, for an entry type you might see something like the following:

```json
"entry_types": [
    {
        "entry_type_name": "post",
        "description": "A blog post entry which has an author",
        "sharing": "public",
        "links_to": []
    }
]
```

This is a Zome that implements only a single entry type, `post`.

These values are likely not to be modified within a JSON file, but within some code itself, where the entry type is defined. The validation rules for the entry type, will of course be defined within the code as well. Since this can be a complex topic, defining the validation logic has [its' own article](./entry_validation.md).

Setting up the entry types for a Zome is an often logical starting point when creating a Zome.

## Building in Rust: Defining an Entry Type

Recall that in [define_zome!](./define_zome.md#building-in-rust-define_zome), there was an array called `entries`. The most minimalistic Zome could look like this:
```rust
#[macro_use]
extern crate hdk;

define_zome! {
    entries: []

    genesis: || {
        Ok(())
    }

    functions: []

    traits: {}
}
```

`entries` is where we will populate the Zome with entry type definitions. It expects an array of `ValidatingEntryType`. So how can one be created?

Easy: the `entry!` macro. It can be used to encapsulate everything needed to define an entry type. All of the following must be defined:

---

__name__
```rust
entry!(
    name: "post",
    ...
)
```

This should be a machine-readable name for the entry type. Spaces should not be used. What will the entry type be that will be given when new entries are being created?

---

__description__
```rust
entry!(
    ...
    description: "A blog post entry which has an author",
    ...
)
```

This should be a human-readable explanation of the meaning or role of this entry type.

---

__sharing__
```rust
use hdk::holochain_core_types::dna::entry_types::Sharing;

entry!(
    ...
    sharing: Sharing::Public,
    ...
)
```

As mentioned above, sharing refers to whether entries of this type are private to their author, or whether they will be gossiped to other peers to hold copies of. The value must be referenced from an [enum in the HDK](/api/0.0.7-alpha/holochain_core_types/dna/entry_types/enum.Sharing.html). Holochain currently supports the first two values in the enum: Public, and Private.

---

__native_type__
```rust
#![feature(try_from)]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_core_types_derive;

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct Post {
    content: String,
    date_created: String,
}

entry!(
    ...
    native_type: Post,
    ...
)
```

Clearly, `native_type` is where things start to get interesting. It requires the introduction of quite a number of dependencies, first of all. Why is that?

It is important to remember that the Rust code of a Zome is compiled into WASM before it can be executed by Holochain. This introduces a certain constraint. How is data passed between Holochain, and the WASM Zome code? Answer: it is stored in the WASM memory as stringified JSON data, and accessed by the WASM code and by Holochain, running the WASM interpreter.

JSON was chosen as the interchange format because it is so universal, and almost all languages have serializers and parsers. Rust's is called `serde`. The three `serde` related dependencies all relate to the need to serialize to and from JSON within Zomes.

Note that the top line in the snippet above is important. It switches on a Rust feature that would otherwise be off, allowing attempted conversions between types, which is exactly what the JSON parsing is doing.
```rust
#![feature(try_from)]
```

Additionally, the HDK offers built-in conversion functions from JSON strings to Entry structs. This comes from the `DefaultJson` [derive](https://doc.rust-lang.org/rust-by-example/trait/derive.html).

Every struct used as a `native_type` reference should include all 4 derives, as in the example:
```rust
#[derive(Serialize, Deserialize, Debug, DefaultJson)]
```

`Serialize` and `Deserialize` come from `serde_derive`, and `DefaultJson` comes from `holochain_core_types_derive`.

Then there is the struct itself. This is the real type definition, because it defines the schema. It is simply a list of property names, the 'keys', and the types of values expected, which should be set to one of the primitive types of the language. This will tell `serde` how to parse JSON string inputs into the type. Note that conversion from JSON strings into the struct type can easily fail, in particular if the proper keys are not present on the input.

---

__validation_package__

```rust
use hdk::ValidationPackageDefinition;

entry!(
    ...
    validation_package: || {
        ValidationPackageDefinition::Entry
    },
    ...
)
```

At the moment, what `validation_package` is will not be covered in great detail. In short, for a peer to perform validation of an entry from another peer, varying degrees of metadata from the original author of the entry might be needed. `validation_package` refers to the carrier for that extra metadata.

Looking at the above code, there is a required import from the HDK needed for use in `validation_package`, and that's the enum `ValidationPackageDefinition`. The value of `validation_package` is a function that takes no arguments. It will be called as a callback by Holochain. The result should be a value from the `ValidationPackageDefinition` enum, whose values can be [seen here](https://developer.holochain.org/api/0.0.7-alpha/hdk/enum.ValidationPackageDefinition.html). In the example, and as the most basic option, simply use `Entry`, which means no extra metadata beyond the entry itself is needed.

Further reading is [here](./entry_validation.md).

---

__validation__
```rust
use hdk::ValidationData;

entry!(
    ...
    validation: |_post: Post, _validation_data: ValidationData| {
        Ok(())
    }
)
```

`validation` is the last required property of `entry!`. Because it is such an important aspect, it has [its' own in depth article](./entry_validation.md).

It is a callback that Holochain will call during different moments in the lifecycle of an entry, in order to confirm which action to take with the entry, depending on its' validity. It will be called with two arguments, the first representing the struct of the entry itself, and the second a struct holding extra metadata that can be used for validation, including, if it was requested, the `validation_package`.

The callback should return a Rust `Result` type. This is seen in `Ok(())`. The example above is the simplest possible `validation` function, since it doesn't perform any real logic. While this is ok in theory, great caution should be taken with the validation rules, and further reading is recommended.

`validation` for a `ValidatingEntryType` should either return `Ok(())` or an `Err` containing the string explaining why validation failed.

The validity of an entry is therefore defined by the author of a Zome. First of all, data which doesn't conform to the schema defined by the `native_type` will fail, but `validation` allows for further rules to be defined.

Note that not only the entry author will call this function to validate the entry during its' creation, but other peers will call this function to validate the entry when it is requested via the network that they hold a copy of it. *This is at the heart of how Holochain functions as peer-to-peer data integrity layer.*

Further reading can be found [here](./entry_validation.md).

---

### Putting It All Together

Taken all together, use of the `entry!` macro may look something like the following:

```rust
...
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
```

This can be embedded directly inside of the entries array of the `define_zome!`, like so:
```rust
...
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

    capabilitites: {}
}
```

If there is only entry type, this can be fine, but if there are multiple, this can hurt readability of the source code. You can wrap the entry type definition in a function, and call it in `define_zome!`, like so:
```rust
...
fn post_definition() -> ValidatingEntryType {
    entry!(
        name: "post",
        description: "A blog post entry which has an author",
        sharing: Sharing::Public,
        native_type: Post,

        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },

        validation: |_post: Post, _validation_data: hdk::ValidationData| {
            Ok(())
        }
    )
}

define_zome! {
    entries: [
        post_definition()
    ]

    genesis: || {
        Ok(())
    }

    functions: []

    capabilitites: {}
}
```

Use of this technique can help you write clean, modular code.

If you want to look closely at a complete example of the use of `entry!` in a Zome, check out the [API reference](https://developer.holochain.org/api/0.0.7-alpha/hdk/macro.entry.html), or the ["app-spec" example app](https://github.com/holochain/holochain-rust/blob/v0.0.4/app_spec/zomes/blog/code/src/post.rs).

#### Summary
This is still a pretty minimal Zome, since it doesn't have any functions yet, and the most basic `genesis` behaviour, so read on to learn about how to work with those aspects of `define_zome!`.
