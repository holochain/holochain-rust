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
        "properties": "{\"description\": \"A blog post entry which has an author\"}",
        "sharing": "public",
        "links_to": []
    }
]
```

This is a Zome that implements only a single entry type, `post`.

These values are likely not to be modified within a JSON file, but within some code itself, where the entry type is defined. The validation rules for the entry type, will of course be defined within the code as well. Since this can be a complex topic, defining the validation logic has [its' own article](./entry_validation.md).

Setting up the entry types for a Zome is an often logical starting point when creating a Zome.

## Building in Rust: Defining an Entry Type

Recall that in [the zome definition](./define_zome.md#building-in-rust-define_zome) it was mentioned that entries could be defined by annotating special function. The signature for an entry definition function inside the zome module is as follows:

```rust
#[entry_def]
fn my_entry_def() -> ValidatingEntryType
```

How do we create a ValidatingEntryType? Easy: the `entry!` macro. It can be used to encapsulate everything needed to define an entry type. All of the following must be defined:

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

Historically this was a human-readable explanation of the meaning or role of this entry type. Now the description field can hold a stringified JSON object to hold various properties of this entry type (Possibly including a description but also UI display hints, indexing fields, example data etc. It is totally up to you). These properties can be accessed via `hdk::entry_type_properties`.

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

As mentioned above, sharing refers to whether entries of this type are private to their author, or whether they will be gossiped to other peers to hold copies of. The value must be referenced from an [enum in the HDK](/api/0.0.26-alpha1/holochain_core_types/dna/entry_types/enum.Sharing.html). Holochain currently supports the first two values in the enum: Public, and Private.

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

Looking at the above code, there is a required import from the HDK needed for use in `validation_package`, and that's the enum `ValidationPackageDefinition`. The value of `validation_package` is a function that takes no arguments. It will be called as a callback by Holochain. The result should be a value from the `ValidationPackageDefinition` enum, whose values can be [seen here](https://developer.holochain.org/api/0.0.26-alpha1/hdk/enum.ValidationPackageDefinition.html). In the example, and as the most basic option, simply use `Entry`, which means no extra metadata beyond the entry itself is needed.

Further reading is [here](./entry_validation.md).

---

__validation__
```rust
use hdk::EntryValidationData;

entry!(
    ...
    validation: |_validation_data: ValidationData<Post>| {
        Ok(())
    }
)
```

`validation` is the last required property of `entry!`. Because it is such an important aspect, it has [its' own in depth article](./entry_validation.md).

It is a callback that Holochain will call during different moments in the lifecycle of an entry, in order to confirm which action to take with the entry, depending on its' validity. It will be called with two arguments, the first representing the struct of the entry itself, and the second a struct holding extra metadata that can be used for validation, including, if it was requested, the `validation_package`.

The callback should return a Rust `Result` type. This is seen in `Ok(())`. The example above is the simplest possible `validation` function, since it doesn't perform any real logic. While this is ok in theory, great caution should be taken with the validation rules, and further reading is recommended.

`validation` for a `ValidatingEntryType` should either return `Ok(())` or an `Err` containing the string explaining why validation failed.

Also note the use of a Rust type as the type parameter to the validation data (`ValidationData<Post>`). In this case we are referencing a struct called `Post`. This automatically adds an extra layer of validation that the entry data must be serializable/deserializable between this type and JSON. Types used here must implement the `DefaultJson` trait.

The validity of an entry is therefore defined by the author of a Zome. First of all, data which doesn't conform to the schema defined by the type will fail, but `validation` allows for further rules to be defined.

Note that not only the entry author will call this function to validate the entry during its' creation, but other peers will call this function to validate the entry when it is requested via the network that they hold a copy of it. *This is at the heart of how Holochain functions as peer-to-peer data integrity layer.*

Further reading can be found [here](./entry_validation.md).

---

### Putting It All Together

Taken all together, defining an entry inside the `#[zome]` module using the `entry!` macro may look something like the following:

```rust
...
#[entry_def]
fn post_entry_def() -> ValidatingEntryType {
    entry!(
        name: "post",
        description: "A blog post entry which has an author",
        sharing: Sharing::Public,
        validation_package: || {
            ValidationPackageDefinition::Entry
        },
        validation: |validation_data: EntryValidationData<Post>| {
            Ok(())
        }
    )
}
```

    If you want to look closely at a complete example of the use of `entry!` in a Zome, check out the [API reference](https://developer.holochain.org/api/0.0.26-alpha1/hdk/macro.entry.html), or the ["app-spec" example app](https://github.com/holochain/holochain-rust/blob/release-0.0.26-alpha1/app_spec/zomes/blog/code/src/post.rs).

#### Summary
This is still a pretty minimal Zome, since it doesn't have any functions yet, and the most basic `init` behaviour, so read on to learn about how to work with those aspects of defining a zome.
