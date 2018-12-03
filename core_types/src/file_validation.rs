use crate::error::HolochainError;
use std::path::Path;

pub fn validate_canonical_path(dir_path: &str) -> Result<String, HolochainError> {
    let canonical = Path::new(&dir_path).canonicalize()?;
    if !canonical.is_dir() {
        return Err(HolochainError::IoError(
            "path is not a directory or permissions don't allow access".to_string(),
        ));
    }
    canonical
        .to_str()
        .map(|e| String::from(e))
        .ok_or_else(|| HolochainError::IoError("could not convert path to string".to_string()))
}
