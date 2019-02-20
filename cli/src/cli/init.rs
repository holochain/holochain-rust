use crate::{
    cli::{
        package::{GITIGNORE_FILE_NAME, IGNORE_FILE_NAME},
        test::TEST_DIR_NAME,
    },
    config_files::App as AppConfig,
    error::DefaultResult,
    util::DIST_DIR_NAME,
};
use colored::*;
use holochain_common::paths::DNA_EXTENSION;
use serde_json;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
};

fn create_test_file(
    test_folder_path: &PathBuf,
    test_file_name: &str,
    test_file_contents: &str,
    maybe_replacements: Option<HashMap<&str, &str>>,
) -> DefaultResult<()> {
    let dest_filepath = test_folder_path.join(test_file_name);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest_filepath)?;
    let contents = match maybe_replacements {
        Some(replacements) => {
            let mut contents = test_file_contents.to_string();
            for (from, to) in &replacements {
                contents = contents.replace(from, to);
            }
            contents
        }
        None => test_file_contents.to_string(),
    };
    file.write_all(contents.as_bytes())?;
    Ok(())
}

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($key, $val); )*
            map
    }}
}

fn setup_test_folder(path: &PathBuf, test_folder: &str) -> DefaultResult<()> {
    let dir_name = path.file_name().expect("directory to be a file");
    let project_name = dir_name.to_string_lossy().to_string();

    let tests_path = path.join(test_folder);
    fs::create_dir_all(tests_path.clone())?;
    create_test_file(
        &tests_path,
        "index.js",
        include_str!("js-tests-scaffold/index.js"),
        Some(hashmap!["<<DNA_NAME>>" =>  project_name.as_str()]),
    )?;
    create_test_file(
        &tests_path,
        "package.json",
        include_str!("js-tests-scaffold/package.json"),
        None,
    )?;
    Ok(())
}

pub fn init(path: &PathBuf) -> DefaultResult<()> {
    if !path.exists() {
        fs::create_dir_all(&path)?;
    } else {
        let zomes_dir = fs::read_dir(&path)?;

        if zomes_dir.count() > 0 {
            bail!("directory is not empty");
        }
    }

    // create empty zomes folder
    fs::create_dir_all(path.join("zomes"))?;

    // create base DNA json config
    let app_config_file = File::create(path.join("app.json"))?;
    serde_json::to_writer_pretty(app_config_file, &AppConfig::default())?;

    // create a default .gitignore file with good defaults
    let gitignore_file_path = path.join(GITIGNORE_FILE_NAME);
    let mut gitignore_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(gitignore_file_path)?;
    let gitignore_starter = include_str!("git-scaffold/.gitignore");
    gitignore_file.write_all(gitignore_starter.as_bytes())?;

    // create a default .hcignore file with good defaults
    let ignores = [
        &DIST_DIR_NAME,
        &TEST_DIR_NAME,
        format!("*.{}", DNA_EXTENSION).as_str(),
        "README.md",
    ]
    .join("\n");
    let mut hcignore_file = File::create(path.join(&IGNORE_FILE_NAME))?;
    hcignore_file.write_all(ignores.as_bytes())?;

    // create a test folder with useful files
    setup_test_folder(&path, &TEST_DIR_NAME)?;

    // CLI feedback
    println!(
        "{} new Holochain project at: {:?}",
        "Created".green().bold(),
        path
    );

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    extern crate tempfile;
    use self::tempfile::{Builder, TempDir};

    const HOLOCHAIN_TEST_PREFIX: &str = "org_holochain_test";

    pub fn gen_dir() -> TempDir {
        Builder::new()
            .prefix(HOLOCHAIN_TEST_PREFIX)
            .tempdir()
            .unwrap()
    }

    #[test]
    fn init_test() {
        let dir = gen_dir();
        let dir_path_buf = &dir.path().to_path_buf();
        let result = init(dir_path_buf);

        assert!(result.is_ok());
        assert!(dir_path_buf.join("zomes").exists());
        assert!(dir_path_buf.join("app.json").exists());
        assert!(dir_path_buf.join(IGNORE_FILE_NAME).exists());
        assert!(dir_path_buf.join(GITIGNORE_FILE_NAME).exists());
        assert!(dir_path_buf.join(TEST_DIR_NAME).exists());
    }

    #[test]
    fn setup_test_folder_test() {
        let dir = gen_dir();
        let dir_path_buf = &dir.path().to_path_buf();
        setup_test_folder(dir_path_buf, &TEST_DIR_NAME).expect("Test folder not set up");

        assert!(dir_path_buf.join(&TEST_DIR_NAME).join("index.js").exists());
        assert!(dir_path_buf
            .join(&TEST_DIR_NAME)
            .join("package.json")
            .exists());
    }
}
