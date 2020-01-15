use crate::{error::DefaultResult, util};
use serde_json;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use holochain_core_types::dna::wasm::DnaWasm;

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

    /// Starts the build using the supplied build steps and returns the contents of the artifact as DnaWasm
    pub fn run(&self, base_path: &PathBuf) -> DefaultResult<DnaWasm> {
        for build_step in &self.steps {
            let slice_vec: Vec<_> = build_step.arguments.iter().map(|e| e.as_str()).collect();
            util::run_cmd(
                &base_path.to_path_buf(),
                build_step.command.clone(),
                &slice_vec[..],
            )?;
        }

        let artifact_path_bashed = std::process::Command::new("bash")
            .args(&["-c", &format!("echo {}", self.artifact.to_string_lossy(),)])
            .output()?
            .stdout;

        let artifact_path_str = std::str::from_utf8(&artifact_path_bashed)?.trim_end();

        let artifact_path_buf = PathBuf::from(artifact_path_str);

        let artifact_path = if artifact_path_buf.is_absolute() {
            artifact_path_buf
        } else {
            base_path.join(artifact_path_buf)
        };

        if artifact_path.exists() && artifact_path.is_file() {
            let mut wasm_buf = Vec::new();
            File::open(&artifact_path)?.read_to_end(&mut wasm_buf)?;

            Ok(DnaWasm::from_bytes(wasm_buf))
        } else {
            bail!(
                "artifact path {} either doesn't point to a file or doesn't exist",
                artifact_path.to_string_lossy()
            )
        }
    }
}
