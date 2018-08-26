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
use std::fs::create_dir_all;
use key::Key;

// folders actually... wish-it-was-tables
enum Table {
    Pairs,
    Metas,
}

// things that can be serialized and put in a file... wish-it-was-rows
trait Row: ToJson + Key {}
impl Row for Pair {}
impl Row for PairMeta {}

impl ToString for Table {
    fn to_string(&self) -> String {
        match self {
            Table::Pairs => "pairs",
            Table::Metas => "metas",
        }.to_string()
    }
}

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

    fn dir (&self, table: Table) -> String {
        let dir_string = format!("{}/{}", self.path, table.to_string());
        // @TODO be more efficient here
        // @TODO avoid unwrap
        create_dir_all(&dir_string).unwrap();
        dir_string
    }

    fn row_path(&self, table: Table, key: &str) -> String {
        let dir = self.dir(table);
        format!(
            "{}/{}.json",
            dir,
            key,
        )
    }

    fn upsert<R: Row>(&self, table: Table, row: &R) -> Result<(), HolochainError> {
        match fs::write(
            self.row_path(table, &row.key()),
            row.to_json().unwrap(),
        ) {
            Err(e) => Err(HolochainError::from(e)),
            _ => Ok(()),
        }
    }

    /// Returns a JSON string option for the given key in the given table
    fn lookup(&self, table: Table, key: &str) -> Result<Option<String>, HolochainError> {
        let path_string = self.row_path(table, key);
        if Path::new(&path_string).is_file() {
            match fs::read_to_string(path_string) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(HolochainError::from(e)),
            }
        }
        else {
            Ok(None)
        }
    }

}

impl HashTable for FileTable {

    fn commit_pair(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        self.upsert(Table::Pairs, pair)
    }

    fn pair(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        Ok(
            self
                .lookup(Table::Pairs, key)?
                // @TODO don't unwrap here
                .and_then(|s| Some(Pair::from_json(&s).unwrap()))
        )
    }

    fn assert_pair_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
        self.upsert(Table::Metas, meta)
    }

    fn pair_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        Ok(
            self
                .lookup(Table::Metas, key)?
                .and_then(|s| Some(PairMeta::from_json(&s).unwrap()))
        )
    }

    fn all_metas_for_pair(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let mut metas = Vec::new();

        // this is a brute force approach that involves reading and parsing every file
        // big meta data should be backed by something indexed like sqlite
        for meta in WalkDir::new(self.dir(Table::Metas)) {
            let meta = meta.unwrap();
            let path = meta.path();
            let key = path.file_stem();
            match key {
                Some(k) => {
                    match self.pair_meta(&k.to_string_lossy())? {
                        Some(pair_meta) => {
                            if pair_meta.pair() == pair.key() {
                                metas.push(pair_meta);
                            }
                        }
                        None => {},
                    }
                },
                None => {},
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

    use tempfile::tempdir;
    use tempfile::TempDir;
    use hash_table::HashTable;
    use hash_table::test_util::test_pair_round_trip;
    use hash_table::test_util::test_modify_pair;
    use hash_table::test_util::test_retract_pair;
    use hash_table::test_util::test_meta_round_trip;
    use hash_table::test_util::test_all_metas_for_pair;

    use hash_table::{
        file::FileTable,
    };

    /// returns a new FileTable for testing and the TempDir created for it
    /// the fs directory associated with TempDir will be deleted when the TempDir goes out of scope
    /// @see https://docs.rs/tempfile/3.0.3/tempfile/struct.TempDir.html
    pub fn test_table() -> (FileTable, TempDir) {
        let dir = tempdir().unwrap();
        (
            FileTable::new(dir.path().to_str().unwrap()),
            dir,
        )
    }

    #[test]
    /// smoke test
    fn new() {
        let (_table, _dir) = test_table();
    }

    #[test]
    /// tests for ht.setup()
    fn setup() {
        let (mut table, _dir) = test_table();
        assert_eq!(Ok(()), table.setup());
    }

    #[test]
    /// tests for ht.teardown()
    fn teardown() {
        let (mut table, _dir) = test_table();
        assert_eq!(Ok(()), table.teardown());
    }

    #[test]
    /// Pairs can round trip through table.commit() and table.get()
    fn pair_round_trip() {
        let (mut table, _dir) = test_table();
        test_pair_round_trip(&mut table);
    }

    #[test]
    /// Pairs can be modified through table.modify()
    fn modify_pair() {
        let (mut table, _dir) = test_table();
        test_modify_pair(&mut table);
    }

    #[test]
    /// Pairs can be retracted through table.retract()
    fn retract_pair() {
        let (mut table, _dir) = test_table();
        test_retract_pair(&mut table);
    }

    #[test]
    /// PairMeta can round trip through table.assert_meta() and table.get_meta()
    fn meta_round_trip() {
        let (mut table, _dir) = test_table();
        test_meta_round_trip(&mut table);
    }

    #[test]
    /// all PairMeta for a Pair can be retrieved with get_pair_meta
    fn all_metas_for_pair() {
        let (mut table, _dir) = test_table();
        test_all_metas_for_pair(&mut table);
    }
}
