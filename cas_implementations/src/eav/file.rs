use chrono::{offset::Utc, DateTime};
use holochain_core_types::{
    cas::content::{AddressableContent, Content},
    eav::{Action, Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value,Key,from_key},
    error::{HcResult, HolochainError},
    hash::HashString,
    json::JsonString,
};
use im::hashmap::HashMap;
use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{prelude::*, ErrorKind},
    path::{Path,PathBuf, MAIN_SEPARATOR},
    sync::{Arc, RwLock},
};
use uuid::Uuid;
use glob::glob;

const ENTITY_DIR: &str = "e";
const ATTRIBUTE_DIR: &str = "a";
const VALUE_DIR: &str = "v";

#[derive(Clone, Debug)]
pub struct EavFileStorage {
    dir_path: String,
    id: Uuid,
    current_hash: HashString,
    lock: Arc<RwLock<()>>,
}

impl PartialEq for EavFileStorage {
    fn eq(&self, other: &EavFileStorage) -> bool {
        self.id == other.id
    }
}

#[warn(unused_must_use)]
pub fn read_eav(parent_path:PathBuf) -> HcResult<Vec<(HashString, String)>> {
    //glob all  files
    let full_path = vec![parent_path.to_str().unwrap_or("").to_string(),"*".to_string(),"*".to_string(),"*.txt".to_string()].join(&MAIN_SEPARATOR.to_string());
    println!("full path {:?}",full_path.clone());
    
    let paths = glob(&*full_path).map_err(|_|HolochainError::ErrorGeneric("Could not get form path".to_string()))?;

   // let path_result = paths.last().ok_or(HolochainError::ErrorGeneric("Could not get form path".to_string()))?;
    let (eav,error) : (Vec<_>,Vec<_>) = paths.map(|path|{
        let path_buf = path.unwrap_or(PathBuf::new());
        OpenOptions::new().read(true).open(path_buf.clone())
        .map(|mut file|{
            let mut content: String = String::new();
            file.read_to_string(&mut content)
            .map(|_|Ok((get_key_from_path(&path_buf).unwrap_or((HashString::from(""))),content)))
            .unwrap_or(Err(HolochainError::ErrorGeneric("Could not read from string".to_string())))
        }).unwrap_or(Err(HolochainError::ErrorGeneric("Could not read from string".to_string())))
    }).partition(Result::is_ok);
    if error.len() > 0
    {
        Err(HolochainError::ErrorGeneric("Could not read from string".to_string()))
    }
    else 
    {
        Ok(eav.iter().cloned().map(|s|s.expect("only good results unwrapped")).collect())
    }

}

fn get_key_from_path(abs_path: &Path) -> HcResult<HashString> {
    let mut path_sections = abs_path
        .to_str()
        .ok_or(HolochainError::ErrorGeneric(
            "Could not get path section".to_string(),
        ))?
        .split(MAIN_SEPARATOR)
        .collect::<Vec<&str>>();
    path_sections.reverse();
    let mut reverse_path_sections = path_sections.iter();
    reverse_path_sections.next();
    let action_type = reverse_path_sections
        .next()
        .ok_or(HolochainError::ErrorGeneric(
            "Cold not get unix_time".to_string(),
        ))?;
    let action = Action::from(action_type.to_string());
    let unix_time = reverse_path_sections
        .next()
        .ok_or(HolochainError::ErrorGeneric(
            "Cold not get unix_time".to_string(),
        ))?;
    Ok(HashString::from(
        vec![unix_time.to_string(), action.to_string()].join("_"),
    ))
}

impl EavFileStorage {
    pub fn new(dir_path: String) -> HcResult<EavFileStorage> {
        Ok(EavFileStorage {
            dir_path,
            id: Uuid::new_v4(),
            lock: Arc::new(RwLock::new(())),
            current_hash: HashString::from(Uuid::new_v4().to_string().replace("-", "_")),
        })
    }

    fn write_to_file(
        &self,
        (unix_time, action): (i64, Action),
        subscript: String,
        eav: &EntityAttributeValue,
    ) -> Result<(), HolochainError> {
        let address: String = match &*subscript {
            ENTITY_DIR => eav.entity().to_string(),
            ATTRIBUTE_DIR => eav.attribute(),
            VALUE_DIR => eav.value().to_string(),
            _ => String::new(),
        };
        let path = vec![
            self.dir_path.clone(),
            subscript,
            address,
            unix_time.to_string(),
            action.to_string(),
        ]
        .join(&MAIN_SEPARATOR.to_string());
             create_dir_all(path.clone())?;
             let address_path = vec![path, eav.address().to_string()].join(&MAIN_SEPARATOR.to_string());
             let full_path = vec![address_path.clone(), "txt".to_string()].join(&".".to_string());
             let mut f = File::create(full_path)?;
             writeln!(f, "{}", eav.content())?;
             Ok(())
        
    }


    fn read_from_dir<T>(
        &self,
        hash: HashString,
        subscript: String,
        eav_constraint: Option<T>,
    ) -> HcResult<HashMap<HashString, String>>
    where
        T: ToString,
    {
        let address = eav_constraint
            .map(|e| e.to_string())
            .unwrap_or(String::from("*"));
        let path =
            vec![self.dir_path.clone(), subscript].join(&MAIN_SEPARATOR.to_string());
        if Path::new(&path.clone()).exists()
        {
            
            let full_path = vec![path.clone(),address.clone()].join(&MAIN_SEPARATOR.to_string());

            let paths = glob(&*full_path.clone()).map_err(|_|HolochainError::ErrorGeneric("Could not get form path".to_string()))?;
            
            let (eavs, errors): (Vec<_>, Vec<_>) = paths
            .map(|path_val| {
                path_val.map(|walk|{
                read_eav(walk.clone()) 
                }).unwrap_or(Err(HolochainError::ErrorGeneric(
                "Could not read eavs from directory".to_string(),
            )))
                
            })
            .partition(Result::is_ok);
        if errors.len() > 0 {
            Err(HolochainError::ErrorGeneric(
                "Could not read eavs from directory".to_string(),
            ))
        } else {
            let mut hashmap: HashMap<HashString, String> = HashMap::new();
            eavs.iter().for_each(|s| {
                s.clone().unwrap_or(Vec::new())
                 .iter().for_each(|k|{
                     let (key, value) = k.clone();
                     hashmap.insert(key, value);
                 })
            });
            Ok(hashmap)
        }
        }
        else
        {
            println!("Cant find");
            Ok(HashMap::new())
        }
    }
}

impl EntityAttributeValueStorage for EavFileStorage {
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        if self.fetch_eav(Some(eav.entity()),Some(eav.attribute()),Some(eav.value()))?.len() ==0 
        {
        let _guard = self.lock.write()?;
        create_dir_all(self.dir_path.clone())?;
        let key = (Utc::now().timestamp_millis(), Action::insert);
        println!("key {:?}",key.0.clone());
        self.write_to_file(key.clone(), ENTITY_DIR.to_string(), eav)
            .and_then(|_| self.write_to_file(key.clone(), ATTRIBUTE_DIR.to_string(), eav))
            .and_then(|_| self.write_to_file(key.clone(), VALUE_DIR.to_string(), eav))
        }
        else
        {
            Ok(())
        }
    }

    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashMap<Key, EntityAttributeValue>, HolochainError> {

        println!("FETCH EAV");
        let _guard = self.lock.read()?;
        let entity_set = self.read_from_dir::<Entity>(
            self.current_hash.clone(),
            ENTITY_DIR.to_string(),
            entity.clone(),
        )?;

        println!("ENTITY SET {:?}",entity_set.clone());
        let attribute_set = self
            .read_from_dir::<Attribute>(
                self.current_hash.clone(),
                ATTRIBUTE_DIR.to_string(),
                attribute,
            )?
            .clone();
        println!("ATTRIBUTE SET {:?}",attribute_set.clone());
        let value_set =
            self.read_from_dir::<Value>(self.current_hash.clone(), VALUE_DIR.to_string(), value)?;
        println!("VALUE SET {:?}",value_set.clone());

        let attribute_value_inter = attribute_set.intersection(value_set);
        println!("intersection {:?}",attribute_value_inter.clone());

        let entity_attribute_value_inter = entity_set.intersection(attribute_value_inter);
        println!("intersection {:?}",entity_attribute_value_inter.clone());
        let (eav, error): (HashMap<_, _>, HashMap<_, _>) = entity_attribute_value_inter
            .into_iter()
            .map(|(hash, content)| {
                (
                    from_key(hash).expect("Could not convert from hash"),
                    EntityAttributeValue::try_from_content(&JsonString::from(content)),
                )
            })
            .partition(|(_, c)| c.is_ok());
        if error.len() > 0 {
            Err(HolochainError::ErrorGeneric(
                "Error Converting EAVs".to_string(),
            ))
        } else {
            Ok(eav
                .into_iter()
                .map(|key_value: (Key, HcResult<EntityAttributeValue>)| {
                    (
                        key_value.0,
                        key_value.1.unwrap_or(EntityAttributeValue::default()),
                    )
                })
                .collect::<HashMap<Key, EntityAttributeValue>>())
        }
    }

    fn fetch_eav_range(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashMap<Key, EntityAttributeValue>, HolochainError> {
        unimplemented!("Could not implment eav on range")
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

}
