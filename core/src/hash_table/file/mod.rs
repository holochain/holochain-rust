use std::path::Path;

use error::HolochainError;
use std::fs;

use hash_table::{pair::Pair, pair_meta::PairMeta, HashTable};
use json::{FromJson, ToJson};
use key::Key;
use std::fs::create_dir_all;
use walkdir::WalkDir;

// folders actually... wish-it-was-tables
#[derive(Debug, Clone)]
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

#[derive(Serialize, Debug, PartialEq, Clone)]
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
            Ok(FileTable {
                path: match canonical.to_str() {
                    Some(p) => p.to_string(),
                    None => {
                        return Err(HolochainError::IoError(
                            "could not convert path to string".to_string(),
                        ));
                    }
                },
            })
        } else {
            Err(HolochainError::IoError(
                "path is not a directory or permissions don't allow access".to_string(),
            ))
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

    fn metas_for_pair(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let mut metas = Vec::new();

        // this is a brute force approach that involves reading and parsing every file
        // big meta data should be backed by something indexed like sqlite
        for meta in WalkDir::new(self.dir(Table::Metas)?) {
            let meta = meta?;
            let path = meta.path();
            if let Some(stem) = path.file_stem() {
                if let Some(key) = stem.to_str() {
                    if let Some(pair_meta) = self.pair_meta(&key)? {
                        if pair_meta.pair_hash() == pair.key() {
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
    use super::Table;
    use hash_table::{file::FileTable, test_util::standard_suite};
    use tempfile::{tempdir, TempDir};
    use regex::Regex;
    use serde_json;
    use error::HolochainError;
    use json::ToJson;
    use key::Key;
    use hash_table::file::Row;

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
    fn test_standard_suite() {
        let (mut table, _dir) = test_table();
        standard_suite(&mut table);
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
        let (table, _dir) = test_table();

        let re = |s| {
            Regex::new(&format!(r".*\.tmp.*/{}", s)).expect("failed to build regex")
        };

        for (s, t) in vec![("pairs", Table::Pairs), ("metas", Table::Metas)] {
            assert!(
                re(s)
                    .is_match(
                        &table
                            .dir(t.clone())
                            .expect(&format!("could not get dir for {:?}", t))
                    )
            );
        }
    }

    #[test]
    /// row_path returns a sensible string for a Table enum and key
    fn test_row_path() {
        let (table, _dir) = test_table();

        let re = |s, k| {
            Regex::new(&format!(r".*\.tmp.*/{}/{}\.json", s, k)).expect("failed to build regex")
        };

        for (s, t) in vec![("pairs", Table::Pairs), ("metas", Table::Metas)] {
            for k in vec!["foo", "bar"] {
                assert!(
                    re(s, k)
                        .is_match(
                            &table
                                .row_path(t.clone(), k.clone())
                                .expect(&format!("could not get row path for {:?} in {:?}", k, t))
                        )
                );
            }
        }
    }

    #[test]
    /// data can round trip through upsert/lookup
    fn test_data_round_trip() {
        #[derive(Serialize)]
        struct SomeData {data: String}

        impl ToJson for SomeData {
            fn to_json(&self) -> Result<String, HolochainError> {
                Ok(serde_json::to_string(&self)?)
            }
        }

        impl Key for SomeData {
            fn key(&self) -> String {
                "bar".to_string()
            }
        }

        impl Row for SomeData {}

        let data = SomeData {data: "foo".to_string()};
        let s = data.to_json().expect("could not serialize data");

        let (table, _dir) = test_table();

        table.upsert(Table::Pairs, &data).expect("could not upsert data");

        assert_eq!(
            Some(s),
            table.lookup(Table::Pairs, &data.key()).expect("could not lookup data"),
        );
    }

}
