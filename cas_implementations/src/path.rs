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
        return DirBuilder::new()
            .create(path)
            .map_err(|_| HolochainError::IoError("Could not create directory".to_string()));
    }
    Ok(())
}

pub mod tests {
    use path::storage_path;
    use std::path::{Path, MAIN_SEPARATOR};

    #[test]
    fn test_storage_path() {
        let dummy_path = storage_path(Path::new("foo"), "bar").unwrap();
        let expected_path = vec!["foo", ".hc", "storage", "bar"].join(&MAIN_SEPARATOR.to_string());
        assert_eq!(dummy_path, expected_path);
    }
}
