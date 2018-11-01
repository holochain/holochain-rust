use holochain_dna::zome::entry_types::EntryTypeDef;
use holochain_wasm_utils::holochain_core_types::{
    hash::HashString,
    validation::{ValidationData, ValidationPackageDefinition},
};
use std::collections::HashMap;

pub type PackageCreator = Box<FnMut() -> ValidationPackageDefinition + Sync>;
pub type Validator = Box<FnMut(String, ValidationData) -> Result<(), String> + Sync>;
pub type LinkValidator =
    Box<FnMut(HashString, String, HashString, ValidationData) -> Result<(), String> + Sync>;

pub struct ValidatingEntryType {
    pub name: String,
    pub entry_type_definition: EntryTypeDef,
    pub package_creator: PackageCreator,
    pub validator: Validator,
    pub link_validators: HashMap<String, LinkValidator>,
}

/// The `entry` macro is a helper for creating `ValidatingEntryType` definitions
/// for use within the [define_zome](macro.define_zome.html) macro.
/// It has 6 component parts:
/// 1. name: `name` is simply the descriptive name of the entry type, such as "post", or "user".
///      It is what must be given as the `entry_type_name` argument when calling [commit_entry](fn.commit_entry.html) and the other data read/write functions.
/// 2. description: `description` is something that is primarily for human readers of your code, just describe this entry type
/// 3. sharing: `sharing` defines what distribution over the DHT, or not, occurs with entries of this type, possible values
///      are defined in the [Sharing](../holochain_dna/zome/entry_types/enum.Sharing.html) enum
/// 4. native_type: `native_type` references a given Rust struct, which provides a clear schema for entries of this type.
/// 5. validation_package: `validation_package` is a special identifier, which declares which data is required from peers
///      when attempting to validate entries of this type.
///      Possible values are found within [ValidationPackageDefinition](enum.ValidationPackageDefinition.html)
/// 6. validation: `validation` is a callback function which will be called any time that a
///      (DHT) node processes or stores this entry, triggered through actions such as [commit_entry](fn.commit_entry.html), [update_entry](fn.update_entry.html), [remove_entry](fn.remove_entry.html).
///      It always expects two arguments, the first of which is the entry attempting to be validated,
///      the second is the validation `context`, which offers a variety of metadata useful for validation.
/// # Examples
/// The following is a standalone Rust file that exports a function which can be called
/// to get a `ValidatingEntryType` of a "post".
/// ```rust
/// use boolinator::*;
/// use hdk::{
///   self,
///   entry_definition::ValidatingEntryType,
///   holochain_dna::zome::entry_types::Sharing
/// };
/// use serde_json;
///
/// #[derive(Serialize, Deserialize)]
/// pub struct Post {
///     content: String,
///     date_created: String,
/// }
///
/// pub fn definition() -> ValidatingEntryType {
///     entry!(
///         name: "post",
///         description: "a short social media style sharing of content",
///         sharing: Sharing::Public,
///         native_type: Post,
///
///         validation_package: || {
///             hdk::ValidationPackageDefinition::ChainFull
///         },
///
///         validation: |post: Post, _ctx: hdk::ValidationData| {
///             (post.content.len() < 280)
///                 .ok_or_else(|| String::from("Content too long"))
///         }
///     )
/// }
/// ```

#[macro_export]
macro_rules! entry {
    (
        name: $name:expr,
        description: $description:expr,
        sharing: $sharing:expr,
        $(native_type: $native_type:ty,)*

        validation_package: || $package_creator:expr,
        validation: | $entry:ident : $entry_type:ty, $ctx:ident : hdk::ValidationData | $entry_validation:expr
    ) => (

        {
            let mut entry_type = ::hdk::holochain_dna::zome::entry_types::EntryTypeDef::new();
            entry_type.description = String::from($description);
            entry_type.sharing = $sharing;

            let package_creator = Box::new(|| {
                $package_creator
            });

            let validator = Box::new(|raw_entry: String, ctx: ::hdk::holochain_wasm_utils::holochain_core_types::validation::ValidationData| {
                let $ctx = ctx;
                match serde_json::from_str(&raw_entry) {
                    Ok(entry) => {
                        let entry_struct : $entry_type = entry;
                        let $entry = entry_struct;
                        $entry_validation
                    },
                    Err(_) => {
                        Err(String::from("Schema validation failed"))
                    }
                }
            });


            ::hdk::entry_definition::ValidatingEntryType {
                name: String::from($name),
                entry_type_definition: entry_type,
                package_creator,
                validator,
                link_validators: std::collections::HashMap::new(),
            }
        }
    );
}
