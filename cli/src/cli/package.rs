use crate::{config_files::Build, error::DefaultResult, util};
use base64;
use colored::*;
use holochain_core::nucleus::ribosome::{run_dna, WasmCallData};
use ignore::WalkBuilder;
use serde_json::{self, Map, Value};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

pub const CODE_DIR_NAME: &str = "code";

pub const BUILD_CONFIG_FILE_NAME: &str = ".build";

pub const GITIGNORE_FILE_NAME: &str = ".gitignore";

pub const IGNORE_FILE_NAME: &str = ".hcignore";

pub const WASM_FILE_EXTENSION: &str = "wasm";

pub const META_FILE_ID: &str = "file";
pub const META_DIR_ID: &str = "dir";
pub const META_BIN_ID: &str = "bin";

pub const META_SECTION_NAME: &str = "__META__";
pub const META_TREE_SECTION_NAME: &str = "tree";
pub const META_CONFIG_SECTION_NAME: &str = "config_file";

pub type Object = Map<String, Value>;

struct Packager {
    strip_meta: bool,
}

impl Packager {
    fn new(strip_meta: bool) -> Packager {
        Packager { strip_meta }
    }

    pub fn package(strip_meta: bool, output: PathBuf) -> DefaultResult<()> {
        // First, check whether they have `cargo` installed, since it will be needed for packaging
        // TODO: in the future, don't check for this here, since other build tools and languages
        // could be used
        let should_continue = util::check_for_cargo(
            "Compiling a Rust based Zome to WASM depends on having Rust installed.",
            Some(vec![
                "Compiling to WASM also requires adding WASM as a compile target.",
                "For this, also run:",
                "$ rustup target add wasm32-unknown-unknown --toolchain nightly-2019-01-24",
            ]),
        )?;
        if !should_continue {
            // early exit, but user will have received feedback within check_for_cargo about why
            return Ok(());
        }

        Packager::new(strip_meta).run(&output)
    }

    fn run(&self, output: &PathBuf) -> DefaultResult<()> {
        let dir_obj_bundle = self.bundle_recurse(&std::env::current_dir()?)?;

        let out_file = File::create(&output)?;

        serde_json::to_writer_pretty(&out_file, &Value::from(dir_obj_bundle))?;

        // CLI feedback
        println!(
            "{} dna package file at {:?}",
            "Created".green().bold(),
            output
        );

        Ok(())
    }

    fn bundle_recurse(&self, path: &PathBuf) -> DefaultResult<Object> {
        let root_dir = WalkBuilder::new(path)
            .max_depth(Some(1))
            .add_custom_ignore_filename(IGNORE_FILE_NAME)
            .build()
            .skip(1);

        let root: Vec<_> = root_dir
            .filter(|e| e.is_ok())
            // unwrap safe here due to is_ok() filter above
            .map(|e| e.unwrap().path().to_path_buf())
            .collect();

        let maybe_json_file_path = root
            .iter()
            .filter(|e| e.is_file())
            .find(|e| e.to_string_lossy().ends_with(".json"));

        // Scan files but discard found json file
        let all_nodes = root.iter().filter(|node_path| {
            maybe_json_file_path
                .and_then(|path| Some(node_path != &path))
                .unwrap_or(true)
        });

        let mut meta_section = Object::new();

        // Obtain the config file
        let mut main_tree: Object = if let Some(json_file_path) = maybe_json_file_path {
            let file_name = util::file_name_string(&json_file_path)?;

            meta_section.insert(
                META_CONFIG_SECTION_NAME.into(),
                Value::String(file_name.clone()),
            );

            let json_file = fs::read_to_string(json_file_path)?;

            // if the json file does not contain an Object at the top level, we can't parse it
            serde_json::from_str(&json_file).unwrap_or_default()
        } else {
            Object::new()
        };

        // Let's go meta. Way meta!
        let mut meta_tree = Object::new();

        for node in all_nodes {
            let file_name = util::file_name_string(&node)?;

            // ignore empty main_tree, which results from an unparseable JSON file
            if node.is_file() && !main_tree.is_empty() {
                meta_tree.insert(file_name.clone(), META_FILE_ID.into());

                let mut buf = Vec::new();
                File::open(node)?.read_to_end(&mut buf)?;
                let encoded_content = base64::encode(&buf);

                main_tree.insert(file_name.clone(), encoded_content.into());
            } else if node.is_dir() {
                // a folder within this folder has a .build in it, meaning this node
                // should build the json and insert it for this zome
                if let Some(dir_with_code) = node
                    .read_dir()?
                    .filter(|e| e.is_ok())
                    .map(|e| e.unwrap().path())
                    .filter(|path| path.is_dir())
                    .find(|path| path.join(BUILD_CONFIG_FILE_NAME).exists())
                {
                    meta_tree.insert(file_name.clone(), META_DIR_ID.into());

                    let build = Build::from_file(dir_with_code.join(BUILD_CONFIG_FILE_NAME))?;

                    let wasm = build.run(&dir_with_code)?;

                    let wasm_binary = base64::decode(&wasm)?;

                    let json_string = run_dna(
                        wasm_binary,
                        Some("{}".as_bytes().to_vec()),
                        WasmCallData::DirectCall("__hdk_get_json_definition".to_string()),
                    )?;

                    let json_from_wasm: Map<String, Value> =
                        serde_json::from_str(&String::from(json_string))?;

                    let mut sub_tree_content = self.bundle_recurse(&node)?;
                    for key in json_from_wasm.keys() {
                        sub_tree_content
                            .insert(key.clone(), json_from_wasm.get(key).unwrap().clone());
                    }

                    // here insert json generated by the wasm, alongside the rest of the sub-tree
                    main_tree.insert(file_name.clone(), sub_tree_content.into());
                // this is the code folder itself, with a .build file in it
                } else if let Some(build_config) = node
                    .read_dir()?
                    .filter(|e| e.is_ok())
                    .map(|e| e.unwrap().path())
                    .find(|path| path.ends_with(BUILD_CONFIG_FILE_NAME))
                {
                    meta_tree.insert(file_name.clone(), META_BIN_ID.into());

                    let build = Build::from_file(build_config)?;

                    let wasm = build.run(&node)?;

                    // here insert the wasm itself
                    main_tree.insert(file_name.clone(), json!({ "code": wasm }));
                } else {
                    meta_tree.insert(file_name.clone(), META_DIR_ID.into());

                    let sub_tree_content = self.bundle_recurse(&node)?;

                    main_tree.insert(file_name.clone(), sub_tree_content.into());
                }
            }
        }

        if !self.strip_meta {
            if !meta_tree.is_empty() {
                meta_section.insert(META_TREE_SECTION_NAME.into(), meta_tree.into());
            }

            if !meta_section.is_empty() {
                main_tree.insert(META_SECTION_NAME.into(), meta_section.into());
            }
        }

        Ok(main_tree)
    }
}

pub fn package(strip_meta: bool, output: PathBuf) -> DefaultResult<()> {
    Packager::package(strip_meta, output)
}

pub fn unpack(path: &PathBuf, to: &PathBuf) -> DefaultResult<()> {
    ensure!(path.is_file(), "argument \"path\" doesn't point to a file");

    if !to.exists() {
        fs::create_dir_all(&to)?;
    }

    ensure!(to.is_dir(), "argument \"to\" doesn't point to a directory");

    let raw_bundle_content = fs::read_to_string(&path)?;
    let bundle_content: Object = serde_json::from_str(&raw_bundle_content)?;

    unpack_recurse(bundle_content, &to)?;

    Ok(())
}

fn unpack_recurse(mut obj: Object, to: &PathBuf) -> DefaultResult<()> {
    if let Some(Value::Object(mut main_meta_obj)) = obj.remove(META_SECTION_NAME) {
        // unpack the tree
        if let Some(Value::Object(tree_meta_obj)) = main_meta_obj.remove(META_TREE_SECTION_NAME) {
            for (meta_entry, meta_value) in tree_meta_obj {
                let entry = obj
                    .remove(&meta_entry)
                    .ok_or_else(|| format_err!("incompatible meta section"))?;

                if let Value::String(node_type) = meta_value {
                    match node_type.as_str() {
                        META_FILE_ID if entry.is_string() => {
                            let base64_content = entry.as_str().unwrap().to_string();
                            let content = base64::decode(&base64_content)?;

                            let mut file_path = to.join(meta_entry);

                            File::create(file_path)?.write_all(&content[..])?;
                        }
                        META_BIN_ID if entry.is_object() => {
                            let base64_content = entry[&meta_entry].to_string();
                            let content = base64::decode(&base64_content)?;

                            let mut file_path =
                                to.join(meta_entry).with_extension(WASM_FILE_EXTENSION);

                            File::create(file_path)?.write_all(&content[..])?;
                        }
                        META_DIR_ID if entry.is_object() => {
                            let directory_obj = entry.as_object().unwrap();
                            let dir_path = to.join(meta_entry);

                            fs::create_dir(&dir_path)?;

                            unpack_recurse(directory_obj.clone(), &dir_path)?;
                        }
                        _ => bail!("incompatible meta section"),
                    }
                } else {
                    bail!("incompatible meta section");
                }
            }
        }

        // unpack the config file
        if let Some(config_file_meta) = main_meta_obj.remove(META_CONFIG_SECTION_NAME) {
            ensure!(
                config_file_meta.is_string(),
                "config file has to be a string"
            );

            if !obj.is_empty() {
                let dna_file = File::create(to.join(config_file_meta.as_str().unwrap()))?;
                serde_json::to_writer_pretty(dna_file, &obj)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
// too slow!
#[cfg(feature = "broken-tests")]
mod tests {
    use crate::cli::init::tests::gen_dir;
    use assert_cmd::prelude::*;
    use std::process::Command;

    #[test]
    fn package_and_unpack_isolated() {
        const TEST_DNA_FILE_NAME: &str = "test.dna.json";

        fn package(shared_file_path: &PathBuf) {
            let temp_space = gen_dir();
            let temp_dir_path = temp_space.path();

            Command::main_binary()
                .unwrap()
                .args(&["init", temp_dir_path.to_str().unwrap()])
                .assert()
                .success();

            let bundle_file_path = shared_file_path.join(TEST_DNA_FILE_NAME);

            Command::main_binary()
                .unwrap()
                .args(&["package", "-o", bundle_file_path.to_str().unwrap()])
                .current_dir(&temp_dir_path)
                .assert()
                .success();
        }

        fn unpack(shared_file_path: &PathBuf) {
            let temp_space = gen_dir();
            let temp_dir_path = temp_space.path();

            Command::main_binary()
                .unwrap()
                .current_dir(&shared_file_path)
                .args(&[
                    "unpack",
                    TEST_DNA_FILE_NAME,
                    temp_dir_path.to_str().unwrap(),
                ])
                .assert()
                .success();
        }

        let shared_space = gen_dir();

        package(&shared_space.path().to_path_buf());

        unpack(&shared_space.path().to_path_buf());

        shared_space.close().unwrap();
    }

    #[test]
    /// A test ensuring that packaging and unpacking a project results in the very same project
    fn package_reverse() {
        const TEST_DNA_FILE_NAME: &str = "test.dna.json";

        const SOURCE_DIR_NAME: &str = "source_app";
        const DEST_DIR_NAME: &str = "dest_app";

        let shared_space = gen_dir();

        let root_path = shared_space.path().to_path_buf();

        let source_path = shared_space.path().join(SOURCE_DIR_NAME);
        fs::create_dir_all(&source_path).unwrap();

        // Initialize and package a project
        Command::main_binary()
            .unwrap()
            .args(&["init", source_path.to_str().unwrap()])
            .assert()
            .success();

        let bundle_file_path = root_path.join(TEST_DNA_FILE_NAME);

        Command::main_binary()
            .unwrap()
            .args(&["package", "-o", bundle_file_path.to_str().unwrap()])
            .current_dir(&source_path)
            .assert()
            .success();

        // Unpack the project from the generated bundle
        let dest_path = shared_space.path().join(DEST_DIR_NAME);
        fs::create_dir_all(&dest_path).unwrap();

        Command::main_binary()
            .unwrap()
            .args(&[
                "unpack",
                bundle_file_path.to_str().unwrap(),
                dest_path.to_str().unwrap(),
            ])
            .assert()
            .success();

        // Assert for equality
        // TODO add .hcignore file itself to the bundle so that source and dest folders are the same
        // @see https://github.com/holochain/holochain-cli/issues/38
        // assert!(!dir_diff::is_different(&source_path, &dest_path).unwrap());
    }

    #[test]
    fn auto_compilation() {
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
        //
        // Command::main_binary()
        //     .unwrap()
        //     .current_dir(&tmp.path())
        //     .args(&["g", "zomes/zubblebat", "assemblyscript"])
        //     .assert()
        //     .success();

        Command::main_binary()
            .unwrap()
            .current_dir(&tmp.path())
            .args(&["package"])
            .assert()
            .success();
    }
}
