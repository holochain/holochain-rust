use crate::{
    cli::{package, scaffold::Scaffold},
    config_files::Build,
    error::DefaultResult,
    util,
};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub const TSCONFIG_FILE_NAME: &str = "tsconfig.json";
pub const TYPESCRIPT_FILE_NAME: &str = "index.ts";

pub struct AssemblyScriptScaffold {
    build_template: Build,
}

impl AssemblyScriptScaffold {
    pub fn new() -> AssemblyScriptScaffold {
        AssemblyScriptScaffold {
            build_template: Build::with_artifact("module.wasm").cmd(
                "./node_modules/assemblyscript/bin/asc",
                &[
                    "index.ts",
                    "-b",
                    "module.wasm",
                    "--transform",
                    "./node_modules/hdk-assemblyscript/transforms",
                ],
            ),
        }
    }
}

impl Scaffold for AssemblyScriptScaffold {
    fn gen<P: AsRef<Path>>(&self, base_path: P) -> DefaultResult<()> {
        fs::create_dir_all(&base_path)?;

        // use npm to initialise a nodejs project
        util::run_cmd(
            base_path.as_ref().to_path_buf(),
            "npm".into(),
            &["init", "-y"],
        )?;

        // add hdk-assemblyscript as a dependency
        util::run_cmd(
            base_path.as_ref().to_path_buf(),
            "npm".into(),
            &["install", "--save", "holochain/hdk-assemblyscript"],
        )?;

        // create a index.ts file
        let typescript_file_path = base_path.as_ref().join(TYPESCRIPT_FILE_NAME);

        let mut typescript_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(typescript_file_path)?;

        let js_starter = include_str!("assemblyscript/index-ts-starter.ts");

        typescript_file.write_all(js_starter.as_bytes())?;

        // create a tsconfig.json file
        let tsconfig_file_path = base_path.as_ref().join(TSCONFIG_FILE_NAME);

        let mut tsconfig_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(tsconfig_file_path)?;

        let tsconfig_json = include_str!("assemblyscript/tsconfig.json");

        tsconfig_file.write_all(tsconfig_json.as_bytes())?;

        // create and fill in a build file appropriate for AssemblyScript
        let build_file_path = base_path.as_ref().join(package::BUILD_CONFIG_FILE_NAME);

        self.build_template.save_as(build_file_path)?;

        Ok(())
    }
}
