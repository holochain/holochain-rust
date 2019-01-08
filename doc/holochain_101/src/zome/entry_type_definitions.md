# App Entry Type Definitions

Recall that (todo place this info somewhere) an entry is a data element that an agent authors to their local device, that is then propogated to peers running the same app. The entry is backed by a 'chain header', which is the data element used for verification of the integrity of itself, as well as the entry.

Entries are a fundamental, primitive type within Holochain. Entries are an abstraction, they can technically be persisted to a device in a variety of ways, using a variety of databases, which can be as simple as files in the file system.

There are types of entries which cannot be written to the chain by users of an application. These are generally called system entries. They include DNA, and initial Agent entries, which are always the first two entries written to a chain.

There are a special type of entries called App Entries. These are entries which are created through the active use of an application by a user. They must have an entry type which, rather than being system defined, is defined application specific.

### Defining App Entry Types

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

These values are likely not to be modified within a JSON file, but within some code itself, where the entry type is defined. The validation rules for the entry type, will of course be defined within the code as well. Since this can be a complex topic, defining the validation logic has [its' own article(TODO add link)]().

Setting up the entry types for a Zome is an often logical starting point when creating a Zome.

### Building in Rust: Defining an Entry Type

Recall that in [define_zome!](./define_zome.md#building-in-rust-define_zome), there was an array called `entries`. The most minimalistic Zome could look like this:
```rust
#[macro_use]
extern crate hdk;

define_zome! {
    entries: []

    genesis: || {
        Ok(())
    }

    functions: {}
}
```

`entries` is where we will populate the Zome with entry type definitions. It expects an array of `ValidatingEntryType`. So how can one be created?

Easy: the `entry!` macro. It can be used to encapsulate everything needed to define an entry type. All of the following must be defined:

__name__
```rust
entry!(
    name: "post",
    ...
)
```

This should be a machine-readable name for the entry type. Spaces should not be used. What will the entry type be that will be given when new entries are being created?

__description__
```rust
entry!(
    ...
    description: "A blog post entry which has an author",
    ...
)
```

This should be a human-readable explanation of the meaning or role of this entry type.

__sharing__
```rust
use hdk::holochain_core_types::dna::entry_types::Sharing;
...
entry!(
    ...
    sharing: Sharing::Public,
    ...
)
```

As mentioned above, sharing refers to whether entries of this type are private to their author, or whether they will be gossiped to other peers to hold copies of. The value must be referenced from an [enum in the HDK](/api/latest/holochain_core_types/dna/entry_types/enum.Sharing.html). Holochain currently supports the first two values in the enum: Public, and Private.

__native_type__
```rust
#![feature(try_from)]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate holochain_core_types_derive;

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct Post {
    content: String,
    date_created: String,
}

entry!(
    ...
    native_type: Post,
    ...
)
```

Clearly, `native_type` is where things start to get interesting.

__validation_package__

__validation__

```rust
#![feature(try_from)]

#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate holochain_core_types_derive;

use hdk::{
    error::ZomeApiResult,
    entry_definition::ValidatingEntryType
    holochain_core_types::{
        cas::content::Address,
        entry::Entry,
        json::JsonString,
        dna::entry_types::Sharing,
        error::HolochainError,
    },
    holochain_wasm_utils::api_serialization::get_links::GetLinksResult,
};

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct Post {
    content: String,
    date_created: String,
}

fn post_definition() -> ValidatingEntryType {
    entry!(
        name: "post",
        description: "A blog post entry which has an author",
        sharing: Sharing::Public,
        native_type: Post,

        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },

        validation: |_post: Post, _ctx: hdk::ValidationData| {
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

    functions: {}
}
```


