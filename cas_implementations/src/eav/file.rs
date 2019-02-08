use glob::glob;
use holochain_core_types::{
    cas::content::AddressableContent,
    eav::{
        get_latest, increment_key_till_no_collision, Attribute, Entity, EntityAttributeValueIndex,
        EntityAttributeValueStorage, IndexQuery, Value,
    },
    error::{HcResult, HolochainError},
    json::JsonString,
};
use std::{
    collections::BTreeSet,
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
            ATTRIBUTE_DIR => eav.attribute(),
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
        eav_constraint: Option<T>,
    ) -> HcResult<BTreeSet<String>>
    where
        T: ToString,
    {
        let address = eav_constraint
            .map(|e| e.to_string())
            .unwrap_or("*".to_string());

        let path = self.dir_path.join(&subscript);

        if path.exists() {
            let full_path = path.join(&address);

            let paths = glob(full_path.to_str().unwrap())
                .map_err(|_| HolochainError::ErrorGeneric("Could not get form path".to_string()))?;

            let (eavs, errors): (Vec<_>, Vec<_>) = paths
                .map(|path_val| {
                    path_val.map(|walk| read_eav(walk.clone())).unwrap_or(Err(
                        HolochainError::ErrorGeneric(
                            "Could not read eavs from directory".to_string(),
                        ),
                    ))
                })
                .partition(Result::is_ok);
            if errors.len() > 0 {
                Err(HolochainError::ErrorGeneric(
                    "Could not read eavs from directory".to_string(),
                ))
            } else {
                let mut ordmap: BTreeSet<String> = BTreeSet::new();
                eavs.iter().for_each(|s| {
                    s.clone().unwrap_or(Vec::new()).iter().for_each(|value| {
                        ordmap.insert(value.clone());
                    })
                });

                Ok(ordmap)
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
        let fetched = self.fetch_eavi(
            Some(eav.entity()),
            Some(eav.attribute()),
            Some(eav.value()),
            IndexQuery::default(),
        )?;
        let _guard = self.lock.write()?;
        create_dir_all(self.dir_path.clone())?;
        let new_eav = increment_key_till_no_collision(eav.clone(), fetched.clone())?;
        self.write_to_file(ENTITY_DIR.to_string(), &new_eav)
            .and_then(|_| self.write_to_file(ATTRIBUTE_DIR.to_string(), &new_eav))
            .and_then(|_| self.write_to_file(VALUE_DIR.to_string(), &new_eav))?;
        Ok(Some(new_eav.clone()))
    }

    fn fetch_eavi(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
        index_query: IndexQuery,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError> {
        let _guard = self.lock.read()?;
        let prefixes = if !index_query.prefixes().is_empty() {
            index_query.prefixes().clone()
        } else {
            vec![""]
        };
        let union_set: BTreeSet<String> = BTreeSet::new();
        let entity_attribute_value_union = prefixes.iter().fold(
            Ok(union_set.clone()),
            |set: Result<BTreeSet<String>, HolochainError>, prefix: &&str| {
                let entity_set =
                    self.read_from_dir::<Entity>(ENTITY_DIR.to_string(), entity.clone())?;
                let attribute_with_prefix = attribute
                    .clone()
                    .map(|attri| prefix.to_string() + &attri.clone());
                let attribute_set = self
                    .read_from_dir::<Attribute>(ATTRIBUTE_DIR.to_string(), attribute_with_prefix)
                    .clone()?;
                let value_set =
                    self.read_from_dir::<Value>(VALUE_DIR.to_string(), value.clone())?;

                let attribute_value_inter: BTreeSet<String> = value_set
                    .intersection(&attribute_set.clone())
                    .cloned()
                    .collect();
                let entity_attribute_value_inter: BTreeSet<String> = attribute_value_inter
                    .intersection(&entity_set)
                    .cloned()
                    .collect();
                let to_union = set?;
                Ok(to_union
                    .union(&entity_attribute_value_inter)
                    .cloned()
                    .collect())
            },
        );

        let (eav, error): (BTreeSet<_>, BTreeSet<_>) = entity_attribute_value_union?
            .clone()
            .into_iter()
            .map(|content| EntityAttributeValueIndex::try_from_content(&JsonString::from(content)))
            .partition(|c| c.is_ok());
        if error.len() > 0 {
            Err(HolochainError::ErrorGeneric(
                "Error Converting EAVs".to_string(),
            ))
        } else {
            let map: BTreeSet<EntityAttributeValueIndex> = eav
                .clone()
                .into_iter()
                .map(|value: HcResult<EntityAttributeValueIndex>| {
                    value.unwrap_or(EntityAttributeValueIndex::default())
                })
                .collect();
            Ok(map
                .clone()
                .into_iter()
                .filter(|e| {
                    index_query
                        .start()
                        .map(|start| start <= e.index())
                        .unwrap_or_else(|| {
                            let latest = get_latest(e.clone(), map.clone(), index_query.clone())
                                .unwrap_or(EntityAttributeValueIndex::default());
                            latest.index() == e.index()
                        })
                })
                .filter(|e| {
                    index_query
                        .end()
                        .map(|end| end >= e.index())
                        .unwrap_or_else(|| {
                            let latest = get_latest(e.clone(), map.clone(), index_query.clone())
                                .unwrap_or(EntityAttributeValueIndex::default());
                            latest.index() == e.index()
                        })
                })
                .collect::<BTreeSet<EntityAttributeValueIndex>>())
        }
    }
}

#[cfg(test)]
pub mod tests {
    extern crate tempfile;
    use self::tempfile::tempdir;
    use eav::file::EavFileStorage;
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
    fn file_eav_one_to_many() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_one_to_many::<ExampleAddressableContent, EavFileStorage>(eav_storage)
    }

    #[test]
    fn file_eav_many_to_one() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_many_to_one::<ExampleAddressableContent, EavFileStorage>(eav_storage)
    }

    #[test]
    fn example_eav_range() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_range::<ExampleAddressableContent, EavFileStorage>(eav_storage);
    }

}
