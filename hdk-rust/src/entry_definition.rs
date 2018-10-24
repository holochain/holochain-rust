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
