
/// Types required by all but the most trivial zomes. 
/// This can greatly simplify imports for the majority of developers
/// by simply adding use hdk::prelude::*;


// macros
pub use crate::{
	define_zome,
	entry,
	link, to, from, load_json,
};

// derive macros
pub use {
	serde_derive::{Serialize, Deserialize},
	holochain_json_derive::DefaultJson,
};

// types
pub use crate::{
	holochain_json_api::{
		json::{JsonString},
		error::JsonError,
	},
	holochain_persistence_api::cas::content::{Address, AddressableContent},
	error::{ZomeApiError, ZomeApiResult},
	entry_definition::ValidatingEntryType,
	EntryValidationData, LinkValidationData,
	ValidationPackageDefinition,
	holochain_core_types::{
		error::HolochainError,
	    dna::entry_types::Sharing,
	    entry::{Entry, entry_type::EntryType},
	    link::LinkMatch,
	    agent::AgentId,
	},
    holochain_wasm_utils::api_serialization::{
        commit_entry::CommitEntryOptions,
        get_entry::{
            EntryHistory, GetEntryOptions, GetEntryResult, GetEntryResultType, StatusRequestKind,
        },
        get_links::{GetLinksOptions, GetLinksResult, LinksStatusRequestKind, GetLinksResultCount},
        QueryArgsOptions, QueryResult,
    },
};
