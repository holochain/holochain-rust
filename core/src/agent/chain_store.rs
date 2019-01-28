use globset::{GlobBuilder, GlobSetBuilder};
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    chain_header::ChainHeader,
    entry::entry_type::EntryType,
    error::RibosomeErrorCode::{self, *},
};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct ChainStore {
    // Storages holding local shard data
    content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
}

impl PartialEq for ChainStore {
    fn eq(&self, other: &ChainStore) -> bool {
        let storage_lock = &self.content_storage.clone();
        let storage = &*storage_lock.read().unwrap();
        let other_storage_lock = &other.content_storage.clone();
        let other_storage = &*other_storage_lock.read().unwrap();
        storage.get_id() == other_storage.get_id()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ChainStoreQueryOptions {
    pub start: usize,
    pub limit: usize,
    pub headers: bool,
}

#[derive(Debug)]
pub enum ChainStoreQueryResult {
    Addresses(Vec<Address>),
    Headers(Vec<ChainHeader>),
}

impl ChainStore {
    pub fn new(content_storage: Arc<RwLock<dyn ContentAddressableStorage>>) -> Self {
        ChainStore { content_storage }
    }

    pub fn content_storage(&self) -> Arc<RwLock<dyn ContentAddressableStorage>> {
        self.content_storage.clone()
    }

    pub fn iter(&self, start_chain_header: &Option<ChainHeader>) -> ChainStoreIterator {
        ChainStoreIterator::new(self.content_storage.clone(), start_chain_header.clone())
    }

    /// Scans the local chain for the first Entry of EntryType, and then creates a
    /// ChainStoreTypeIter to return the sequence of all Entrys with the same EntryType. Requires a
    /// single EntryType.
    pub fn iter_type(
        &self,
        start_chain_header: &Option<ChainHeader>,
        entry_type: &EntryType,
    ) -> ChainStoreTypeIterator {
        ChainStoreTypeIterator::new(
            self.content_storage.clone(),
            self.iter(start_chain_header)
                .find(|chain_header| chain_header.entry_type() == entry_type),
        )
    }

    // Supply a None for options to get defaults (all elements, no ChainHeaders just Addresses)
    pub fn query(
        &self,
        start_chain_header: &Option<ChainHeader>,
        entry_type_names: &[&str],
        options: ChainStoreQueryOptions,
    ) -> Result<ChainStoreQueryResult, RibosomeErrorCode> {
        // Get entry_type name(s), if any.  If empty/blank, returns the complete source chain.  A
        // single matching entry type name with no glob pattern matching will use the single
        // entry_type optimization.  Otherwise, we'll construct a GlobSet match and scan the list to
        // create a pattern-match engine to select the EntryTypes we want.
        fn is_glob(c: &char) -> bool {
            "./*[]{}".chars().any(|y| y == *c)
        }
        fn is_glob_str(s: &str) -> bool {
            s.chars().any(|c| is_glob(&c))
        }

        // Unpack options; start == 0 --> start at beginning, limit == 0 --> take all remaining
        let start = options.start;
        let limit = if options.limit == 0 {
            usize::max_value()
        } else {
            options.limit
        };
        let headers = options.headers;

        let vector = match entry_type_names {
            // Vec<Address> or Vec<ChainHeader>
            [] | [""] | ["**"] => {
                // No filtering desired; uses bare .iter()
                if headers {
                    ChainStoreQueryResult::Headers(
                        self.iter(start_chain_header)
                            .skip(start)
                            .take(limit)
                            .map(|header| header.to_owned())
                            .collect(),
                    )
                } else {
                    ChainStoreQueryResult::Addresses(
                        self.iter(start_chain_header)
                            .skip(start)
                            .take(limit)
                            .map(|header| header.entry_address().to_owned())
                            .collect(),
                    )
                }
            }
            [one] if !is_glob_str(one) => {
                // Single EntryType without "glob" pattern; uses .iter_type()
                let entry_type = match EntryType::from_str(&one) {
                    Ok(inner) => inner,
                    Err(..) => return Err(UnknownEntryType),
                };
                if headers {
                    ChainStoreQueryResult::Headers(
                        self.iter_type(start_chain_header, &entry_type)
                            .skip(start)
                            .take(limit)
                            .map(|header| header.to_owned())
                            .collect(),
                    )
                } else {
                    ChainStoreQueryResult::Addresses(
                        self.iter_type(start_chain_header, &entry_type)
                            .skip(start)
                            .take(limit)
                            .map(|header| header.entry_address().to_owned())
                            .collect(),
                    )
                }
            }
            rest => {
                // 1 or more EntryTypes, may or may not include glob wildcards.  Create a
                // GlobSetBuilder and add all the EntryType name patterns to it; this will recognize
                // all matching EntryTypes using a single regex machine invocation.  In order to
                // support .../... EntryType namespaces, force the '/' path separator to match
                // literally.
                let mut builder = GlobSetBuilder::new();
                for name in rest {
                    builder.add(
                        GlobBuilder::new(name)
                            .literal_separator(true)
                            .build()
                            .map_err(|_| UnknownEntryType)?,
                    );
                }
                let globset = builder.build().map_err(|_| UnknownEntryType)?;
                if headers {
                    ChainStoreQueryResult::Headers(
                        self.iter(start_chain_header)
                            .filter(|header| {
                                globset.matches(header.entry_type().to_string()).len() > 0
                            })
                            .skip(start)
                            .take(limit)
                            .map(|header| header.to_owned())
                            .collect(),
                    )
                } else {
                    ChainStoreQueryResult::Addresses(
                        self.iter(start_chain_header)
                            .filter(|header| {
                                globset.matches(header.entry_type().to_string()).len() > 0
                            })
                            .skip(start)
                            .take(limit)
                            .map(|header| header.entry_address().to_owned())
                            .collect(),
                    )
                }
            }
        };

        Ok(vector)
    }
}

/// Access each Entry
///
/// # Remarks
///
/// Locates the next Entry by following ChainHeader's .link
///
pub struct ChainStoreIterator {
    content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
    current: Option<ChainHeader>,
}

impl ChainStoreIterator {
    pub fn new(
        content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
        current: Option<ChainHeader>,
    ) -> ChainStoreIterator {
        ChainStoreIterator {
            content_storage,
            current,
        }
    }
}

/// Follows ChainHeader.link through every previous Entry (of any EntryType) in the chain
impl Iterator for ChainStoreIterator {
    type Item = ChainHeader;

    /// May panic if there is an underlying error in the table
    fn next(&mut self) -> Option<ChainHeader> {
        let previous = self.current.take();
        let storage = &self.content_storage.clone();
        self.current = previous
            .as_ref()
            .and_then(|chain_header| chain_header.link())
            .as_ref()
            // @TODO should this panic?
            // @see https://github.com/holochain/holochain-rust/issues/146
            .and_then(|linked_chain_header_address| {
                storage
                    .read()
                    .unwrap()
                    .fetch(linked_chain_header_address)
                    .expect("failed to fetch from CAS")
                    .map(|content| {
                        ChainHeader::try_from_content(&content)
                            .expect("failed to load ChainHeader from Content")
                    })
            });
        previous
    }
}

/// Quickly access each Entry of a single known EntryType
///
/// # Remarks
///
/// Iterates over subsequent instances of the same EntryType using .link_same_type.
///
/// This Iterator will only work with a single EntryType; it cannot handle None (wildcard) or
/// multiple EntryType queries.
///
pub struct ChainStoreTypeIterator {
    content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
    current: Option<ChainHeader>,
}

impl ChainStoreTypeIterator {
    pub fn new(
        content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
        current: Option<ChainHeader>,
    ) -> ChainStoreTypeIterator {
        ChainStoreTypeIterator {
            content_storage,
            current,
        }
    }
}

/// Follows ChainHeader.link_same_type through every previous Entry of the same EntryType in the chain
impl Iterator for ChainStoreTypeIterator {
    type Item = ChainHeader;

    /// May panic if there is an underlying error in the table
    fn next(&mut self) -> Option<ChainHeader> {
        let previous = self.current.take();
        let storage = &self.content_storage.clone();
        self.current = previous
            .as_ref()
            .and_then(|chain_header| chain_header.link_same_type())
            .as_ref()
            // @TODO should this panic?
            // @see https://github.com/holochain/holochain-rust/issues/146
            .and_then(|linked_chain_header_address| {
                storage
                    .read()
                    .unwrap()
                    .fetch(linked_chain_header_address)
                    .expect("failed to fetch from CAS")
                    .map(|content| {
                        ChainHeader::try_from_content(&content)
                            .expect("failed to load ChainHeader from Content")
                    })
            });
        previous
    }
}

#[cfg(test)]
pub mod tests {
    extern crate tempfile;
    use self::tempfile::tempdir;
    use crate::agent::chain_store::{ChainStore, ChainStoreQueryOptions, ChainStoreQueryResult};
    use holochain_cas_implementations::cas::file::FilesystemStorage;
    use holochain_core_types::{
        cas::content::AddressableContent,
        chain_header::{test_chain_header, test_provenances, ChainHeader},
        entry::{
            entry_type::{test_entry_type_b, AppEntryType},
            test_entry, test_entry_b, test_entry_c, Entry,
        },
        json::JsonString,
        signature::{test_signature_b, test_signature_c, test_signatures, Signature},
        time::test_iso_8601,
    };

    pub fn test_chain_store() -> ChainStore {
        ChainStore::new(std::sync::Arc::new(std::sync::RwLock::new(
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap())
                .expect("could not create chain store"),
        )))
    }

    #[test]
    /// show Iterator implementation for chain store
    fn iterator_test() {
        let chain_store = test_chain_store();

        let entry = test_entry_b();
        let chain_header_a = test_chain_header();
        let chain_header_b = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_provenances(),
            &vec![test_signature_b()],
            &Some(chain_header_a.address()),
            &None,
            &None,
            &test_iso_8601(),
        );

        let storage = chain_store.content_storage.clone();
        (*storage.write().unwrap())
            .add(&chain_header_a)
            .expect("could not add header to cas");
        (*storage.write().unwrap())
            .add(&chain_header_b)
            .expect("could not add header to cas");

        let expected = vec![chain_header_b.clone(), chain_header_a.clone()];
        let mut found = vec![];
        for chain_header in chain_store.iter(&Some(chain_header_b)) {
            found.push(chain_header);
        }
        assert_eq!(expected, found);

        let expected = vec![chain_header_a.clone()];
        let mut found = vec![];
        for chain_header in chain_store.iter(&Some(chain_header_a)) {
            found.push(chain_header);
        }
        assert_eq!(expected, found);
    }

    #[test]
    /// show entry typed Iterator implementation for chain store
    fn type_iterator_test() {
        let chain_store = test_chain_store();

        let chain_header_a = test_chain_header();
        // b has a different type to a
        let entry_b = test_entry_b();
        let chain_header_b = ChainHeader::new(
            &entry_b.entry_type(),
            &entry_b.address(),
            &test_provenances("sig"),
            &Some(chain_header_a.address()),
            &None,
            &None,
            &test_iso_8601(),
        );
        // c has same type as a
        let entry_c = test_entry();
        let chain_header_c = ChainHeader::new(
            &entry_c.entry_type(),
            &entry_c.address(),
            &test_provenances("sig"),
            &Some(chain_header_b.address()),
            &Some(chain_header_a.address()),
            &None,
            &test_iso_8601(),
        );

        for chain_header in vec![&chain_header_a, &chain_header_b, &chain_header_c] {
            let storage = chain_store.content_storage.clone();
            (*storage.write().unwrap())
                .add(chain_header)
                .expect("could not add header to cas");
        }

        let expected = vec![chain_header_c.clone(), chain_header_a.clone()];
        let mut found = vec![];
        for chain_header in
            chain_store.iter_type(&Some(chain_header_c.clone()), &chain_header_c.entry_type())
        {
            found.push(chain_header);
        }
        assert_eq!(expected, found);

        let expected = vec![chain_header_a.clone()];
        let mut found = vec![];
        for chain_header in
            chain_store.iter_type(&Some(chain_header_b.clone()), &chain_header_c.entry_type())
        {
            found.push(chain_header);
        }
        assert_eq!(expected, found);

        let expected = vec![chain_header_b.clone()];
        let mut found = vec![];
        for chain_header in
            chain_store.iter_type(&Some(chain_header_c.clone()), &chain_header_b.entry_type())
        {
            found.push(chain_header);
        }
        assert_eq!(expected, found);

        let expected = vec![chain_header_b.clone()];
        let mut found = vec![];
        for chain_header in
            chain_store.iter_type(&Some(chain_header_b.clone()), &chain_header_b.entry_type())
        {
            found.push(chain_header);
        }
        assert_eq!(expected, found);
    }

    #[test]
    /// show query() implementation
    fn query_test() {
        let chain_store = test_chain_store();

        // Two entries w/ the same EntryType "testEntryTypeB".
        let chain_header_a = test_chain_header();
        let entry = test_entry_b();
        let chain_header_b = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_provenances("sig-b"),
            &Some(chain_header_a.address()), // .link                (to previous entry)
            &None, // .link_same_type      (to previous entry of same type)
            &None,
            &test_iso_8601(),
        );
        let entry = test_entry_c();
        let chain_header_c = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_provenances("sig-c"),
            &Some(chain_header_b.address()),
            &Some(chain_header_b.address()),
            &None,
            &test_iso_8601(),
        );
        let entry = Entry::App(
            AppEntryType::from("another/something"),
            JsonString::from("Hello, World!"),
        );
        let chain_header_d = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_provenances("sig-d"),
            &Some(chain_header_c.address()),
            &None,
            &None,
            &test_iso_8601(),
        );
        let entry = Entry::App(
            AppEntryType::from("another/different"),
            JsonString::from("Kthxbye"),
        );
        let chain_header_e = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_provenances("sig-e"),
            &Some(chain_header_d.address()),
            &None,
            &None,
            &test_iso_8601(),
        );

        let storage = chain_store.content_storage.clone();
        (*storage.write().unwrap())
            .add(&chain_header_a)
            .expect("could not add header to cas");
        (*storage.write().unwrap())
            .add(&chain_header_b)
            .expect("could not add header to cas");
        (*storage.write().unwrap())
            .add(&chain_header_c)
            .expect("could not add header to cas");
        (*storage.write().unwrap())
            .add(&chain_header_d)
            .expect("could not add header to cas");
        (*storage.write().unwrap())
            .add(&chain_header_e)
            .expect("could not add header to cas");

        // First, lets see if we can find the EntryType "testEntryTypeB" Entries
        let found = match chain_store
            .query(
                &Some(chain_header_e.clone()),
                &vec![test_entry_type_b().to_string().as_ref()],
                ChainStoreQueryOptions::default(),
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };

        let expected = vec![
            chain_header_c.entry_address().clone(),
            chain_header_b.entry_address().clone(),
        ];
        assert_eq!(expected, found);

        // Then, limit to 1 at a time, starting from the 0'th match
        let found = match chain_store
            .query(
                &Some(chain_header_e.clone()),
                &vec![test_entry_type_b().to_string().as_ref()],
                ChainStoreQueryOptions {
                    start: 0,
                    limit: 1,
                    headers: false,
                },
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };
        let expected = vec![chain_header_c.entry_address().clone()];
        assert_eq!(expected, found);

        // Now query for all EntryTypes via entry_type == None
        let found = match chain_store
            .query(
                &Some(chain_header_e.clone()),
                &[],
                ChainStoreQueryOptions::default(),
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };
        let expected = vec![
            chain_header_e.entry_address().clone(),
            chain_header_d.entry_address().clone(),
            chain_header_c.entry_address().clone(),
            chain_header_b.entry_address().clone(),
            chain_header_a.entry_address().clone(),
        ];
        assert_eq!(expected, found);

        // Test Glob matching, namespacing.

        // Wildcard glob, all paths
        let found = match chain_store
            .query(
                &Some(chain_header_e.clone()),
                &vec!["**".to_string().as_ref()],
                ChainStoreQueryOptions::default(),
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };
        assert_eq!(expected, found);

        // Globbing plus some arbitrary EntryType names, thus matches everything again
        let found = match chain_store
            .query(
                &Some(chain_header_e.clone()),
                &vec!["another/*".to_string().as_ref(), "testEntryType*"],
                ChainStoreQueryOptions::default(),
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };
        assert_eq!(expected, found);

        // Just globbing
        let found = match chain_store
            .query(
                &Some(chain_header_e.clone()),
                &vec!["another/*".to_string().as_ref()],
                ChainStoreQueryOptions::default(),
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };
        let expected = vec![
            chain_header_e.entry_address().clone(),
            chain_header_d.entry_address().clone(),
        ];
        assert_eq!(expected, found);

        let entry = Entry::App(AppEntryType::from("ns/one"), JsonString::from("1"));
        let chain_header_f = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_provenances(),
            &vec![Signature::from("sig-f")],
            &Some(chain_header_e.address()),
            &None,
            &None,
            &test_iso_8601(),
        );
        let entry = Entry::App(AppEntryType::from("ns/sub/two"), JsonString::from("2"));
        let chain_header_g = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_provenances(),
            &vec![Signature::from("sig-g")],
            &Some(chain_header_f.address()),
            &None,
            &None,
            &test_iso_8601(),
        );
        let entry = Entry::App(AppEntryType::from("ns/sub/three"), JsonString::from("3"));
        let chain_header_h = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_provenances(),
            &vec![Signature::from("sig-g")],
            &Some(chain_header_g.address()),
            &None,
            &None,
            &test_iso_8601(),
        );
        (*storage.write().unwrap())
            .add(&chain_header_f)
            .expect("could not add header to cas");
        (*storage.write().unwrap())
            .add(&chain_header_g)
            .expect("could not add header to cas");
        (*storage.write().unwrap())
            .add(&chain_header_h)
            .expect("could not add header to cas");

        // Multiple complex globs.  The leading '**/' matches 0 or more leading .../ segments, so returns
        let found = match chain_store
            .query(
                &Some(chain_header_h.clone()),
                &vec!["another/*", "ns/**/t*"],
                ChainStoreQueryOptions::default(),
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };
        let expected = vec![
            chain_header_h.entry_address().clone(),
            chain_header_g.entry_address().clone(),
            chain_header_e.entry_address().clone(),
            chain_header_d.entry_address().clone(),
        ];
        assert_eq!(expected, found);

        // So, we should be able to find EntryType names by suffix at any depth
        let found = match chain_store
            .query(
                &Some(chain_header_h.clone()),
                &vec!["**/*{e,B}"],
                ChainStoreQueryOptions::default(),
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };
        let expected = vec![
            chain_header_h.entry_address().clone(), // .../three
            chain_header_f.entry_address().clone(), // .../one
            chain_header_c.entry_address().clone(), // testEntryTypeB
            chain_header_b.entry_address().clone(), // testEntryTypeB
            chain_header_a.entry_address().clone(), // testEntryType
        ];
        assert_eq!(expected, found);

        let entry = Entry::App(
            AppEntryType::from("%system_entry_type"),
            JsonString::from("System Entry"),
        );
        let chain_header_i = ChainHeader::new(
            &entry.entry_type(),
            &entry.address(),
            &test_provenances(),
            &vec![Signature::from("sig-i")],
            &Some(chain_header_h.address()),
            &None,
            &None,
            &test_iso_8601(),
        );
        (*storage.write().unwrap())
            .add(&chain_header_i)
            .expect("could not add header to cas");

        // Find EntryTypes which are/not System (start with '%'), and end in 'e'
        let found = match chain_store
            .query(
                &Some(chain_header_i.clone()),
                &vec!["[!%]*e"],
                ChainStoreQueryOptions::default(),
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };
        let expected = vec![
            chain_header_a.entry_address().clone(), // testEntryType
        ];
        assert_eq!(expected, found);

        let found = match chain_store
            .query(
                &Some(chain_header_i.clone()),
                &vec!["%*e"],
                ChainStoreQueryOptions::default(),
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };
        let expected = vec![
            chain_header_i.entry_address().clone(), // %system_entry_type
        ];
        assert_eq!(expected, found);

        // Including all namespaced EntryTypes
        let found = match chain_store
            .query(
                &Some(chain_header_i.clone()),
                &vec!["**/[!%]*e"],
                ChainStoreQueryOptions::default(),
            )
            .unwrap()
        {
            ChainStoreQueryResult::Addresses(addresses) => addresses,
            other => panic!("Unexpected query value {:?}", other),
        };
        let expected = vec![
            chain_header_h.entry_address().clone(), // .../three
            chain_header_f.entry_address().clone(), // .../one
            chain_header_a.entry_address().clone(), // testEntryType
        ];
        assert_eq!(expected, found);

        // Including all namespaced EntryTypes, getting ChainHeader
        let found = match chain_store
            .query(
                &Some(chain_header_i.clone()),
                &vec!["**/[!%]*e"],
                ChainStoreQueryOptions {
                    headers: true,
                    ..Default::default()
                },
            )
            .unwrap()
        {
            ChainStoreQueryResult::Headers(headers) => headers,
            other => panic!("Unexpected query value {:?}", other),
        };
        for (h, a) in found.iter().zip(expected.iter()) {
            assert_eq!(h.entry_address(), a);
        }
    }

    use globset::{Glob, GlobBuilder, GlobSetBuilder};

    #[test]
    /// show query() globbing implementation
    fn glob_query_test() {
        let glob = match Glob::new("*.rs") {
            Ok(pat) => pat.compile_matcher(),
            Err(_) => panic!("Couldn't craete new Glob"),
        };
        assert!(glob.is_match("foo.rs"));
        assert!(glob.is_match("foo/bar.rs")); // separators not specially handled
        assert!(!glob.is_match("Cargo.toml"));

        let glob = match GlobBuilder::new("*.rs").literal_separator(true).build() {
            Ok(pat) => pat.compile_matcher(),
            Err(_) => panic!("Couldn't craete new Glob"),
        };
        assert!(glob.is_match("foo.rs"));
        assert!(!glob.is_match("foo/bar.rs")); // separators now are special
        assert!(!glob.is_match("Cargo.toml"));

        let mut builder = GlobSetBuilder::new();
        // A GlobBuilder can be used to configure each glob's match semantics
        // independently.  Either using simple Glob::new (default semantics):
        builder.add(match Glob::new("*.rs") {
            Ok(pat) => pat,
            Err(_) => panic!("Couldn't craete new Glob"),
        });
        builder.add(match Glob::new("src/lib.rs") {
            Ok(pat) => pat,
            Err(_) => panic!("Couldn't craete new Glob"),
        });
        builder.add(match Glob::new("src/**/foo.rs") {
            Ok(pat) => pat,
            Err(_) => panic!("Couldn't craete new Glob"),
        });
        let set = match builder.build() {
            Ok(globset) => globset,
            Err(_) => panic!("Couldn't build GlobSetBuilder"),
        };
        assert_eq!(set.matches("src/bar/baz/foo.rs"), vec![0, 2]); // separators are not treated specially; '*' matches them

        // Or using GlobBuilder::new for specific modifiers on each pattern's behaviour
        let mut builder = GlobSetBuilder::new();
        builder.add(
            match GlobBuilder::new("*.rs").literal_separator(true).build() {
                Ok(pat) => pat,
                Err(_) => panic!("Couldn't craete new Glob"),
            },
        );
        builder.add(
            match GlobBuilder::new("src/lib.rs")
                .literal_separator(true)
                .build()
            {
                Ok(pat) => pat,
                Err(_) => panic!("Couldn't craete new Glob"),
            },
        );
        builder.add(
            match GlobBuilder::new("src/**/foo.rs")
                .literal_separator(true)
                .build()
            {
                Ok(pat) => pat,
                Err(_) => panic!("Couldn't craete new Glob"),
            },
        );
        builder.add(
            match GlobBuilder::new("**/foo.rs")
                .literal_separator(true)
                .build()
            {
                Ok(pat) => pat,
                Err(_) => panic!("Couldn't craete new Glob"),
            },
        );
        let set = match builder.build() {
            Ok(globset) => globset,
            Err(_) => panic!("Couldn't build GlobSetBuilder"),
        };
        assert_eq!(set.matches("src/bar/baz/foo.rs"), vec![2, 3]); // *.rs no longer matches, due to '/' separators
        assert_eq!(set.matches("foo.rs"), vec![0, 3]); // but, any number of leading '/' are matched by a '**/...'
    }

}
