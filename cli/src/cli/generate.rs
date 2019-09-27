use crate::error::DefaultResult;
use git2::Repository;
use glob::glob;
use std::{fs, io::prelude::*, path::PathBuf};
use tera::{Context, Tera};

const RUST_TEMPLATE_REPO_URL: &str = "https://github.com/holochain/rust-zome-template";
const RUST_PROC_TEMPLATE_REPO_URL: &str = "https://github.com/holochain/rust-proc-zome-template";

const HOLOCHAIN_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn generate(zome_path: &PathBuf, scaffold: &String) -> DefaultResult<()> {
    let zome_name = zome_path
        .components()
        .last()
        .ok_or_else(|| format_err!("New zome path must have a target directory"))?
        .as_os_str()
        .to_str()
        .ok_or_else(|| format_err!("Zome path contains invalid characters"))?;

    // match against all supported templates
    let url = match scaffold.as_ref() {
        "rust" => RUST_TEMPLATE_REPO_URL,
        "rust-proc" => RUST_PROC_TEMPLATE_REPO_URL,
        _ => scaffold, // if not a known type assume that a repo url was passed
    };

    Repository::clone(url, zome_path)?;

    // delete the .git directory
    fs::remove_dir_all(zome_path.join(".git"))?;

    let mut context = Context::new();
    context.insert("name", &zome_name);
    context.insert("author", &"hc-scaffold-framework");
    context.insert("version", HOLOCHAIN_VERSION);

    apply_template_substitution(zome_path, context)?;

    Ok(())
}

fn apply_template_substitution(root_path: &PathBuf, context: Context) -> DefaultResult<()> {
    let zome_name_component = root_path
        .components()
        .last()
        .ok_or_else(|| format_err!("New zome path must have a target directory"))?;
    let template_glob_path: PathBuf = [root_path, &PathBuf::from("**/*")].iter().collect();
    let template_glob = template_glob_path
        .to_str()
        .ok_or_else(|| format_err!("Zome path contains invalid characters"))?;

    let templater =
        Tera::new(template_glob).map_err(|_| format_err!("Could not load repo for templating"))?;

    for entry in glob(template_glob).map_err(|_| format_err!("Failed to read glob pattern"))? {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    let template_id: PathBuf = path
                        .components()
                        .skip_while(|c| c != &zome_name_component)
                        .skip(1)
                        .collect();
                    let result = templater
                        .render(template_id.to_str().unwrap(), &context)
                        .unwrap();
                    let mut file = fs::OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(path)?;
                    file.write_all(result.as_bytes())?;
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(())
}

#[cfg(test)]
// too slow!
#[cfg(feature = "broken-tests")]
mod tests {
    use assert_cmd::prelude::*;
    use std::process::Command;
    use tempfile::{Builder, TempDir};

    const HOLOCHAIN_TEST_PREFIX: &str = "org.holochain.test";

    fn gen_dir() -> TempDir {
        Builder::new()
            .prefix(HOLOCHAIN_TEST_PREFIX)
            .tempdir()
            .unwrap()
    }

    #[test]
    fn can_generate_scaffolds() {
        let tmp = gen_dir();

        Command::main_binary()
            .unwrap()
            .current_dir(&tmp.path())
            .args(&["init", "."])
            .assert()
            .success();

        Command::main_binary()
            .unwrap()
            .current_dir(&tmp.path())
            .args(&["g", "zomes/bubblechat", "rust"])
            .assert()
            .success();

        // TODO: We cannot test this since there is no complete implementation of hdk-assemblyscript
        // Command::main_binary()
        //  .unwrap()
        //   .current_dir(&tmp.path())
        //   .args(&["g", "zomes/zubblebat", "assemblyscript"])
        //   .assert()
        //   .success();
    }
}
