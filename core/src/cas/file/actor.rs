use std::fs::Path;
use error::HolochainError;
use actor::Protocol;
use riker::actors::*;
use cas::file::FilesystemStorage;
use cas::content::Address;
use cas::content::Content;
use std::fs::create_dir_all;
use std::fs::write;

pub struct FilesystemStorageActor {
    /// path to the directory where content will be saved to disk
    dir_path: String,
}

impl FilesystemStorageActor {
    fn new_ref(dir_path: String) -> Result<ActorRef<Protocol>, HolochainError> {
        let canonical = Path::new(dir_path).canonicalize()?;
        if !canonical.is_dir() {
            return Err(HolochainError::IoError(
                "path is not a directory or permissions don't allow access".to_string(),
            ));
        }
        Ok(FilesystemStorage {
            dir_path: canonical
                .to_str()
                .ok_or_else(|| {
                    HolochainError::IoError("could not convert path to string".to_string())
                })?
                .to_string(),
        })
    }

    fn unsafe_add(&self, address: &Address, content: &Content) -> Result<(), HolochainError> {
        // @TODO be more efficient here
        // @see https://github.com/holochain/holochain-rust/issues/248
        create_dir_all(&self.dir_path)?;
        Ok(write(
            self.address_to_path(address),
            content,
        )?)
    }
}
