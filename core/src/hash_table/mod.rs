pub mod status;
pub mod entry;
pub mod header;
pub mod pair;
pub mod pair_meta;
pub mod memory;

use error::HolochainError;
use hash_table::pair::Pair;
use hash_table::pair_meta::PairMeta;

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub trait HashTableClone {
    fn clone_box(&self) -> Box<HashTable>;
}

impl<HT> HashTableClone for HT
    where
        HT: 'static + HashTable + Clone,
        {
            fn clone_box(&self) -> Box<HashTable> {
                Box::new(self.clone())
            }
}

impl Clone for Box<HashTable> {
    fn clone(&self) -> Box<HashTable> {
        self.clone_box()
    }
}

pub trait HashTable: HashTableClone + Send + Sync {

    // fn box_clone (&self) -> Box<HashTable>;
    // fn clone (&self) -> HashTable;

    // state changes
    fn open (&mut self) -> Result<(), HolochainError>;
    fn close (&mut self) -> Result<(), HolochainError>;

    // crud
    fn commit (&mut self, pair: &Pair) -> Result<(), HolochainError>;
    fn get (&self, key: &str) -> Result<Option<Pair>, HolochainError>;
    fn modify (&mut self, old_pair: &Pair, new_pair: &Pair) -> Result<(), HolochainError>;
    fn retract (&mut self, pair: &Pair) -> Result<(), HolochainError>;

    // meta
    fn assert_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError>;
    fn get_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError>;

    // query
    // fn query (&self, query: &Query) -> Result<std::collections::HashSet, HolochainError>;

}

// clone_trait_object!(HashTable);

// impl<'a> HashTable for &'a HashTable {
//
//     fn open (&mut self) -> Result<(), HolochainError> {
//         self.open()
//     }
//
//     fn close (&mut self) -> Result<(), HolochainError> {
//         self.close()
//     }
//
//     // crud
//     fn commit (&mut self, pair: &Pair) -> Result<(), HolochainError> {
//         self.commit(pair)
//     }
//     fn get (&self, key: &str) -> Result<Option<Pair>, HolochainError> {
//         self.get(key)
//     }
//     fn modify (&mut self, old_pair: &Pair, new_pair: &Pair) -> Result<(), HolochainError> {
//         self.modify(old_pair, new_pair)
//     }
//     fn retract (&mut self, pair: &Pair) -> Result<(), HolochainError> {
//         self.retract(pair)
//     }
//
//     // meta
//     fn assert_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
//         self.assert_meta(meta)
//     }
// }

// impl HashTable for Box<HashTable> {
//     fn box_clone(&self) -> Box<HashTable> {
//         self.clone()
//     }
//
//     fn open (&mut self) -> Result<(), HolochainError> {
//         self.open()
//     }
//     fn close (&mut self) -> Result<(), HolochainError> {
//         self.close()
//     }
//
//     // crud
//     fn commit (&mut self, pair: &Pair) -> Result<(), HolochainError> {
//         self.commit(pair)
//     }
//     fn get (&self, key: &str) -> Result<Option<Pair>, HolochainError> {
//         self.get(key)
//     }
//     fn modify (&mut self, old_pair: &Pair, new_pair: &Pair) -> Result<(), HolochainError> {
//         self.modify(old_pair, new_pair)
//     }
//     fn retract (&mut self, pair: &Pair) -> Result<(), HolochainError> {
//         self.retract(pair)
//     }
//
//     fn assert_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
//         self.assert_meta(meta)
//     }
// }

// https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
// impl Clone for Box<HashTable> {
//     fn clone(&self) -> Box<HashTable> {
//         self.box_clone()
//     }
// }

// impl PartialEq for Box<HashTable> {
//     fn eq(&self, other: &Box<HashTable>) -> bool {
//         self == other
//     }
// }
