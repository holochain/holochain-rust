# Example - Validation of a Peer Chat message

A Peer Chat message needs to be >= 1 characters and <= 1024 characters. This can be neatly implemented using the [Validator](!https://crates.io/crates/validator) and [Validator Derive](!https://crates.io/crates/validator_derive) crates.


In the Cargo.toml add in the following dependencies.
```=rust
[dependencies]
  ...
  validator = "0.9.0"
  validator_derive = "0.9.0"
    
```
In the lib.rs file 
```=rust
#[macro_use]
extern crate validator_derive;
extern crate validator;
```

In the Message mod.rs file add the `Validate` trait and set the validation rule on the payload property.

```=rust
use validator::{Validate};

#[derive(Serialize, Deserialize, Debug, Clone, DefaultJson, Validate)]
pub struct Message {
    pub timestamp: u32,
    pub author: String,
    pub message_type: String,
    #[validate(length(min = 1, max = 1024))]
    pub payload: String,
    pub meta: String,
}
```

Then add the validation logic into the entry definition. The validation extracts the Message from the validation entry and runs the `validate()` function, if it passes Ok(()) is returned otherwise the Error is returned.

```=rust
pub fn message_definition() -> ValidatingEntryType {
    entry!(
        name: MESSAGE_ENTRY,
        description: "A generic message entry",
        sharing: Sharing::Public,

        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: | validation_data: hdk::EntryValidationData<Message>| {
            match validation_data {
                EntryValidationData::Create{entry, ..} => {
                    let new_message = Message::from(entry);
                    match new_message.validate() {
                      Ok(_) => Ok(()),
                      Err(e) => Err(e.to_string())
                    }
                },
                _ => {
                    Err("Cannot modify or delete a message".into())
                }
            }
        }
    )
}
```
