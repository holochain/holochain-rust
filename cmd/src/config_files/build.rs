use crate::{error::DefaultResult, util};
use base64;
use serde_json;
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Deserialize, Serialize)]
pub struct Build {
    pub steps: HashMap<String, Vec<String>>,
    pub artifact: PathBuf,
}

impl Build {
    /// Creates a Build struct from a .build JSON file and returns it
    pub fn from_file<T: AsRef<Path>>(path: T) -> DefaultResult<Build> {
        let file = File::open(path)?;

        let build = serde_json::from_reader(&file)?;

        Ok(build)
    }

    pub fn save_as<T: AsRef<Path>>(&self, path: T) -> DefaultResult<()> {
        let file = File::create(path)?;

        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }

    /// Starts the build using the supplied build steps and returns the contents of the artifact
    pub fn run(&self, base_path: &PathBuf) -> DefaultResult<String> {
        for (bin, args) in &self.steps {
            let slice_vec: Vec<_> = args.iter().map(|e| e.as_str()).collect();
            util::run_cmd(base_path.to_path_buf(), bin.to_string(), &slice_vec[..])?;
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

    pub fn with_artifact<P: Into<PathBuf>>(artifact: P) -> Build {
        let path: PathBuf = artifact.into();

        Build {
            steps: HashMap::new(),
            artifact: path,
        }
    }

    pub fn cmd(mut self, cmd: &str, args: &[&str]) -> Build {
        let cmd: String = cmd.to_owned();
        let args = args
            .to_vec()
            .iter()
            .map(|raw_arg| raw_arg.to_string())
            .collect();

        self.steps.insert(cmd, args);
        self
    }
}
