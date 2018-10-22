use super::super::super::actor::{Protocol, SYS};
use holochain_core_types::{
    cas::content::{Address, Content}, error::HolochainError, file_validation,
};
use riker::actors::*;
use std::{
    fs::{create_dir_all, read_to_string, write}, path::{Path, MAIN_SEPARATOR},
};

const ACTOR_ID_ROOT: &'static str = "/filesystem_storage_actor/";

fn actor_id(dir_path: &str) -> String {
    format!("{}{}", ACTOR_ID_ROOT, dir_path)
}

pub struct FilesystemStorageActor {
    /// path to the directory where content will be saved to disk
    dir_path: String,
}

impl FilesystemStorageActor {
    pub fn new(dir_path: String) -> FilesystemStorageActor {
        FilesystemStorageActor { dir_path }
    }

    /// actor() for riker
    fn actor(dir_path: String) -> BoxActor<Protocol> {
        Box::new(FilesystemStorageActor::new(dir_path))
    }

    /// props() for riker
    fn props(dir_path: &str) -> BoxActorProd<Protocol> {
        Props::new_args(
            Box::new(FilesystemStorageActor::actor),
            dir_path.to_string(),
        )
    }

    pub fn new_ref(dir_path: &str) -> Result<ActorRef<Protocol>, HolochainError> {
        let dir_path = file_validation::validate_canonical_path(dir_path)?;
        SYS.actor_of(
            FilesystemStorageActor::props(&dir_path),
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

    /// builds an absolute path for an AddressableContent address
    fn address_to_path(&self, address: &Address) -> String {
        // using .txt extension because content is arbitrary and controlled by the
        // AddressableContent trait implementation
        format!("{}{}{}.txt", self.dir_path, MAIN_SEPARATOR, address)
    }

    /// filesystem CAS add. NOT thread safe.
    fn unthreadable_add(&self, address: &Address, content: &Content) -> Result<(), HolochainError> {
        // @TODO be more efficient here
        // @see https://github.com/holochain/holochain-rust/issues/248
        create_dir_all(&self.dir_path)?;
        Ok(write(self.address_to_path(address), content)?)
    }

    /// filesystem CAS contains. NOT thread safe.
    fn unthreadable_contains(&self, address: &Address) -> Result<bool, HolochainError> {
        Ok(Path::new(&self.address_to_path(address)).is_file())
    }

    /// filesystem CAS fetch. NOT thread safe.
    fn unthreadable_fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
        if self.unthreadable_contains(&address)? {
            Ok(Some(read_to_string(self.address_to_path(address))?))
        } else {
            Ok(None)
        }
    }
}

impl Actor for FilesystemStorageActor {
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
                    Protocol::CasAdd(address, content) => {
                        Protocol::CasAddResult(self.unthreadable_add(&address, &content))
                    }
                    Protocol::CasContains(address) => {
                        Protocol::CasContainsResult(self.unthreadable_contains(&address))
                    }
                    Protocol::CasFetch(address) => {
                        Protocol::CasFetchResult(self.unthreadable_fetch(&address))
                    }
                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .expect("failed to tell FilesystemStorage sender");
    }
}

#[cfg(test)]
pub mod tests {

    use cas::file::actor::actor_id;

    #[test]
    fn path_to_actor_id_test() {
        assert_eq!(
            String::from("/filesystem_storage_actor/foo"),
            actor_id("foo"),
        );
    }

}
