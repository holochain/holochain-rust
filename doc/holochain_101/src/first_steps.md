# First steps writing Holochain hApps with Rust

___
This tutorial builds for the 0.0.25-alpha1 release. Since the HDK is still in alpha it may break under future releases.
___

Holochain hApps are made of compiled WebAssembly that encodes the rules of the hApp, the data it can store and how users will interact with it. This means that [any language that can compile to WebAssembly](https://github.com/appcypher/awesome-wasm-langs) can one day be used for Holochain.

Writing WebAssembly that complies with the Holochain runtime can be tricky. To make development as streamlined as possible the core team has been developing a Holochain-dev-kit (HDK) for the first supported language, Rust! In the near future the community is encouraged to develop an HDK for their language of choice.

In this article we will walk through the steps of creating a simple hApp using Rust.

## Requirements

First step is to follow the quickstart instructions installations on [this page](https://developer.holochain.org/start.html) to install the required dependencies. The nix-shell tool makes it incredibly easy to ensure consistent dev environments between machines. Once you have holonix installed make sure you are inside the nix-shell before running any of the commands in this guide.

If you want to jump ahead to see what the completed project will look like, the [full source code is available on GitHub](https://github.com/willemolding/holochain-rust-todo).

## First steps

We will be making a classic to-do list hApp. A user can create new lists and add items to a list. They should also be able to retrieve a list by its address and all of the items on each list.

Let's begin by generating an empty hApp skeleton by running:

```
hc init holochain-rust-todo
```

This will generate the following directory structure:

```
holochain-rust-todo/
├── app.json
├── test
│ └── …
└── zomes
```

Notice the `zomes` directory. All Holochain hApps are comprised of one or more zomes. They can be thought of as similar to modules in JavaScript, each one should provide some self-contained functionality. Every zome has its own build system so it is possible to combine zomes written in different languages to produce a single hApp.

We will create a single zome called `lists` that uses a Rust build system:

```
cd holochain-rust-todo
hc generate zomes/lists rust-proc
```

The project structure should now be as follows:

```
├── app.json
├── test
│ └── …
└── zomes
 └── lists
 ├── code
 │ ├── .hcbuild
 │ ├── Cargo.toml
 │ └── src
 │  └── lib.rs
 └── zome.json
```

## Writing the lists zome
The Rust HDK makes use of Rust macros to reduce the need for boilerplate code. The holochain HDK uses Rust annotations to label parts of the code for special purposes. The most important of which is `#[zome]`. Every zome must use this annotation on a module in the `lib.rs` to define the structure of the zome, what entries it contains, which functions it exposes and what to do on first start-up (init).

Open up `lib.rs` and replace its contents with the following:

```rust
#![feature(proc_macro_hygiene)]
#[macro_use]
extern crate hdk_proc_macros;

#[zome]
mod todo {
    #[init]
    fn init() -> ZomeApiResult<()> {
        Ok(())
    }

    #[validate_agent]
    fn validate_agent(validation_data : EntryValidationData::<AgentId>) -> ZomeApiResult {
        Ok(())
    }
}

```

This is the simplest possible valid zome with no entries and no exposed functions.

## Adding some Entries
Unlike in holochain-proto, where you needed to define a JSON schema to validate entries, holochain entries in Rust map to a native struct type. We can define our list and listItem structs as follows:

```rust
#[derive(Serialize, Deserialize, Debug, Clone, DefaultJson)]
struct List {
	name: String
}

#[derive(Serialize, Deserialize, Debug, Clone, DefaultJson)]
struct ListItem {
	text: String,
	completed: bool
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct GetListResponse {
    name: String,
    items: Vec<ListItem>
}
```

You might notice that the `List` struct does not contain a field that holds a collection of `ListItem`s. This will be implemented using links, which we will discuss later.

Also be sure to replace the list of imports with the following that contains everything required for this example:

```rust
#![feature(proc_macro_hygiene)]
#[macro_use]
extern crate hdk_proc_macros;
#[macro_use]
extern crate hdk;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_json_derive;
use hdk::{
    error::ZomeApiResult,
    holochain_core_types::{
        dna::entry_types::Sharing,
        entry::Entry,
        link::LinkMatch,
    },
    entry_definition::ValidatingEntryType,
    holochain_persistence_api::cas::content::Address,
    holochain_json_api::{
       json::JsonString,
       error::JsonError
    },
};
```


The `Serialize` and `Deserialize` derived traits allow the structs to be converted to and from JSON, which is how entries are managed internally in Holochain. The DefaultJson derived trait comes from the holochain HDK itself and allows for seamless converting between data stored in the DHT and rust structs.

The List and ListItem structs on their own are not yet valid Holochain entries. To create an entry we must write a function that returns a `ValidatingEntryType` and tag it using the `#[entry_def]` attribute. The `entry!` macro makes it easy to define a `ValidatingEntryType`.

```rust

#[zome]
mod todo {
    // -- snip -- //

    #[entry_def]
    fn list_entry_def() -> ValidatingEntryType {
        entry!(
            name: "list",
            description: "",
            sharing: Sharing::Public,
            validation_package: || hdk::ValidationPackageDefinition::Entry,
            validation: |validation_data: hdk::EntryValidationData<List>| {
                Ok(())
            },
            links: [
                to!(
                    "listItem",
                    link_type: "items",
                    validation_package: || hdk::ValidationPackageDefinition::Entry,
                    validation: |_validation_data: hdk::LinkValidationData| {
                        Ok(())
                    }
                )
            ]
        )
    }

    #[entry_def]
    fn list_item_entry_def() -> ValidatingEntryType {
        entry!(
            name: "listItem",
            description: "",
            sharing: Sharing::Public,
            validation_package: || hdk::ValidationPackageDefinition::Entry,
            validation: |_validation_data: hdk::EntryValidationData<ListItem>| {
                Ok(())
            }
        )
    }
}
```

Take note of the use of the struct types (`List` and `ListItem`) in the parameters to the validation function. This will automatically add validation to ensure these entries always match the structures. The `validation_package` field is a function that defines what data should be passed to the validation function through the `validation_data` argument. In this case we use a predefined function to only include the entry itself, but it is also possible to pass chain headers, chain entries or the full local chain. The validation field is a function that performs custom validation for the entry. In both our cases we are just returning `Ok(())`.

Take note also of the `links` field. As we will see later links are the main way to encode relational data in holochain. The `links` section of the entry macro defines what other types of entries are allowed to link to and from this type. This also includes a validation function for fine grain control over linking.


## Adding Functions
Finally we need a way to interact with the hApp. We will define the following functions: `create_list`, `add_item` and `get_list`. get_list will retrieve a list and all the items linked to each list.

Rust functions inside the zome module can be annotated with `#[zome_fn("hc_public")]` which will expose them to be callable from the conductor. The `"hc_public"` defines the capability required to call this function. This guide will not go in depth on capabilities but just know that hc_public means these functions are callable externally with no added security.  You can read more about capabilities [here](./zome/capabilities.md).

It is best practice for functions to always return a `ZomeApiResult<T>`, where `T` is the type the function should return if it runs without error. This is an extension of the Rust Result type and allows zome functions to abort early on errors using the `?` operator. `create_list` could be written as:

```rust
#[zome_fn("hc_public")]
fn create_list(list: List) -> ZomeApiResult<Address> {
    // define the entry
    let list_entry = Entry::App(
        "list".into(),
        list.into()
    );

    // commit the entry and return the address
    hdk::commit_entry(&list_entry)
}
```

The `hdk::commit_entry` function is how a zome can interact with holochain core to add entries to the DHT or local chain. This will trigger the validation function for the entry and if successful will store the entry and return its hash/address.

The `add_item` function requires the use of holochain links to associate two entries. In holochain-proto this required the use of a commit with a special Links entry but it can now be done using the HDK function `link_entries(address1, address2, link_type, tag)`. The `link_type` must exactly match one of the types of links defined in an `entry!` macro for this base (e.g. `link_type: "items"` in this case). The `tag` can be any string we wish to associate with this individual link. We will just use an empty string for this example. The add item handler accepts a `ListItem` and an address of a list, commits the `ListItem`, then links it to the list address:

```rust
#[zome_fn("hc_public")]
fn add_item(list_item: ListItem, list_addr: HashString) -> ZomeApiResult<Address> {
    // define the entry
    let list_item_entry = Entry::App(
        "listItem".into(),
        list_item.into()
    );

    let item_addr = hdk::commit_entry(&list_item_entry)?; // commit the list item
    hdk::link_entries(&list_addr, &item_addr, "items", "")?; // if successful, link to list address
    Ok(item_addr)
}
```

At the moment there is no validation done on the link entries. This will be added soon with an additional validation callback.

Finally, `get_list` requires us to use the HDK function `get_links(base_address, link_type, tag)`. As you may have guessed, this will return the addresses of all the entries that are linked to the `base_address` with a given link_type and a given tag. Both `link_type` and `tag` are LinkMatch types, which is an enum for matching anything, matching exactly, or matching with a regular expression. Passing `LinkMatch::Exactly("string")` means retrieve links that match the type/tag string exactly and passing `LinkMatch::Any` to either of them means to retrieve all links regardless of the type/tag. As this only returns the addresses, we must then map over each of then and load the required entry.

```rust
#[zome_fn("hc_public")]
fn get_list(list_addr: HashString) -> ZomeApiResult<GetListResponse> {

    // load the list entry. Early return error if it cannot load or is wrong type
    let list = hdk::utils::get_as_type::<List>(list_addr.clone())?;

    // try and load the list items, filter out errors and collect in a vector
    let list_items = hdk::get_links(&list_addr, LinkMatch::Exactly("items"), LinkMatch::Any)?.addresses()
        .iter()
        .map(|item_address| {
            hdk::utils::get_as_type::<ListItem>(item_address.to_owned())
        })
        .filter_map(Result::ok)
        .collect::<Vec<ListItem>>();

    // if this was successful then return the list items
    Ok(GetListResponse{
        name: list.name,
        items: list_items
    })
}
```

Phew! and there we have it! If you are coding along the full lib.rs should now look like this:

```rust
#![feature(proc_macro_hygiene)]
#[macro_use]
extern crate hdk_proc_macros;
#[macro_use]
extern crate hdk;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_json_derive;
use hdk::{
    error::ZomeApiResult,
    holochain_core_types::{
        error::HolochainError,
        dna::entry_types::Sharing,
        entry::Entry,
    },
    holochain_persistence_api::cas::content::Address,
    holochain_json_api::{
       json::JsonString,
       error::JsonError
    },
};


#[zome]
mod todo {
    
    #[init]
    fn init() -> ZomeApiResult<()> {
        Ok(())
    }

    #[validate_agent]
    fn validate_agent(validation_data : EntryValidationData::<AgentId>) -> ZomeApiResult {
        Ok(())
    }

    /*=========================================
    =            Entry definitions            =
    =========================================*/
    #[entry_def]
    fn list_entry_def() -> ValidatingEntryType {
        entry!(
            name: "list",
            description: "",
            sharing: Sharing::Public,
            validation_package: || hdk::ValidationPackageDefinition::Entry,
            validation: |validation_data: hdk::EntryValidationData<List>| {
                Ok(())
            },
            links: [
                to!(
                    "listItem",
                    link_type: "items",
                    validation_package: || hdk::ValidationPackageDefinition::Entry,
                    validation: |_validation_data: hdk::LinkValidationData| {
                        Ok(())
                    }
                )
            ]
        )
    }

    #[entry_def]
    fn list_item_entry_def() -> ValidatingEntryType {
        entry!(
            name: "listItem",
            description: "",
            sharing: Sharing::Public,
            validation_package: || hdk::ValidationPackageDefinition::Entry,
            validation: |_validation_data: hdk::EntryValidationData<ListItem>| {
                Ok(())
            }
        )
    }

    /*=====  End of Entry definitions  ======*/

    /*======================================
    =            Zome functions            =
    ======================================*/

    #[zome_fn("hc_public")]
    fn create_list(list: List) -> ZomeApiResult<Address> {
        // define the entry
        let list_entry = Entry::App(
            "list".into(),
            list.into()
        );

        // commit the entry and return the address
        hdk::commit_entry(&list_entry)
    }

    #[zome_fn("hc_public")]
    fn add_item(list_item: ListItem, list_addr: HashString) -> ZomeApiResult<Address> {
        // define the entry
        let list_item_entry = Entry::App(
            "listItem".into(),
            list_item.into()
        );

        let item_addr = hdk::commit_entry(&list_item_entry)?; // commit the list item
        hdk::link_entries(&list_addr, &item_addr, "items", "")?; // if successful, link to list address
        Ok(item_addr)
    }

    #[zome_fn("hc_public")]
    fn get_list(list_addr: HashString) -> ZomeApiResult<GetListResponse> {

        // load the list entry. Early return error if it cannot load or is wrong type
        let list = hdk::utils::get_as_type::<List>(list_addr.clone())?;

        // try and load the list items, filter out errors and collect in a vector
        let list_items = hdk::get_links(&list_addr, LinkMatch::Exactly("items"), LinkMatch::Any)?.addresses()
            .iter()
            .map(|item_address| {
                hdk::utils::get_as_type::<ListItem>(item_address.to_owned())
            })
            .filter_map(Result::ok)
            .collect::<Vec<ListItem>>();

        // if this was successful then return the list items
        Ok(GetListResponse{
            name: list.name,
            items: list_items
        })
    }

    /*=====  End of Zome functions  ======*/
}
```

The Zome we created should now build if we run:

```
hc package
```

from the root directory. This will compile the Rust to WebAssembly and produce a `holochain-rust-todo.dna.json` file in the `dist` folder which contains the compiled WASM code and the required metadata. This is the file that we can load and run using `hc`.

## Writing tests

The testing framework is built on JavaScript around Tape.js and allows for writing single agent and muti-agent tests using javascript async/await syntax.
Opening up the `test/index.js` file you will see a skeleton test file already created:

```javascript
// This test file uses the tape testing framework.
// To learn more, go here: https://github.com/substack/tape
const { Config, Scenario } = require("@holochain/holochain-nodejs")
Scenario.setTape(require("tape"))

const dnaPath = "./dist/holochain-rust-todo.dna.json"
const agentAlice = Config.agent("alice")
const dna = Config.dna(dnaPath)
const instanceAlice = Config.instance(agentAlice, dna)
const scenario = new Scenario([instanceAlice])

scenario.runTape("description of example test", async (t, { alice }) => {
  // Make a call to a Zome function
  // indicating the function, and passing it an input
  const addr = alice.call("my_zome", "create_my_entry", {"entry" : {"content":"sample content"}})
  const result = alice.call("my_zome", "get_my_entry", {"address": addr.Ok})

  // check for equality of the actual and expected results
  t.deepEqual(result, { Ok: { App: [ 'my_entry', '{"content":"sample content"}' ] } })
})
```

This illustrates the `app.call` function that is exposed by the conductor for each app and that can be used to call our functions. Take note that the input-data should be a JSON object that matches the function signature. `call` will also return a JSON object.

Lets add some tests for our todo list:

```javascript
const { Config, Scenario } = require('@holochain/holochain-nodejs')
Scenario.setTape(require('tape'))
const dnaPath = "./dist/holochain-rust-todo.dna.json"
const dna = Config.dna(dnaPath, 'happs')
const agentAlice = Config.agent('alice')
const instanceAlice = Config.instance(agentAlice, dna)
const scenario = new Scenario([instanceAlice])

scenario.runTape('Can create a list', async (t, { alice }) => {
  const createResult = await alice.callSync('lists', 'create_list', { list: { name: 'test list' } })
  console.log(createResult)
  t.notEqual(createResult.Ok, undefined)
})

scenario.runTape('Can add some items', async (t, { alice }) => {
  const createResult = await alice.callSync('lists', 'create_list', { list: { name: 'test list' } })
  const listAddr = createResult.Ok

  const result1 = await alice.callSync('lists', 'add_item', { list_item: { text: 'Learn Rust', completed: true }, list_addr: listAddr })
  const result2 = await alice.callSync('lists', 'add_item', { list_item: { text: 'Master Holochain', completed: false }, list_addr: listAddr })

  console.log(result1)
  console.log(result2)

  t.notEqual(result1.Ok, undefined)
  t.notEqual(result2.Ok, undefined)
})

scenario.runTape('Can get a list with items', async (t, { alice }) => {
  const createResult = await alice.callSync('lists', 'create_list', { list: { name: 'test list' } })
  const listAddr = createResult.Ok

  await alice.callSync('lists', 'add_item', { list_item: { text: 'Learn Rust', completed: true }, list_addr: listAddr })
  await alice.callSync('lists', 'add_item', { list_item: { text: 'Master Holochain', completed: false }, list_addr: listAddr })

  const getResult = await alice.callSync('lists', 'get_list', { list_addr: listAddr })
  console.log(getResult)

  t.equal(getResult.Ok.items.length, 2, 'there should be 2 items in the list')
})
```

Running `hc test` will build the test file and run it using `node` which is able to load and execute holochain hApps via the holochain node conductor. If everything has worked correctly you should see some test output with everything passing.

Pro tip: [Pipe the output to tap-spec](https://github.com/scottcorgan/tap-spec) (which must be installed via npm first) to get beautifully formatted test output.

## Conclusion

And there we have it! A simple Zome created with Holochain using the Rust HDK.

The [complete working version of this project is available on github](https://github.com/willemolding/holochain-rust-todo). This builds under the 0.0.9-alpha release but as the API and HDK are changing it will likely fail under newer releases.
