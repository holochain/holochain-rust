use crate::{chain_header::ChainHeader, entry::Entry, link::link_data::LinkData};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::cas::content::{Address, AddressableContent, Content};
use std::{
    convert::{Into, TryFrom},
    fmt,
};

impl AddressableContent for EntryAspect {
    fn content(&self) -> Content {
        self.to_owned().into()
    }

    fn try_from_content(content: &Content) -> Result<Self, JsonError> {
        Self::try_from(content.to_owned())
    }
}

#[derive(Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum EntryAspect {
    // Basic case: entry content is communicated
    // with its header.
    // Content alone never makes sense
    // (receiveing node needs header and especially
    // source to run validation)
    Content(Entry, ChainHeader),

    // Communicating only the header makes sense if an
    // entry was deleted but we need to remember that
    // there was an entry that got deleted (sacrileged)
    Header(ChainHeader),

    // This is the meta item for adding a link.
    // The ChainHeader is needed for validation of
    // this LinkAdd statement/entry.
    // (Instead of type `LinkData` we could also have
    // an `Entry` that would always be expected to the be
    // `Entry::LinkAdd` specialisation, but in order to make
    // impossible states impossible we choose `LinkData`.
    // Putting that `LinkData` in an `Entry::LinkAdd` should
    // result in the exact same entry the `ChainHeader` is
    // a header for)
    LinkAdd(LinkData, ChainHeader),

    // Same as LinkAdd but for removal of links
    LinkRemove((LinkData, Vec<Address>), ChainHeader),

    // TODO this looks wrong to me.  I don't think we actually want to
    // send the updated Entry as part of the meta item.  That would mean the
    // new entry is getting stored two places on the dht.  I think this
    // should look the same same as Deletion
    // AND, we don't actually need to even have the Address as part of the
    // Variant because the correct value is already in the Chain Header
    // as the link_update_delete attribute
    // Meta item for updating an entry.
    // The given Entry is the new version and ChainHeader
    // the header of the new version.
    // The header's CRUD link must reference the base address
    // of the EntryData this is in.
    //  Update(Entry, ChainHeader),
    Update(Entry, ChainHeader),

    // Meta item for removing an entry.
    // Address is the address of the deleted entry.
    // ChainHeader is the header of that deletion entry that
    // could be assembled by putting the address in an
    // `Entry::Deletion(address)`.
    // Deletion(Address, ChainHeader),
    Deletion(ChainHeader),
}

impl EntryAspect {
    pub fn type_hint(&self) -> String {
        match self {
            EntryAspect::Content(_, _) => String::from("content"),
            EntryAspect::Header(_) => String::from("header"),
            EntryAspect::LinkAdd(_, _) => String::from("link_add"),
            EntryAspect::LinkRemove(_, _) => String::from("link_remove"),
            EntryAspect::Update(_, _) => String::from("update"),
            EntryAspect::Deletion(_) => String::from("deletion"),
        }
    }
    pub fn header(&self) -> ChainHeader {
        match self {
            EntryAspect::Content(_, header) => header.clone(),
            EntryAspect::Header(header) => header.clone(),
            EntryAspect::LinkAdd(_, header) => header.clone(),
            EntryAspect::LinkRemove(_, header) => header.clone(),
            EntryAspect::Update(_, header) => header.clone(),
            EntryAspect::Deletion(header) => header.clone(),
        }
    }
}

fn format_header(header: &ChainHeader) -> String {
    format!(
        "Header[type: {}, crud_link: {:?}]",
        header.entry_type(),
        header.link_update_delete()
    )
}
impl fmt::Debug for EntryAspect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntryAspect::Content(entry, header) => write!(
                f,
                "EntryAspect::Content({}, {})",
                entry.address(),
                format_header(header)
            ),
            EntryAspect::Header(header) => {
                write!(f, "EntryAspect::Header({})", format_header(header))
            }
            EntryAspect::LinkAdd(link_data, header) => write!(
                f,
                "EntryAspect::LinkAdd({} -> {} [tag: {}, type: {}], {})",
                link_data.link.base(),
                link_data.link.target(),
                link_data.link.tag(),
                link_data.link.link_type(),
                format_header(header)
            ),
            EntryAspect::LinkRemove((link_data, _), header) => write!(
                f,
                "EntryAspect::LinkRemove({} -> {} [tag: {}, type: {}], {})",
                link_data.link.base(),
                link_data.link.target(),
                link_data.link.tag(),
                link_data.link.link_type(),
                format_header(header)
            ),
            EntryAspect::Update(entry, header) => write!(
                f,
                "EntryAspect::Update({}, {})",
                entry.address(),
                format_header(header)
            ),
            EntryAspect::Deletion(header) => {
                write!(f, "EntryAspect::Deletion({})", format_header(header))
            }
        }
    }
}
