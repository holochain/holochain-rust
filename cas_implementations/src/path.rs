use holochain_core_types::error::{HcResult, HolochainError};
use std::{
    fs::DirBuilder,
    path::{Path, PathBuf},
};

pub fn storage_path(path: &Path, folder_name: &str) -> HcResult<PathBuf> {
    let full_path = path.join(".hc").join("storage").join(folder_name);

    Ok(full_path)
}

pub fn create_path_if_not_exists(path: &Path) -> HcResult<()> {
    if !path.exists() {
        return DirBuilder::new()
            .create(path)
            .map_err(|_| HolochainError::IoError("Could not create directory".to_string()));
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::create_path_if_not_exists;
    use crate::path::storage_path;
    use holochain_core_types::error::HolochainError;
    use std::path::{Path, PathBuf};
    extern crate tempfile;
    use self::tempfile::tempdir;

    #[test]
    fn test_storage_path() {
        let dummy_path = storage_path(Path::new("foo"), "bar").unwrap();
        let expected_path: PathBuf = vec!["foo", ".hc", "storage", "bar"].iter().collect();
        assert_eq!(dummy_path, expected_path);
    }

    #[test]
    fn test_create_path_if_not_exists() {
        let bad_path = storage_path(Path::new("/*?abc"), "bar").unwrap();
        let result = create_path_if_not_exists(&bad_path);
        match result {
            Ok(()) => unreachable!(),
            Err(err) => assert_eq!(
                err,
                HolochainError::IoError("Could not create directory".to_string())
            ),
        };
        let dir = tempdir().unwrap();
        let result = create_path_if_not_exists(dir.path());
        match result {
            Ok(val) => assert_eq!(val, ()),
            Err(_) => unreachable!(),
        };
    }
}
