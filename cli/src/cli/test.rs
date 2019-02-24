use crate::{cli::package, error::DefaultResult, util};
use colored::*;
use failure::Error;
use std::{
    io::ErrorKind,
    path::PathBuf,
    process::{Command, Stdio},
};

pub const TEST_DIR_NAME: &str = "test";

pub fn test(
    path: &PathBuf,
    tests_folder: &str,
    testfile: &str,
    skip_build: bool,
) -> DefaultResult<()> {
    // First, check whether they have `node` installed
    match Command::new("node")
        .args(&["--version"])
        .stdout(Stdio::null())
        .status()
    {
        Ok(_) => {}
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    println!("This command requires the `node` and `npm` commands.");
                    println!("The built in test suite utilizes nodejs.");
                    println!("Other methods of testing Zomes will be integrated in the future.");
                    println!(
                        "Visit https://nodejs.org to install node and npm (which comes with node)."
                    );
                    println!("Once installed, retry this command.");
                    // early exit with Ok, since this is the graceful exit
                    return Ok(());
                }
                // convert from a std::io::Error into a failure::Error
                // and actually return that error since it's something
                // different than just not finding `node`
                _ => return Err(Error::from(e)),
            }
        }
    };

    if !skip_build {
        // build the package file, within the dist folder
        let file_path = util::std_package_path(path)?;
        println!(
            "{} files for testing to file: {:?}",
            "Packaging".green().bold(),
            &file_path
        );
        package(true, PathBuf::from(file_path))?;
    }

    // build tests
    let tests_path = path.join(&tests_folder);
    ensure!(
        tests_path.exists(),
        "Directory {} does not exist",
        tests_folder
    );

    // npm install, if no node_modules yet
    let node_modules_path = tests_path.join("node_modules");
    if !node_modules_path.exists() {
        // CLI feedback
        println!("{}", "Installing node_modules".green().bold());
        util::run_cmd(
            tests_path.clone(),
            "npm".to_string(),
            &["install", "--silent"],
        )?;
    }

    // execute the built test file using node
    // CLI feedback
    println!("{} tests in {}", "Running".green().bold(), testfile,);
    util::run_cmd(
        path.to_path_buf(),
        "node".to_string(),
        &[testfile.to_string().as_str()],
    )?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    #[test]
    // flagged as broken for:
    // 1. taking 60+ seconds
    // NOTE, before re-enabling make sure to add an environment variable
    // HC_SCAFFOLD_VERSION='branch="develop"' when you run the test.
    #[cfg(feature = "broken-tests")]
    fn test_command_basic_test() {
        let temp_dir = gen_dir();
        let temp_dir_path = temp_dir.path();
        let temp_dir_path_buf = temp_dir_path.to_path_buf();

        let mut gen_cmd = Command::main_binary().unwrap();

        let _ = init(&temp_dir_path_buf);

        assert!(env::set_current_dir(&temp_dir_path).is_ok());

        // do gen my_zome first, so there's a zome
        gen_cmd
            .args(&["generate", "zomes/my_zome"])
            .assert()
            .success();

        test(&temp_dir_path_buf, &TEST_DIR_NAME, "test/index.js", false)
            .unwrap_or_else(|e| panic!("test call failed: {}", e));

        // check success of packaging step
        assert!(temp_dir_path_buf
            .join(&DIST_DIR_NAME)
            .join(package::DEFAULT_BUNDLE_FILE_NAME)
            .exists());

        // check success of npm install step
        assert!(temp_dir_path_buf
            .join(&TEST_DIR_NAME)
            .join("node_modules")
            .exists());
    }

    // TODO: this test is non-deterministic, pivoting around the fact that the
    // behaviour of the command is different, depending whether nodejs is installed on the system or not
    #[test]
    #[cfg(not(windows))]
    fn test_command_no_test_folder() {
        let temp_dir = gen_dir();
        let temp_dir_path = temp_dir.path();
        let temp_dir_path_buf = temp_dir_path.to_path_buf();

        let _ = init(&temp_dir_path_buf);

        let result = test(&temp_dir_path_buf, "west", "test/index.js", false);

        // should err because "west" directory doesn't exist
        assert!(result.is_err());
    }
}
