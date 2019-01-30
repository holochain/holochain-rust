//! This file contains the macros used for creating validating entry type definitions,
//! and validating links definitions within those.

use holochain_core_types::{
    dna::entry_types::EntryTypeDef,
    entry::{entry_type::EntryType, Entry},
    hash::HashString,
    validation::{ValidationData, ValidationPackageDefinition},
};
use holochain_wasm_utils::api_serialization::validation::LinkDirection;

pub type PackageCreator = Box<FnMut() -> ValidationPackageDefinition + Sync>;

pub type Validator = Box<FnMut(Entry, ValidationData) -> Result<(), String> + Sync>;

pub type LinkValidator =
    Box<FnMut(HashString, HashString, ValidationData) -> Result<(), String> + Sync>;

/// This struct represents a complete entry type definition.
/// It wraps [EntryTypeDef](struct.EntryTypeDef.html) defined in the DNA crate
/// which only represents the static parts that show up in the JSON definition
/// of an entry type.
/// What is missing from there is the validation callbacks that can not be defined as JSON
/// and are added here as Box<FnMut> objects (types PackageCreator, Validator, LinkValidator)
///
/// Instances of this struct are expected and used in the [define_zome! macro](macro.define_zome.html).
/// Although possible, a DNA developer does not need to create these instances directly but instead
/// should use the [entry! macro](macro.entry.html) for a clean syntax.
pub struct ValidatingEntryType {
    /// Name of the entry type
    pub name: EntryType,
    /// All the static aspects of the entry type as
    pub entry_type_definition: EntryTypeDef,
    /// Callback that returns a validation package definition that Holochain reads in order
    /// to create the right validation package to pass in to the validator callback on validation.
    pub package_creator: PackageCreator,
    /// This is the validation callback that is used to determine if an entry is valid.
    pub validator: Validator,

    pub links: Vec<ValidatingLinkDefinition>,
}

/// Similar to ValidatingEntryType, this provides the dynamic aspects of link definitions,
/// the validation callbacks, and thus completes the structs in the DNA crate.
/// The [entry! macro](macro.entry.html) expects an array of links that are represented by
/// instances of this struct.
///
/// DNA developers don't need to use this type directly but instead should use the
/// [link!](macro.link.html), [to!](macro.to.html) or [from!](macro.from.html) macro.
pub struct ValidatingLinkDefinition {
    /// Is this link defined as pointing from this entry type to some other type,
    /// or from the other type to this?
    pub link_type: LinkDirection,
    /// The other entry type the link connects this entry type to
    pub other_entry_type: String,
    /// Tag (i.e. name) of this type of links
    pub tag: String,
    /// Callback that returns a validation package definition that Holochain reads in order
    /// to create the right validation package to pass in to the validator callback on validation.
    pub package_creator: PackageCreator,
    /// This is the validation callback that is used to determine if a link is valid.
    pub validator: LinkValidator,
}

/// The `entry` macro is a helper for creating `ValidatingEntryType` definitions
/// for use within the [define_zome](macro.define_zome.html) macro.
/// It has 7 component parts:
/// 1. name: `name` is simply the descriptive name of the entry type, such as "post", or "user".
///      It is what must be given as the `entry_type_name` argument when calling [commit_entry](fn.commit_entry.html) and the other data read/write functions.
/// 2. description: `description` is something that is primarily for human readers of your code, just describe this entry type
/// 3. sharing: `sharing` defines what distribution over the DHT, or not, occurs with entries of this type, possible values
///      are defined in the [Sharing](../core_types/entry/dna/zome/entry_types/enum.Sharing.html) enum
/// 4. native_type: `native_type` references a given Rust struct, which provides a clear schema for entries of this type.
/// 5. validation_package: `validation_package` is a special identifier, which declares which data is required from peers
///      when attempting to validate entries of this type.
///      Possible values are found within [ValidationPackageDefinition](enum.ValidationPackageDefinition.html)
/// 6. validation: `validation` is a callback function which will be called any time that a
///      (DHT) node processes or stores this entry, triggered through actions such as [commit_entry](fn.commit_entry.html), [update_entry](fn.update_entry.html), [remove_entry](fn.remove_entry.html).
///      It always expects two arguments, the first of which is the entry attempting to be validated,
///      the second is the validation `context`, which offers a variety of metadata useful for validation.
///      See [ValidationData](struct.ValidationData.html) for more details.
/// 7. links: `links` is a vector of link definitions represented by `ValidatingLinkDefinition`.
///     Links can be defined with the `link!` macro or, more concise, with either the `to!` or `from!` macro,
///     to define an association pointing from this entry type to another, or one that points back from
///     the other entry type to this one.
///     See [link!](macro.link.html), [to!](macro.to.html) and [from!](macro.to.html) for more details.
/// # Examples
/// The following is a standalone Rust file that exports a function which can be called
/// to get a `ValidatingEntryType` of a "post".
/// ```rust
/// # #![feature(try_from)]
/// # extern crate boolinator;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate hdk;
/// # #[macro_use]
/// # extern crate holochain_core_types_derive;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # use boolinator::*;
/// # use hdk::entry_definition::ValidatingEntryType;
/// # use hdk::holochain_core_types::{
/// #   cas::content::Address,
/// #   dna::entry_types::Sharing,
/// #   json::JsonString,
/// #   error::HolochainError,
/// # };
///
/// # fn main() {
///
/// #[derive(Serialize, Deserialize, Debug, DefaultJson)]
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
///         validation: |post: Post, _validation_data: hdk::ValidationData| {
///             (post.content.len() < 280)
///                 .ok_or_else(|| String::from("Content too long"))
///         },
///
///         links: [
///             to!(
///                 "post",
///                 tag: "comments",
///
///                 validation_package: || {
///                     hdk::ValidationPackageDefinition::ChainFull
///                 },
///
///                 validation: |base: Address, target: Address, _validation_data: hdk::ValidationData| {
///                     Ok(())
///                 }
///             )
///         ]
///     )
/// }
///
/// # }
/// ```

#[macro_export]
macro_rules! entry {
    (
        name: $name:expr,
        description: $description:expr,
        sharing: $sharing:expr,
        $(native_type: $native_type:ty,)*

        validation_package: || $package_creator:expr,
        validation: | $entry:ident : $entry_type:ty, $validation_data:ident : hdk::ValidationData | $entry_validation:expr

        $(
            ,
            links : [
                $( $link_expr:expr ),*
            ]
        )*

    ) => (

        {
            let mut entry_type = hdk::holochain_core_types::dna::entry_types::EntryTypeDef::new();
            entry_type.description = String::from($description);
            entry_type.sharing = $sharing;

            $($(
                match $link_expr.link_type {
                    $crate::LinkDirection::To => {
                        entry_type.links_to.push(
                            $crate::holochain_core_types::dna::entry_types::LinksTo{
                                target_type: $link_expr.other_entry_type,
                                tag: $link_expr.tag,
                            }
                        );
                    },
                    $crate::LinkDirection::From => {
                        entry_type.linked_from.push(
                            $crate::holochain_core_types::dna::entry_types::LinkedFrom{
                                base_type: $link_expr.other_entry_type,
                                tag: $link_expr.tag,
                            }
                        );
                    }
                }

            )*)*

            let package_creator = Box::new(|| {
                $package_creator
            });

            let validator = Box::new(|entry: hdk::holochain_core_types::entry::Entry, validation_data: hdk::holochain_wasm_utils::holochain_core_types::validation::ValidationData| {
                let $validation_data = validation_data;
                match entry {
                    hdk::holochain_core_types::entry::Entry::App(_, app_entry_value) => {
                        let entry: $entry_type = ::std::convert::TryInto::try_into(app_entry_value)?;
                        let $entry = entry;
                        $entry_validation
                    },
                    _ => {
                        Err(String::from("Schema validation failed"))?
                    }
                }
            });

            hdk::entry_definition::ValidatingEntryType {
                name: hdk::holochain_core_types::entry::entry_type::EntryType::App(hdk::holochain_core_types::entry::entry_type::AppEntryType::from($name.to_string())),
                entry_type_definition: entry_type,
                package_creator,
                validator,
                links: vec![
                    $($(
                        $link_expr
                    ),*)*
                ],
            }
        }
    );
}

/// The `link` macro is a helper for creating `ValidatingEntryType` definitions
/// for use within the [entry](macro.entry.html) macro.
/// It has 5 component parts:
/// 1. direction: `direction` defines if this entry type (in which the link is defined) points
///     to another entry, or if it is referenced from another entry.
///     The latter is needed in cases where the definition of the entry to link from is not
///     accessible because it is a system entry type (AGENT_ADDRESS), or the other entry is
///     defined in library zome.
///     Must be of type [LinkDirection](struct.LinkDirection.html), so either `hdk::LinkDirection::To`
///     or `hdk::LinkDirection::From`.
/// 2. other_type: `other_type` is the entry type this link connects to. If direction is `to` this
///     would be the link target, if direction is `from` this defines the link's base type.
/// 3. tag: `tag` is the name of this association and thus the handle by which it can be retrieved
///     if given to [get_links()](fn.get_links.html) in conjunction with the base address.
/// 4. validation_package: Similar to entries, links have to be validated.
///        `validation_package` is a special identifier, which declares which data is required from peers
///         when attempting to validate entries of this type.
///         Possible values are found within [ValidationPackageDefinition](enum.ValidationPackageDefinition.html)
/// 5. validation: `validation` is a callback function which will be called any time that a
///         (DHT) node processes or stores a link of this kind, triggered through the link actions [link_entries](fn.commit_entry.html) and [remove_link]().
///         It always expects three arguments, the first being the base and the second the target of the link.
///         The third is the validation `context`, which offers a variety of metadata useful for validation.
///         See [ValidationData](struct.ValidationData.html) for more details.
#[macro_export]
macro_rules! link {
    (
        direction: $direction:expr,
        other_type: $other_type:expr,
        tag: $tag:expr,

        validation_package: || $package_creator:expr,
        validation: | $source:ident : Address,  $target:ident : Address, $validation_data:ident : hdk::ValidationData | $link_validation:expr
    ) => (

        {
            let package_creator = Box::new(|| {
                $package_creator
            });

            let validator = Box::new(|source: Address, target: Address, validation_data: ::hdk::holochain_wasm_utils::holochain_core_types::validation::ValidationData| {
                let $source = source;
                let $target = target;
                let $validation_data = validation_data;
                $link_validation
            });


            ::hdk::entry_definition::ValidatingLinkDefinition {
                link_type: $direction,
                other_entry_type: String::from($other_type),
                tag: String::from($tag),
                package_creator,
                validator,
            }
        }
    );
}

/// The `to` macro is a helper for creating `ValidatingEntryType` definitions
/// for use within the [entry](macro.entry.html) macro.
/// It is a convenience wrapper around [link!](macro.link.html) that has all the
/// same properties except for the direction which gets set to `LinkDirection::To`.
#[macro_export]
macro_rules! to {
    (
        $other_type:expr,
        tag: $tag:expr,

        validation_package: || $package_creator:expr,
        validation: | $source:ident : Address,  $target:ident : Address, $validation_data:ident : hdk::ValidationData | $link_validation:expr
    ) => (
        link!(
            direction: $crate::LinkDirection::To,
            other_type: $other_type,
            tag: $tag,

            validation_package: || $package_creator,
            validation: | $source : Address,  $target : Address, $validation_data : hdk::ValidationData | $link_validation
        )
    )
}

/// The `from` macro is a helper for creating `ValidatingEntryType` definitions
/// for use within the [entry](macro.entry.html) macro.
/// It is a convenience wrapper around [link!](macro.link.html) that has all the
/// same properties except for the direction which gets set to `LinkDirection::From`.
#[macro_export]
macro_rules! from {
    (
        $other_type:expr,
        tag: $tag:expr,

        validation_package: || $package_creator:expr,
        validation: | $source:ident : Address,  $target:ident : Address, $validation_data:ident : hdk::ValidationData | $link_validation:expr
    ) => (
        link!(
            direction: $crate::LinkDirection::From,
            other_type: $other_type,
            tag: $tag,

            validation_package: || $package_creator,
            validation: | $source : Address,  $target : Address, $validation_data : hdk::ValidationData | $link_validation
        )
    )
}
