use actor::{Protocol, SYS};
use holochain_core_types::{
    cas::content::AddressableContent,
    eav::{Attribute, Entity, EntityAttributeValue, Value},
    error::{HcResult, HolochainError},
    file_validation,
    json::JsonString,
};
use riker::actors::*;
use std::{
    collections::HashSet,
    fs::{create_dir_all, File, OpenOptions},
    io::prelude::*,
    path::MAIN_SEPARATOR,
};
use walkdir::{DirEntry, WalkDir};

const ACTOR_ID_ROOT: &'static str = "/eav_file_actor/";

const ENTITY_DIR: &str = "e";
const ATTRIBUTE_DIR: &str = "a";
const VALUE_DIR: &str = "v";

fn actor_id(dir_path: &str) -> String {
    format!("{}{}", ACTOR_ID_ROOT, dir_path)
}

#[warn(unused_must_use)]
pub fn add_eav_to_hashset(dir_entry: DirEntry, set: &mut HashSet<HcResult<String>>) {
    let path = dir_entry.path();
    match OpenOptions::new().read(true).open(path) {
        Ok(mut file) => {
            let mut content: String = String::new();
            let _result = file
                .read_to_string(&mut content)
                .map(|e| {
                    if e > 0 {
                        Ok(content)
                    } else {
                        Err(HolochainError::IoError(format!(
                            "Could not read from path {:?}",
                            path
                        )))
                    }
                })
                .map(|e| {
                    set.insert(e);
                });
        }
        Err(_) => {
            set.insert(Err(HolochainError::IoError(format!(
                "Could not read from path {:?}",
                path
            ))));
        }
    }
}

pub struct EavFileStorageActor {
    dir_path: String,
}

impl EavFileStorageActor {
    pub fn new(dir_path: String) -> EavFileStorageActor {
        EavFileStorageActor { dir_path }
    }

    /// actor() for riker
    fn actor(dir_path: String) -> BoxActor<Protocol> {
        Box::new(EavFileStorageActor::new(dir_path))
    }

    /// props() for riker
    fn props(dir_path: &str) -> BoxActorProd<Protocol> {
        Props::new_args(Box::new(EavFileStorageActor::actor), dir_path.to_string())
    }

    pub fn new_ref(dir_path: &str) -> Result<ActorRef<Protocol>, HolochainError> {
        let dir_path = file_validation::validate_canonical_path(dir_path)?;
        SYS.actor_of(
            EavFileStorageActor::props(&dir_path),
            // always return the same reference to the same actor for the same path
            // consistency here provides safety for CAS methods
            &actor_id(&dir_path),
        ).map_err(|actor_create_error| {
            HolochainError::ErrorGeneric(format!(
                "Failed to create actor in system: {:?}",
                actor_create_error
            ))
        })
    }

    fn write_to_file(
        &self,
        subscript: String,
        eav: &EntityAttributeValue,
    ) -> Result<(), HolochainError> {
        let address: String = match &*subscript {
            ENTITY_DIR => eav.entity().to_string(),
            ATTRIBUTE_DIR => eav.attribute(),
            VALUE_DIR => eav.value().to_string(),
            _ => String::new(),
        };
        let path =
            vec![self.dir_path.clone(), subscript, address].join(&MAIN_SEPARATOR.to_string());
        create_dir_all(path.clone())?;
        let address_path = vec![path, eav.address().to_string()].join(&MAIN_SEPARATOR.to_string());
        let mut f = File::create(address_path)?;
        writeln!(f, "{}", String::from(eav.content()))?;
        Ok(())
    }

    fn read_from_dir<T>(
        &self,
        subscript: String,
        eav_constraint: Option<T>,
    ) -> HashSet<HcResult<String>>
    where
        T: ToString,
    {
        let address = eav_constraint
            .map(|e| e.to_string())
            .unwrap_or(String::new());
        let full_path =
            vec![self.dir_path.clone(), subscript, address].join(&MAIN_SEPARATOR.to_string());
        let mut set = HashSet::new();
        WalkDir::new(full_path.clone())
            .into_iter()
            .for_each(|dir_entry| match dir_entry {
                Ok(eav_content) => {
                    add_eav_to_hashset(eav_content, &mut set);
                }
                Err(_) => {
                    set.insert(Err(HolochainError::IoError(format!(
                        "Could not obtain directory{:?}",
                        full_path
                    ))));
                }
            });

        set
    }

    fn unthreadable_add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        create_dir_all(self.dir_path.clone())?;
        self.write_to_file(ENTITY_DIR.to_string(), eav)
            .and_then(|_| self.write_to_file(ATTRIBUTE_DIR.to_string(), eav))
            .and_then(|_| self.write_to_file(VALUE_DIR.to_string(), eav))
    }

    fn unthreadable_fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        let entity_set = self.read_from_dir::<Entity>(ENTITY_DIR.to_string(), entity);
        let attribute_set = self
            .read_from_dir::<Attribute>(ATTRIBUTE_DIR.to_string(), attribute)
            .clone();
        let value_set = self.read_from_dir::<Value>(VALUE_DIR.to_string(), value);
        let attribute_value_inter = attribute_set.intersection(&value_set).cloned().collect();
        let entity_attribute_value_inter: HashSet<Result<String, HolochainError>> = entity_set
            .intersection(&attribute_value_inter)
            .cloned()
            .collect();
        Ok(entity_attribute_value_inter
            .into_iter()
            .filter(|e| e.is_ok())
            .map(|eav_content| {
                EntityAttributeValue::from_content(&JsonString::from(eav_content.unwrap()))
            })
            .collect())
    }
}

impl Actor for EavFileStorageActor {
    type Msg = Protocol;

    fn receive(
        &mut self,
        context: &Context<Self::Msg>,
        message: Self::Msg,
        sender: Option<ActorRef<Self::Msg>>,
    ) {
        sender
            .try_tell(
                match message {
                    Protocol::EavAdd(eav) => {
                        Protocol::EavAddResult(self.unthreadable_add_eav(&eav))
                    }
                    Protocol::EavFetch(e, a, v) => {
                        Protocol::EavFetchResult(self.unthreadable_fetch_eav(e, a, v))
                    }
                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .expect("failed to tell FilesystemStorage sender");
    }
}
