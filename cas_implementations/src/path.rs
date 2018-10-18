use holochain_core_types::error::HolochainError;
use std::path::Path;

pub fn storage_path(path : &Path,folder_name : &str) -> Result<String, HolochainError> {

         let full_path = path.join(folder_name);
         let path_as_string = full_path
                .to_str().ok_or(HolochainError::IoError(
                    "Could not find home directory".to_string(),
                ))?;
        Ok(String::from(path_as_string))
}