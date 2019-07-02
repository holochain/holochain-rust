use crate::{
    error::DefaultResult,
};
use std::{
    path::PathBuf,
};
use git2::Repository;

const RUST_TEMPLATE_REPO_URL: &str = "https://github.com/holochain/rust-zome-template";


pub fn generate(zome_name: &PathBuf, scaffold: &String) -> DefaultResult<()> {

    // match against all supported templates
    let url = match scaffold.as_ref() {
        "rust" => {
            RUST_TEMPLATE_REPO_URL
        },
        _ => scaffold, // if not a known type assume that a repo url was passed
    };

    Repository::clone(url, zome_name)?;

    // apply the template substitution

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
