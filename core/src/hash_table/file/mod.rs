use std::path::Path;

use error::HolochainError;
use std::fs;

use hash_table::{
    pair::Pair,
    pair_meta::PairMeta,
    HashTable,
};
use json::ToJson;
use walkdir::WalkDir;
use json::FromJson;

#[derive(Serialize, Debug, PartialEq)]
pub struct FileTable {
    path: String,
}

impl FileTable {
    pub fn new(path: &str) -> FileTable {
        // @TODO unwrap
        let canonical = Path::new(path).canonicalize().unwrap();
        FileTable {
            // @TODO is lossy string right?
            path: canonical.to_string_lossy().to_string(),
        }
    }

    fn pairs_dir(&self) -> String {
        format!("{}/pairs", self.path)
    }

    fn key_path(&self, dir: &str, key: &str) -> String {
        format!(
            "{}/{}.json",
            dir,
            key,
        )
    }

    fn pair_key_path(&self, key: &str) -> String {
        self.key_path(&self.pairs_dir(), key)
    }

    fn metas_dir(&self) -> String {
        format!("{}/meta", self.path)
    }

    fn meta_key_path(&self, key: &str) -> String {
        self.key_path(&self.metas_dir(), key)
    }

}

impl HashTable for FileTable {

    fn commit_pair(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        fs::write(
            self.pair_key_path(&pair.key()),
            pair.to_json().unwrap(),
        );
        // @TODO real result
        Ok(())
    }

    fn pair(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        // @TODO real result
        Ok(
            Some(
                Pair::from_json(
                    &fs::read_to_string(self.pair_key_path(key)).unwrap()
                ).unwrap()
            )
        )
        // Ok(self.pairs.get(key).cloned())
    }

    fn assert_pair_meta(&mut self, meta: PairMeta) -> Result<(), HolochainError> {
        fs::write(
            self.meta_key_path(&meta.key()),
            meta.to_json().unwrap(),
        );
        // TODO real result
        Ok(())
    }

    fn pair_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        // @TODO real result
        Ok(
            Some(
                PairMeta::from_json(
                    &fs::read_to_string(self.meta_key_path(key)).unwrap()
                ).unwrap()
            )
        )
    }

    fn all_metas_for_pair(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let mut metas = Vec::new();

        // this is a brute force approach that involves reading and parsing every file
        // big meta data should be backed by something indexable like sqlite
        for meta in WalkDir::new(self.metas_dir()) {
            let path = meta?.path();
            let meta_parsed = PairMeta::from_json(
                &fs::read_to_string(
                    &path.to_string_lossy().to_string()
                )?
            )?;
            if meta_parsed.pair() == pair.key() {
                metas.push(meta_parsed);
            }
        }

        // @TODO should this be sorted at all at this point?
        // @see https://github.com/holochain/holochain-rust/issues/144
        metas.sort();
        Ok(metas)
    }
}

#[cfg(test)]
pub mod tests {

    use agent::keys::tests::test_keys;
    use hash_table::{
        file::FileTable,
        pair::tests::{test_pair, test_pair_a, test_pair_b},
        pair_meta::{
            tests::{test_pair_meta, test_pair_meta_a, test_pair_meta_b},
            PairMeta,
        },
        status::{CRUDStatus, LINK_NAME, STATUS_NAME},
        HashTable,
    };

    // @TODO use a temp dir
    pub fn test_table_path() -> String {
        "/tmp/hc_test".to_string()
    }

    pub fn test_table() -> MemTable {
        FileTable::new(&test_table_path())
    }

    #[test]
    /// smoke test
    fn new() {
        test_table();
    }
    //
    // #[test]
    // /// tests for ht.setup()
    // fn setup() {
    //     let mut ht = test_table();
    //     assert_eq!(Ok(()), ht.setup());
    // }
    //
    // #[test]
    // /// tests for ht.teardown()
    // fn teardown() {
    //     let mut ht = test_table();
    //     assert_eq!(Ok(()), ht.teardown());
    // }
    //
    // #[test]
    // /// Pairs can round trip through table.commit() and table.get()
    // fn pair_round_trip() {
    //     let mut ht = test_table();
    //     let p = test_pair();
    //     ht.commit(&p).expect("should be able to commit valid pair");
    //     assert_eq!(ht.get(&p.key()), Ok(Some(p)));
    // }
    //
    // #[test]
    // /// Pairs can be modified through table.modify()
    // fn modify() {
    //     let mut ht = test_table();
    //     let p1 = test_pair_a();
    //     let p2 = test_pair_b();
    //
    //     ht.commit(&p1).expect("should be able to commit valid pair");
    //     ht.modify(&test_keys(), &p1, &p2)
    //         .expect("should be able to edit with valid pair");
    //
    //     assert_eq!(
    //         vec![
    //             PairMeta::new(&test_keys(), &p1, LINK_NAME, &p2.key()),
    //             PairMeta::new(
    //                 &test_keys(),
    //                 &p1,
    //                 STATUS_NAME,
    //                 &CRUDStatus::MODIFIED.bits().to_string(),
    //             ),
    //         ],
    //         ht.get_pair_meta(&p1)
    //             .expect("getting the metadata on a pair shouldn't fail")
    //     );
    //
    //     let empty_vec: Vec<PairMeta> = Vec::new();
    //     assert_eq!(
    //         empty_vec,
    //         ht.get_pair_meta(&p2)
    //             .expect("getting the metadata on a pair shouldn't fail")
    //     );
    // }
    //
    // #[test]
    // /// Pairs can be retracted through table.retract()
    // fn retract() {
    //     let mut ht = test_table();
    //     let p = test_pair();
    //     let empty_vec: Vec<PairMeta> = Vec::new();
    //
    //     ht.commit(&p).expect("should be able to commit valid pair");
    //     assert_eq!(
    //         empty_vec,
    //         ht.get_pair_meta(&p)
    //             .expect("getting the metadata on a pair shouldn't fail")
    //     );
    //
    //     ht.retract(&test_keys(), &p)
    //         .expect("should be able to retract");
    //     assert_eq!(
    //         vec![PairMeta::new(
    //             &test_keys(),
    //             &p,
    //             STATUS_NAME,
    //             &CRUDStatus::DELETED.bits().to_string(),
    //         )],
    //         ht.get_pair_meta(&p)
    //             .expect("getting the metadata on a pair shouldn't fail"),
    //     );
    // }
    //
    // #[test]
    // /// PairMeta can round trip through table.assert_meta() and table.get_meta()
    // fn meta_round_trip() {
    //     let mut ht = test_table();
    //     let m = test_pair_meta();
    //
    //     assert_eq!(
    //         None,
    //         ht.get_meta(&m.key())
    //             .expect("getting the metadata on a pair shouldn't fail")
    //     );
    //
    //     ht.assert_meta(m.clone())
    //         .expect("asserting metadata shouldn't fail");
    //     assert_eq!(
    //         Some(&m),
    //         ht.get_meta(&m.key())
    //             .expect("getting the metadata on a pair shouldn't fail")
    //             .as_ref()
    //     );
    // }
    //
    // #[test]
    // /// all PairMeta for a Pair can be retrieved with get_pair_meta
    // fn get_pair_meta() {
    //     let mut ht = test_table();
    //     let p = test_pair();
    //     let m1 = test_pair_meta_a();
    //     let m2 = test_pair_meta_b();
    //     let empty_vec: Vec<PairMeta> = Vec::new();
    //
    //     assert_eq!(
    //         empty_vec,
    //         ht.get_pair_meta(&p)
    //             .expect("getting the metadata on a pair shouldn't fail")
    //     );
    //
    //     ht.assert_meta(m1.clone())
    //         .expect("asserting metadata shouldn't fail");
    //     assert_eq!(
    //         vec![m1.clone()],
    //         ht.get_pair_meta(&p)
    //             .expect("getting the metadata on a pair shouldn't fail")
    //     );
    //
    //     ht.assert_meta(m2.clone())
    //         .expect("asserting metadata shouldn't fail");
    //     assert_eq!(
    //         vec![m2, m1],
    //         ht.get_pair_meta(&p)
    //             .expect("getting the metadata on a pair shouldn't fail")
    //     );
    // }
}
