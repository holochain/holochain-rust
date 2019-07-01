use crate::{
    cli::{
        package::CODE_DIR_NAME,
        scaffold::{self, Scaffold},
    },
    error::DefaultResult,
    util,
};
use serde_json;
use std::{
    fs::{self, File},
    path::PathBuf,
};

pub const ZOME_CONFIG_FILE_NAME: &str = "zome.json";

pub fn generate(zome_name: &PathBuf, language: &str) -> DefaultResult<()> {
    if !zome_name.exists() {
        fs::create_dir_all(&zome_name)?;
    }

    ensure!(
        zome_name.is_dir(),
        "argument \"zome_name\" doesn't point to a directory"
    );

    let file_name = util::file_name_string(&zome_name)?;

    let zome_config_json = json! {
        {
            "description": format!("The {} App", file_name)
        }
    };

    let file = File::create(zome_name.join(ZOME_CONFIG_FILE_NAME))?;
    serde_json::to_writer_pretty(file, &zome_config_json)?;

    let code_dir = zome_name.join(CODE_DIR_NAME);
    fs::create_dir_all(&code_dir)?;
    let zome_name_string = zome_name
        .to_str()
        .expect("Invalid zome path given")
        .to_string()
        .replace("/", "_")
        .replace("zomes_", "");

    // match against all supported languages
    match language {
        "rust" => {
            ensure!(
                !file_name.contains("-"),
                "Cannot use hyphens '-' as names for Rust zomes. Use underscore '_' instead."
            );
            scaffold(
                &scaffold::rust::RustScaffold::new(
                    &zome_name_string,
                    scaffold::rust::HdkMacroStyle::Declarative,
                ),
                code_dir,
            )?
        }
        "rust-proc" => {
            ensure!(
                !file_name.contains("-"),
                "Cannot use hyphens '-' as names for Rust-Proc zomes. Use underscore '_' instead."
            );
            scaffold(
                &scaffold::rust::RustScaffold::new(
                    &zome_name_string,
                    scaffold::rust::HdkMacroStyle::Procedural,
                ),
                code_dir,
            )?
        }
        "assemblyscript" => scaffold(
            &scaffold::assemblyscript::AssemblyScriptScaffold::new(),
            code_dir,
        )?,
        // TODO: supply zome name for AssemblyScriptScaffold as well
        _ => bail!("unsupported language: {}", language),
    }

    Ok(())
}

fn scaffold<S: Scaffold>(tooling: &S, base_path: PathBuf) -> DefaultResult<()> {
    tooling.gen(base_path)
}

#[cfg(test)]
// too slow!
// #[cfg(feature = "broken-tests")]
mod tests {
    extern crate tempfile;
    use self::tempfile::{Builder, TempDir};
    extern crate assert_cmd;
    use self::assert_cmd::prelude::*;
    use std::{fs, process::Command};

    const HOLOCHAIN_TEST_PREFIX: &str = "org.holochain.test";

    fn gen_dir() -> TempDir {
        Builder::new()
            .prefix(HOLOCHAIN_TEST_PREFIX)
            .tempdir()
            .unwrap()
    }

    #[test]
    fn can_generate_scaffolds() {
        let shared_space = gen_dir();

        let root_path = shared_space.path().to_path_buf();

        fs::create_dir_all(&root_path).unwrap();

        Command::main_binary()
            .unwrap()
            .current_dir(&root_path)
            .args(&["init", root_path.to_str().unwrap()])
            .assert()
            .success();

        Command::main_binary()
            .unwrap()
            .current_dir(&root_path)
            .args(&["generate", "zomes/bubblechat", "rust"])
            .assert()
            .success();
    }

    #[test]
    fn rejects_rust_zomes_with_hyphens() {
        let shared_space = gen_dir();

        let root_path = shared_space.path().to_path_buf();

        fs::create_dir_all(&root_path).unwrap();

        Command::main_binary()
            .unwrap()
            .current_dir(&root_path)
            .args(&["init", root_path.to_str().unwrap()])
            .assert()
            .success();

        Command::main_binary()
            .unwrap()
            .current_dir(&root_path)
            .args(&["generate", "zomes/bubble-chat", "rust"])
            .assert()
            .failure();
    }

}
