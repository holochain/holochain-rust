use holochain_core_types::error::{HcResult, HolochainError};
use std::{fs::DirBuilder, path::Path};

pub fn storage_path(path: &Path, folder_name: &str) -> HcResult<String> {
    let full_path = path.join(".hc").join("storage").join(folder_name);
    let path_as_string = full_path.to_str().ok_or(HolochainError::IoError(
        "Could not find home directory".to_string(),
    ))?;
    Ok(String::from(path_as_string))
}

pub fn create_path_if_not_exists(path: &str) -> HcResult<()> {
    if !Path::new(path).exists() {
        return DirBuilder::new().recursive(true).create(path).map_err(|e| {
            HolochainError::IoError(format!(
                "Error while attempting to create directory {}: {}",
                path, e
            ))
        });
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::create_path_if_not_exists;
    use crate::path::storage_path;
    use holochain_core_types::error::HolochainError;
    use std::path::{Path, MAIN_SEPARATOR};
    extern crate tempfile;
    use self::tempfile::tempdir;

    #[test]
    fn test_storage_path() {
        let dummy_path = storage_path(Path::new("foo"), "bar").unwrap();
        let expected_path = vec!["foo", ".hc", "storage", "bar"].join(&MAIN_SEPARATOR.to_string());
        assert_eq!(dummy_path, expected_path);
    }

    #[test]
    fn test_create_path_if_not_exists() {
        let bad_path = storage_path(Path::new("/foo"), "bar").unwrap();
        let result = create_path_if_not_exists(&bad_path);
        match result {
            Ok(()) => panic!("expected error"),
            Err(HolochainError::IoError(_)) => (),
            Err(_) => panic!("expected IoError"),
        };
        let dir = tempdir().unwrap();
        let file_path = dir.path();
        let result = create_path_if_not_exists(file_path.to_str().unwrap());
        match result {
            Ok(val) => assert_eq!(val, ()),
            Err(_) => unreachable!(),
        };
    }
}
