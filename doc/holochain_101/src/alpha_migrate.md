# Updating from holochain-proto to holochain-rust

If you wrote an application for [holochain-proto](https://github.com/holochain/holochain-proto), you are likely wondering what it may take to port your app to the new [holochain-rust](https://github.com/holochain/holochain-proto) version of Holochain.

The following should provide multiple levels of insight into what this could involve.

At a very general level:
- In terms of code, you have at least 2 options
    - rewriting the code in Rust
    - waiting for Assemblyscript support, and migrating at that point to Assemblyscript (the caveat to this approach is that it is not yet known at which point this support will arrive)
- The API between a user interface and Holochain has switched from HTTP to Websockets (for now), and so any user interface must be updated to use this approach.
- The DNA file has been simplified. Less is defined as JSON in the dna.json file and more is defined in the code.
- Testing of DNA utilizes Nodejs to run tests, using the testing library of your choice. This replaces the custom (and limited) JSON test configuration employed by holochain-proto.
- Schemas for entry types are no longer defined using json-schema, but using native Rust structs.

At the level of the code, in more detail, the changes are as follows (note that this is in reference to Javascript Zomes being ported to Rust Zomes):
- all camel case function names are now snake case
- `makeHash` is now named `entry_address`
- `commit` is now named `commit_entry`
- `get` is now named `get_entry`
- `update` is now named `update_entry`
- `remove` is now named `remove_entry`
- Links are no longer created using `commit`, but instead have their own method, named `link_entries`
- Instead of being implicitly imported, the Zome API functions are explicitly imported into Zomes, e.g.
`extern crate hdk;`
- The code of each Zome must now utilize a Rust "macro" called "define_zome!", and its various subproperties, which did not previously exist.
- Many aspects of validation have changed, see the section below on validation

### Updating Validation
There is a conceptual change to the approach to validation of entries, and even whereabouts that logic lives in the code.

In `holochain-proto`, there were a number of hooks which Holochain would call back into, to perform validation, such as
- validateCommit
- validatePut
- validateMod
- validateDel
- validateLink

Regardless of how many entry types there were, there would still be only 5 callbacks defined maximum. These validation callbacks were performed at a certain stage in the lifecycle of an entry.

Now, an entry type is defined all in one place, including its validation rules, which are unique to it as an entry type.
This could look as follows:
```rust
#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct Person {
    name: String,
}
```

```rust
entry!(
    name: "person",
    description: "",
    sharing: Sharing::Public,
    native_type: Person,
    validation_package: || {
        hdk::ValidationPackageDefinition::Entry
    },
    validation: |person: Person, validation_data: hdk::ValidationData| {
        (person.name.len() >= 2)
            .ok_or_else(|| String::from("Name must be at least 2 characters"))
    }
)
```

The callback `validation`, replaces `validateCommit` and all the rest from holochain-proto. However, validation still happens at various times in the lifecycle of an entry, so if the validation is to operate differently between initial `commit` to the chain, `update`, or `remove`, then that logic must be written into this single validation function. To determine which context validation is being called within, you can check in a property of the second parameter of the callback, which in the example above is called `validation_data`.

For this, you can use the Rust `match` operator, and check against the `validation_data.action`. It will be one of an enum that can be seen in detail [in the API reference](/api/0.0.6-alpha/hdk/enum.EntryAction.html).



### Yet to cover:
- Capabilities
- Traits
- UI
