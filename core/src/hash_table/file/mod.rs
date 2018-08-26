use std::path::Path;

use error::HolochainError;
use std::fs;

use hash_table::{pair::Pair, pair_meta::PairMeta, HashTable};
use json::{FromJson, ToJson};
use key::Key;
use std::fs::create_dir_all;
use walkdir::WalkDir;

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
    /// attempts to build a new FileTable
    /// can fail if the given path can't be resolved to a directory on the filesystem
    /// can fail if permissions don't allow access to the directory on the filesystem
    pub fn new(path: &str) -> Result<FileTable, HolochainError> {
        let canonical = Path::new(path).canonicalize()?;
        if canonical.is_dir() {
            Ok(
                FileTable {
                    path: match canonical.to_str() {
                        Some(p) => p.to_string(),
                        None => { return Err(HolochainError::IoError("could not convert path to string".to_string())); },
                    }
                }
            )
        }
        else {
            Err(HolochainError::IoError("path is not a directory or permissions don't allow access".to_string()))
        }
    }

    /// given a Table enum, ensure that the correct sub-directory exists and return the string path
    fn dir(&self, table: Table) -> Result<String, HolochainError> {
        let dir_string = format!("{}/{}", self.path, table.to_string());
        // @TODO be more efficient here
        // @see https://github.com/holochain/holochain-rust/issues/248
        create_dir_all(&dir_string)?;
        Ok(dir_string)
    }

    fn row_path(&self, table: Table, key: &str) -> Result<String, HolochainError> {
        let dir = self.dir(table)?;
        Ok(format!("{}/{}.json", dir, key))
    }

    fn upsert<R: Row>(&self, table: Table, row: &R) -> Result<(), HolochainError> {
        match fs::write(self.row_path(table, &row.key())?, row.to_json()?) {
            Err(e) => Err(HolochainError::from(e)),
            _ => Ok(()),
        }
    }

    /// Returns a JSON string option for the given key in the given table
    fn lookup(&self, table: Table, key: &str) -> Result<Option<String>, HolochainError> {
        let path_string = self.row_path(table, key)?;
        if Path::new(&path_string).is_file() {
            Ok(Some(fs::read_to_string(path_string)?))
        } else {
            Ok(None)
        }
    }
}

impl HashTable for FileTable {
    fn commit_pair(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        self.upsert(Table::Pairs, pair)
    }

    fn pair(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        match self.lookup(Table::Pairs, key)? {
            Some(json) => Ok(Some(Pair::from_json(&json)?)),
            None => Ok(None),
        }
    }

    fn assert_pair_meta(&mut self, meta: &PairMeta) -> Result<(), HolochainError> {
        self.upsert(Table::Metas, meta)
    }

    fn pair_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        match self.lookup(Table::Metas, key)? {
            Some(json) => Ok(Some(PairMeta::from_json(&json)?)),
            None => Ok(None),
        }
    }

    fn all_metas_for_pair(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let mut metas = Vec::new();

        // this is a brute force approach that involves reading and parsing every file
        // big meta data should be backed by something indexed like sqlite
        for meta in WalkDir::new(self.dir(Table::Metas)?) {
            let meta = meta?;
            let path = meta.path();
            if let Some(stem) = path.file_stem() {
                if let Some(key) = stem.to_str() {
                    if let Some(pair_meta) = self.pair_meta(&key)? {
                        if pair_meta.pair() == pair.key() {
                            metas.push(pair_meta);
                        }
                    }
                }
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

    use hash_table::{
        test_util::{
            test_all_metas_for_pair, test_meta_round_trip, test_modify_pair, test_pair_round_trip,
            test_retract_pair,
        },
        HashTable,
    };
    use tempfile::{tempdir, TempDir};

    use hash_table::file::FileTable;

    /// returns a new FileTable for testing and the TempDir created for it
    /// the fs directory associated with TempDir will be deleted when the TempDir goes out of scope
    /// @see https://docs.rs/tempfile/3.0.3/tempfile/struct.TempDir.html
    pub fn test_table() -> (FileTable, TempDir) {
        let dir = tempdir().unwrap();
        (FileTable::new(dir.path().to_str().unwrap()).unwrap(), dir)
    }

    #[test]
    /// smoke test
    fn new() {
        let (_table, _dir) = test_table();
    }

    #[test]
    /// a missing directory gives an error result
    fn new_error_missing_dir() {
        let result = FileTable::new("foo bar missing dir");
        assert!(result.is_err());
    }

    #[test]
    /// dir returns a sensible string for every Table enum variant
    fn test_dir() {
        // @TODO
    }

    #[test]
    /// row_path returns a sensible string for a Table enum and key
    fn test_row_path() {
        // @TODO
    }

    #[test]
    /// rows can round trip through upsert/lookup
    fn test_row_round_trip() {
        // @TODO
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
