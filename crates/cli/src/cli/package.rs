use crate::{config_files::Build, error::DefaultResult, util};
use base64;
use colored::*;
use holochain_core::nucleus::ribosome::{run_dna, WasmCallData};
use holochain_core_types::dna::Dna;
use holochain_json_api::json::JsonString;
use holochain_persistence_api::cas::content::AddressableContent;
use ignore::WalkBuilder;
use json_patch::merge;
use serde_json::{self, Map, Value};
use std::{
    convert::TryFrom,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

use holochain_core_types::hdk_version::{HDKVersion, HDK_VERSION};

pub const BUILD_CONFIG_FILE_NAME: &str = ".hcbuild";

pub const GITIGNORE_FILE_NAME: &str = ".gitignore";

pub const IGNORE_FILE_NAME: &str = ".hcignore";

pub const WASM_FILE_EXTENSION: &str = "wasm";

pub const META_FILE_ID: &str = "file";
pub const META_DIR_ID: &str = "dir";
pub const META_BIN_ID: &str = "bin";

pub const META_SECTION_NAME: &str = "__META__";
pub const META_TREE_SECTION_NAME: &str = "tree";
pub const META_CONFIG_SECTION_NAME: &str = "config_file";

const CARGO_FILE_NAME: &str = "Cargo.toml";

pub type Object = Map<String, Value>;

fn hdk_version_compare(hdk_version: &HDKVersion, cargo_toml: &str) -> DefaultResult<bool> {
    let toml: Value = toml::from_str(cargo_toml)?;
    let dependancies = toml
        .get("dependencies")
        .ok_or_else(|| format_err!("Could not get dependencies"))?;
    let hdk = dependancies
        .get("hdk")
        .ok_or_else(|| format_err!("Could not get HDK"))?;
    let tag = hdk
        .get("tag")
        .ok_or_else(|| format_err!("Could not get HDK tag"))?
        .as_str()
        .ok_or_else(|| format_err!("Could not parse string"))?;
    let hdk_version_from_toml = HDKVersion::new(tag)?;
    Ok(hdk_version == &hdk_version_from_toml)
}

struct Packager {
    include_meta: bool,
}

impl Packager {
    fn new(include_meta: bool) -> Packager {
        Packager { include_meta }
    }

    pub fn package(include_meta: bool, output: PathBuf, properties: Value) -> DefaultResult<()> {
        // First, check whether they have `cargo` installed, since it will be needed for packaging
        // TODO: in the future, don't check for this here, since other build tools and languages
        // could be used
        let should_continue = util::check_for_cargo(
            "Compiling a Rust based Zome to WASM depends on having Rust installed.",
            Some(vec![
                "Compiling to WASM also requires adding WASM as a compile target.",
                "Make sure to be running inside a nix-shell or from a nix-env installation.",
                "See https://docs.holochain.love for more information.",
            ]),
        )?;
        if !should_continue {
            // early exit, but user will have received feedback within check_for_cargo about why
            return Ok(());
        }

        Packager::new(include_meta).run(&output, properties)
    }

    fn run(&self, output: &PathBuf, mut properties: Value) -> DefaultResult<()> {
        let current_dir = std::env::current_dir()?;
        let dir_obj_bundle = Value::from(
            self.bundle_recurse(&current_dir)
                .map(|mut val| {
                    if let Some(props_from_dir) = val.get("properties") {
                        merge(&mut properties, props_from_dir);
                    }
                    val.insert("properties".to_string(), properties);
                    val
                })
                .map_err(|e| {
                    format_err!(
                        "Couldn't traverse DNA in directory {:?}: {}",
                        &current_dir,
                        e
                    )
                })?,
        );

        let dna_str =
            serde_json::to_string_pretty(&dir_obj_bundle).expect("failed to make pretty DNA");
        let dna_json = JsonString::from_json(&dna_str);

        let dna = Dna::try_from(dna_json).map_err(|e| {
            format_err!(
                "Couldn't create a DNA from the bundle, got error {}\nJSON bundle was:\n {}",
                e,
                &dna_str
            )
        })?;

        let out_file = File::create(&output)
            .map_err(|e| format_err!("Couldn't create DNA output file {:?}; {}", output, e))?;

        serde_json::to_writer_pretty(&out_file, &(dir_obj_bundle))?;

        // CLI feedback
        println!(
            "{} DNA package file at {:?}",
            "Created".green().bold(),
            output
        );
        println!("DNA hash: {}", dna.address());

        Ok(())
    }

    fn bundle_recurse(&self, path: &PathBuf) -> DefaultResult<Object> {
        let root_dir = WalkBuilder::new(path)
            .max_depth(Some(1))
            .add_custom_ignore_filename(IGNORE_FILE_NAME)
            .build()
            .skip(1);

        let root: Vec<_> = root_dir
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .collect();

        let root_json_files: Vec<&PathBuf> = root
            .iter()
            .filter(|e| e.is_file())
            .filter(|e| e.to_string_lossy().ends_with(".json"))
            .collect();

        let mut meta_section = Object::new();

        let maybe_json_file_path = match root_json_files.len() {
            0 => {
                // A root json file is optional so can still package the dna
                None
            }
            1 => Some(root_json_files[0]),
            _ => {
                // more than one .json file is ambiguous so present an error
                return Err (format_err!("Error Packaging DNA: Multiple files with extension .json were found in the root of the project, {:?}.\
                    This is ambiguous as the packager is unable to tell which should be used as the base for the .dna.json", root_json_files)
                );
            }
        };

        // Scan files but discard found json file
        let all_nodes = root.iter().filter(|node_path| {
            maybe_json_file_path
                .and_then(|path| Some(node_path != &path))
                .unwrap_or(true)
        });

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
                // a folder within this folder has a .hcbuild in it, meaning this node
                // should build the json and insert it for this zome
                if let Some(dir_with_code) = node
                    .read_dir()?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|path| path.is_dir())
                    .find(|path| path.join(BUILD_CONFIG_FILE_NAME).exists())
                {
                    meta_tree.insert(file_name.clone(), META_DIR_ID.into());

                    let build = Build::from_file(dir_with_code.join(BUILD_CONFIG_FILE_NAME))?;
                    let wasm = build.run(&dir_with_code)?;
                    let wasm_binary = Arc::new(base64::decode(&wasm)?);

                    let json_string = run_dna(
                        Some("{}".as_bytes().to_vec()),
                        WasmCallData::DirectCall(
                            "__hdk_get_json_definition".to_string(),
                            wasm_binary.clone(),
                        ),
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
                // this is the code folder itself, with a .hcbuild file in it
                } else if let Some(build_config) = node
                    .read_dir()?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .find(|path| path.ends_with(BUILD_CONFIG_FILE_NAME))
                {
                    meta_tree.insert(file_name.clone(), META_BIN_ID.into());

                    let build = Build::from_file(build_config)?;
                    if build.steps.iter().any(|s| s.command == "cargo") {
                        let directories = node
                            .read_dir()?
                            .collect::<Result<Vec<_>, _>>()
                            .unwrap_or_default();

                        directories
                        .iter()
                        .map(|p|p.path())
                        .filter(|path| path.ends_with(CARGO_FILE_NAME))
                        .for_each(|read_path|{

                            File::open(read_path.clone())
                            .map(|mut read_file|{
                                let mut contents = String::new();
                                read_file
                                .read_to_string(&mut contents)
                                .map(|_|{
                                    hdk_version_compare(&HDK_VERSION,&*contents)
                                    .map(|hdk_match|{
                                        if let false = hdk_match
                                        {
                                            eprintln!("WARNING: The HDK version found in {:?} does not match the current version.\n If you are seeing compilation problems, update the version in your Cargo.toml files to the current version: {}", read_path, HDK_VERSION.to_string())
                                        }
                                    }).unwrap_or_default()
                                }).unwrap_or_else(|_|eprintln!("Could not read hdk from zome file and cannnot verify mismatch."))
                            }).unwrap_or_else(|_|eprintln!("Could not open zome file and cannnot verify mismatch, check if cargo toml is in use"))

                        });
                    }

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

        if self.include_meta {
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

pub fn package(
    strip_meta: bool,
    output: PathBuf,
    properties: serde_json::Value,
) -> DefaultResult<()> {
    Packager::package(strip_meta, output, properties)
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

                            let file_path = to.join(meta_entry);

                            File::create(file_path)?.write_all(&content[..])?;
                        }
                        META_BIN_ID if entry.is_object() => {
                            let base64_content = entry[&meta_entry].to_string();
                            let content = base64::decode(&base64_content)?;

                            let file_path = to.join(meta_entry).with_extension(WASM_FILE_EXTENSION);

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
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "broken-tests")]
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
    fn hdk_version_compare_test() {
        //compare same
        let hdk_version = HDKVersion::new("99.99.99-alpha99").expect("cannot create hdk version");
        assert!(hdk_version_compare(
            &hdk_version,
            r#"
        name = 'stuff'

        [dependencies]
        hdk = {github='xxx', tag='99.99.99-alpha99'}

    "#
        )
        .expect("Could not compare"));

        let hdk_version = HDKVersion::new("99.99.99-alpha99").expect("cannot create hdk version");
        assert!(!hdk_version_compare(
            &hdk_version,
            r#"
        name = 'stuff'

        [dependencies]
        hdk = {github='xxx', tag='0.0.0-alpha1'}

    "#
        )
        .expect("Could not compare"))
    }

    #[test]
    #[cfg(feature = "broken-tests")]
    fn aborts_if_multiple_json_in_root() {
        let shared_space = gen_dir();

        let root_path = shared_space.path().to_path_buf();

        fs::create_dir_all(&root_path).unwrap();

        // Initialize and package a project
        Command::main_binary()
            .unwrap()
            .args(&["init", root_path.to_str().unwrap()])
            .assert()
            .success();

        // copy the json
        fs::copy(root_path.join("app.json"), root_path.join("app2.json")).unwrap();

        // ensure the package command fails
        Command::main_binary()
            .unwrap()
            .args(&["package"])
            .current_dir(&root_path)
            .assert()
            .failure();
    }

    #[test]
    /// A test ensuring that packaging and unpacking a project results in the very same project
    #[cfg(feature = "broken-tests")]
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
    #[cfg(feature = "broken-tests")]
    fn auto_compilation() {
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
            .current_dir(&shared_space.path())
            .args(&["package"])
            .assert()
            .success();
    }
}
