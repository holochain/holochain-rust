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
            .recursive(true) // create parent dirs if necessary
            .create(path)
            .map_err(|_| {
                HolochainError::IoError(format!("Could not create directory: {:?}", path))
            });
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::create_path_if_not_exists;
    use crate::path::storage_path;
    use std::path::{Path, PathBuf};
    extern crate tempfile;
    use self::tempfile::tempdir;

    #[cfg(not(windows))]
    extern crate users;

    #[test]
    fn test_storage_path() {
        let dummy_path = storage_path(Path::new("foo"), "bar").unwrap();
        let expected_path: PathBuf = vec!["foo", ".hc", "storage", "bar"].iter().collect();
        assert_eq!(dummy_path, expected_path);
    }

    #[test]
    #[cfg(not(windows))]
    fn test_create_path_if_not_exists() {
        let bad_path = storage_path(Path::new("/*?abc"), "bar").unwrap();
        let result = create_path_if_not_exists(&bad_path);
        match result {
            Ok(()) => assert!(
                self::users::get_current_uid() == 0,
                "Creation of / path should only work for root, not UID {}",
                self::users::get_current_uid()
            ),
            Err(err) => {
                assert!(err.to_string() == "Could not create directory: \"/*?abc/.hc/storage/bar\"")
            }
        };
        let dir = tempdir().unwrap();
        let result = create_path_if_not_exists(dir.path());
        match result {
            Ok(val) => assert_eq!(val, ()),
            Err(_) => unreachable!(),
        };
    }

    #[test]
    #[cfg(windows)]
    fn test_create_path_if_not_exists() {
        let bad_path = storage_path(Path::new("/*?abc"), "bar").unwrap();
        let result = create_path_if_not_exists(&bad_path);
        match result {
            Ok(()) => unreachable!(),
            Err(err) => assert!(
                err.to_string()
                    == "Could not create directory: \"/*?abc\\\\.hc\\\\storage\\\\bar\""
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
