use crate::{error::DefaultResult, util};
use base64;
use serde_json;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Deserialize, Serialize)]
pub struct BuildStep {
    pub command: String,
    pub arguments: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Build {
    pub steps: Vec<BuildStep>,
    pub artifact: PathBuf,
}

impl Build {
    /// Creates a Build struct from a .hcbuild JSON file and returns it
    pub fn from_file<T: AsRef<Path>>(path: T) -> DefaultResult<Build> {
        let file = File::open(path)?;

        let build = serde_json::from_reader(&file)?;

        Ok(build)
    }

    /// Starts the build using the supplied build steps and returns the contents of the artifact
    pub fn run(&self, base_path: &PathBuf) -> DefaultResult<String> {
        for build_step in &self.steps {
            let slice_vec: Vec<_> = build_step.arguments.iter().map(|e| e.as_str()).collect();
            util::run_cmd(
                &base_path.to_path_buf(),
                build_step.command.clone(),
                &slice_vec[..],
            )?;
        }

        let artifact_path = base_path.join(&self.artifact);

        if artifact_path.exists() && artifact_path.is_file() {
            let mut wasm_buf = Vec::new();
            File::open(&artifact_path)?.read_to_end(&mut wasm_buf)?;

            Ok(base64::encode(&wasm_buf))
        } else {
            bail!(
                "artifact path {} either doesn't point to a file or doesn't exist",
                artifact_path.to_string_lossy()
            )
        }
    }
}
