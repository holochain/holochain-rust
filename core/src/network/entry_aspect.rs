use holochain_core_types::{
 //   cas::content::Address,
    chain_header::ChainHeader,
    entry::{Entry},
    error::HolochainError,
    link::link_data::LinkData,
    json::JsonString,
};


#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
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
  LinkRemove(LinkData, ChainHeader),


    // CLEANUP this looks wrong to me.  I don't think we actually want to
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
  Update(ChainHeader),

  // Meta item for removing an entry.
  // Address is the address of the deleted entry.
  // ChainHeader is the header of that deletion entry that
  // could be assembled by putting the address in an
  // `Entry::Deletion(address)`.
    // Deletion(Address, ChainHeader),
    Deletion(ChainHeader),
}
