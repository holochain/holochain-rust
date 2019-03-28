use glob::glob;
use holochain_core_types::{
    cas::content::AddressableContent,
    eav::{
        Attribute, EavFilter, EaviQuery, Entity, EntityAttributeValueIndex,
        EntityAttributeValueStorage, Value,
    },
    error::{HcResult, HolochainError},
    json::JsonString,
};
use std::{
    collections::BTreeSet,
    convert::{TryFrom, TryInto},
    fs::{create_dir_all, File, OpenOptions},
    io::prelude::*,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};
use uuid::Uuid;

const ENTITY_DIR: &str = "e";
const ATTRIBUTE_DIR: &str = "a";
const VALUE_DIR: &str = "v";

#[derive(Clone, Debug)]
pub struct EavFileStorage {
    dir_path: PathBuf,
    id: Uuid,

    lock: Arc<RwLock<()>>,
}

impl PartialEq for EavFileStorage {
    fn eq(&self, other: &EavFileStorage) -> bool {
        self.id == other.id
    }
}

#[warn(unused_must_use)]
pub fn read_eav(parent_path: PathBuf) -> HcResult<Vec<String>> {
    //glob all  files
    let full_path = parent_path.join("*").join("*.txt");

    let paths = glob(full_path.to_str().unwrap())
        .map_err(|_| HolochainError::ErrorGeneric("Could not get form path".to_string()))?;

    // let path_result = paths.last().ok_or(HolochainError::ErrorGeneric("Could not get form path".to_string()))?;
    let (eav, error): (BTreeSet<_>, BTreeSet<_>) = paths
        .map(|path| {
            let path_buf: PathBuf = path.unwrap_or_default();
            OpenOptions::new()
                .read(true)
                .open(path_buf.clone())
                .map(|mut file| {
                    let mut content = String::new();
                    file.read_to_string(&mut content)
                        .map(|_| Ok(content))
                        .unwrap_or_else(|_| {
                            Err(HolochainError::ErrorGeneric(
                                "Could not read from string".to_string(),
                            ))
                        })
                })
                .unwrap_or_else(|_| {
                    Err(HolochainError::ErrorGeneric(
                        "Could not read from string".to_string(),
                    ))
                })
        })
        .partition(Result::is_ok);
    if !error.is_empty() {
        Err(HolochainError::ErrorGeneric(
            "Could not read from string".to_string(),
        ))
    } else {
        Ok(eav.iter().cloned().map(|s| s.unwrap_or_default()).collect())
    }
}

impl EavFileStorage {
    pub fn new<P: AsRef<Path>>(dir_path: P) -> HcResult<EavFileStorage> {
        let dir_path = dir_path.as_ref().into();

        Ok(EavFileStorage {
            dir_path,
            id: Uuid::new_v4(),
            lock: Arc::new(RwLock::new(())),
        })
    }

    fn write_to_file(
        &self,
        subscript: String,
        eav: &EntityAttributeValueIndex,
    ) -> Result<(), HolochainError> {
        let address: String = match &*subscript {
            ENTITY_DIR => eav.entity().to_string(),
            ATTRIBUTE_DIR => eav.attribute().to_string(),
            VALUE_DIR => eav.value().to_string(),
            _ => String::new(),
        };

        let path = self
            .dir_path
            .join(&subscript)
            .join(&address)
            .join(&eav.index().to_string());

        create_dir_all(&path)?;

        let address_path = path.join(eav.address().to_string());

        let full_path = address_path.with_extension("txt");

        let mut file = File::create(full_path)?;
        writeln!(file, "{}", eav.content())?;
        Ok(())
    }

    fn read_from_dir<T>(
        &self,
        subscript: String,
        eav_filter: &EavFilter<T>,
    ) -> HcResult<BTreeSet<String>>
    where
        T: Eq + ToString + TryFrom<String>,
    {
        let path = self.dir_path.join(&subscript);

        if path.exists() {
            let full_path = path.join("*");

            let paths = glob(full_path.to_str().unwrap())
                .map_err(|_| HolochainError::ErrorGeneric("Could not get form path".to_string()))?;

            let (paths, errors): (Vec<_>, Vec<_>) = paths.partition(Result::is_ok);
            let eavs = paths
                .into_iter()
                .map(|p| p.unwrap())
                .filter(|pathbuf| {
                    pathbuf
                        .iter()
                        .last()
                        .and_then(|v| {
                            let v = v.to_string_lossy();
                            v.to_string()
                                .try_into()
                                .map_err(|_| println!("warn/eav: invalid EAV string: {}", v))
                                .ok()
                                .map(|val| eav_filter.check(val))
                        })
                        .unwrap_or_default()
                })
                .map(|pathbuf| read_eav(pathbuf.clone()));
            if !errors.is_empty() {
                Err(HolochainError::ErrorGeneric(
                    "Could not read eavs from directory".to_string(),
                ))
            } else {
                Ok(eavs.filter_map(|s| s.ok()).flatten().collect())
            }
        } else {
            Ok(BTreeSet::new())
        }
    }
}

impl EntityAttributeValueStorage for EavFileStorage {
    fn add_eavi(
        &mut self,
        eav: &EntityAttributeValueIndex,
    ) -> Result<Option<EntityAttributeValueIndex>, HolochainError> {
        let _guard = self.lock.write()?;
        let wild_card = Path::new("*");
        //create glob path to query file system parentdir/*/*/*/{address}.txt
        let text_file_path = Path::new(eav.address().to_string()).with_extension("txt");
        let path = self
            .dir_path
            .join(wild_card)
            .join(ENTITY_DIR)
            .join(&*eav.index().to_string())
            .join(&text_file_path);

        //if next exists create a new eav with a different index
        let eav = if path.exists() {
            EntityAttributeValueIndex::new(&eav.entity(), &eav.attribute(), &eav.value())?
        } else {
            eav.clone()
        };

        self.write_to_file(ENTITY_DIR.to_string(), &eav)
            .and_then(|_| self.write_to_file(ATTRIBUTE_DIR.to_string(), &eav))
            .and_then(|_| self.write_to_file(VALUE_DIR.to_string(), &eav))?;
        Ok(Some(eav.clone()))
    }

    fn fetch_eavi(
        &self,
        query: &EaviQuery,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError> {
        let _guard = self.lock.read()?;

        let entity_set = self.read_from_dir::<Entity>(ENTITY_DIR.to_string(), query.entity())?;
        let attribute_set = self
            .read_from_dir::<Attribute>(ATTRIBUTE_DIR.to_string(), query.attribute())
            .clone()?;
        let value_set = self.read_from_dir::<Value>(VALUE_DIR.to_string(), query.value())?;

        let attribute_value_inter: BTreeSet<String> = value_set
            .intersection(&attribute_set.clone())
            .cloned()
            .collect();
        let entity_attribute_value_inter: BTreeSet<String> = attribute_value_inter
            .intersection(&entity_set)
            .cloned()
            .collect();
        let total = entity_attribute_value_inter.len();
        let eavis: BTreeSet<_> = entity_attribute_value_inter
            .clone()
            .into_iter()
            .filter_map(|content| {
                EntityAttributeValueIndex::try_from_content(&JsonString::from(content)).ok()
            })
            .collect();
        if eavis.len() < total {
            // not all EAVs were converted
            Err(HolochainError::ErrorGeneric(
                "Error Converting EAVs".to_string(),
            ))
        } else {
            // Build a query that only filters by Index, to be run on the collection that was already filtered
            // by the above code
            let index_query = EaviQuery::new(
                Default::default(),
                Default::default(),
                Default::default(),
                query.index().clone(),
            );
            let it = eavis.iter().cloned();
            let results = index_query.run(it);
            Ok(results)
        }
    }
}

#[cfg(test)]
pub mod tests {
    extern crate tempfile;
    use self::tempfile::tempdir;
    use eav::file::EavFileStorage;
    #[cfg(any(not(windows), feature = "broken-tests"))]
    use holochain_core_types::eav::Attribute;
    use holochain_core_types::{
        cas::{
            content::{AddressableContent, ExampleAddressableContent},
            storage::EavTestSuite,
        },
        json::RawString,
    };

    #[test]
    fn file_eav_round_trip() {
        let temp = tempdir().expect("test was supposed to create temp dir");

        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let entity_content =
            ExampleAddressableContent::try_from_content(&RawString::from("foo").into()).unwrap();
        let attribute = "favourite-color".to_string();
        let value_content =
            ExampleAddressableContent::try_from_content(&RawString::from("blue").into()).unwrap();

        EavTestSuite::test_round_trip(
            EavFileStorage::new(temp_path).unwrap(),
            entity_content,
            attribute,
            value_content,
        )
    }

    #[test]
    // breaks on av https://ci.appveyor.com/project/thedavidmeister/holochain-rust/builds/23356009
    #[cfg(any(not(windows), feature = "broken-tests"))]
    fn file_eav_one_to_many() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_one_to_many::<ExampleAddressableContent, EavFileStorage>(eav_storage);
    }

    #[test]
    // breaks on av https://ci.appveyor.com/project/thedavidmeister/holochain-rust/builds/23356009
    #[cfg(any(not(windows), feature = "broken-tests"))]
    fn file_eav_many_to_one() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_many_to_one::<ExampleAddressableContent, EavFileStorage>(eav_storage);
    }

    #[test]
    // breaks on av https://ci.appveyor.com/project/thedavidmeister/holochain-rust/builds/23356009
    #[cfg(any(not(windows), feature = "broken-tests"))]
    fn file_eav_range() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_range::<ExampleAddressableContent, EavFileStorage>(eav_storage);
    }

    #[test]
    // breaks on av https://ci.appveyor.com/project/thedavidmeister/holochain-rust/builds/23356009
    #[cfg(any(not(windows), feature = "broken-tests"))]
    fn file_eav_prefixes() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_multiple_attributes::<ExampleAddressableContent, EavFileStorage>(
            eav_storage,
            vec!["a_", "b_", "c_", "d_"]
                .into_iter()
                .map(|p| Attribute::LinkTag(p.to_string() + "one_to_many"))
                .collect(),
        );
    }

}
