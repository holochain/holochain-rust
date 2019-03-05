use std::path::{Path, PathBuf};

pub fn storage_path(path: &Path, folder_name: &str) -> PathBuf {
    path.join(".hc").join("storage").join(folder_name)
}

#[cfg(test)]
pub mod tests {

    use crate::path::storage_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_storage_path() {
        let dummy_path = storage_path(Path::new("foo"), "bar");
        let expected_path: PathBuf = vec!["foo", ".hc", "storage", "bar"].iter().collect();
        assert_eq!(dummy_path, expected_path);
    }
}
