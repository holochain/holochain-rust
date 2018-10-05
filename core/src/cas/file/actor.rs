use actor::{Protocol, SYS};
use cas::content::{Address, Content};
use error::HolochainError;
use riker::actors::*;
use std::{
    fs::{create_dir_all, read_to_string, write},
    path::{Path, MAIN_SEPARATOR},
};

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
    fn props(dir_path: String) -> BoxActorProd<Protocol> {
        Props::new_args(Box::new(FilesystemStorageActor::actor), dir_path)
    }

    pub fn new_ref(dir_path: String) -> Result<ActorRef<Protocol>, HolochainError> {
        let canonical = Path::new(&dir_path).canonicalize()?;
        if !canonical.is_dir() {
            return Err(HolochainError::IoError(
                "path is not a directory or permissions don't allow access".to_string(),
            ));
        }
        let dir_path = canonical
            .to_str()
            .ok_or_else(|| HolochainError::IoError("could not convert path to string".to_string()))?
            .to_string();
        Ok(SYS.actor_of(
            FilesystemStorageActor::props(dir_path.clone()),
            // always return the same reference to the same actor for the same path
            // consistency here provides safety for CAS methods
            &format!("filesystem_storage_actor/{}", dir_path),
        )?)
    }

    /// builds an absolute path for an AddressableContent address
    fn address_to_path(&self, address: &Address) -> String {
        // using .txt extension because content is arbitrary and controlled by the
        // AddressableContent trait implementation
        format!("{}{}{}.txt", self.dir_path, MAIN_SEPARATOR, address)
    }

    /// filesystem CAS add. NOT thread safe.
    fn unsafe_add(&self, address: &Address, content: &Content) -> Result<(), HolochainError> {
        // @TODO be more efficient here
        // @see https://github.com/holochain/holochain-rust/issues/248
        create_dir_all(&self.dir_path)?;
        Ok(write(self.address_to_path(address), content)?)
    }

    /// filesystem CAS contains. NOT thread safe.
    fn unsafe_contains(&self, address: &Address) -> Result<bool, HolochainError> {
        Ok(Path::new(&self.address_to_path(address)).is_file())
    }

    /// filesystem CAS fetch. NOT thread safe.
    fn unsafe_fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
        if self.unsafe_contains(&address)? {
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
                        Protocol::CasAddResult(self.unsafe_add(&address, &content))
                    }
                    Protocol::CasContains(address) => {
                        Protocol::CasContainsResult(self.unsafe_contains(&address))
                    }
                    Protocol::CasFetch(address) => {
                        Protocol::CasFetchResult(self.unsafe_fetch(&address))
                    }
                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .expect("failed to tell FilesystemStorage sender");
    }
}
