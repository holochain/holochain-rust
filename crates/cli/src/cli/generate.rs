use crate::error::DefaultResult;
use glob::glob;
use std::{fs, io::prelude::*, path::PathBuf};
use std::io::copy;
use std::fs::File;
use tempfile::Builder;
use tera::{Context, Tera};
use flate2::read::GzDecoder;
use tar::Archive;
// use std::path::Path;

// const RUST_TEMPLATE_REPO_URL: &str = "https://github.com/holochain/rust-zome-template";
// const RUST_PROC_TEMPLATE_REPO_URL: &str = "https://github.com/holochain/rust-proc-zome-template";
const RUST_TEMPLATE_TARBALL_URL: &str = "https://github.com/holochain/rust-zome-template/archive/master.tar.gz";
const RUST_PROC_TEMPLATE_TARBALL_URL: &str = "https://github.com/holochain/rust-proc-zome-template/archive/master.tar.gz";

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
        "rust" => RUST_TEMPLATE_TARBALL_URL,
        "rust-proc" => RUST_PROC_TEMPLATE_TARBALL_URL,
        _ => scaffold, // if not a known type assume that a repo url was passed
    };

    println!("foo!");

    // https://rust-lang-nursery.github.io/rust-cookbook/web/clients/download.html
    let tmp_dir = Builder::new().prefix("example").tempdir()?;
    let mut response = reqwest::get(url)?;

    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.bin");

    println!("file to download: '{}'", fname);
    let fname = tmp_dir.path().join(fname);
    println!("will be located under: '{:?}'", fname);
    let mut dest = File::create(&fname)?;
    copy(&mut response, &mut dest)?;

    println!("foo! {:?}", dest);

    // https://rust-lang-nursery.github.io/rust-cookbook/compression/tar.html

    let tar_gz = File::open(fname)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.entries()?
    .filter_map(|e| e.ok())
    .map(|mut entry| -> DefaultResult<PathBuf> {
        let path = entry.path()?.strip_prefix(entry.path()?.components().nth(0).unwrap())?.to_owned();
        entry.unpack(&path)?;
        Ok(path)
    })
    .filter_map(|e| e.ok())
    .for_each(|x| println!("> {}", x.display()));
    // for e in archive.entries()? {
    //     let mut entry = e?;
    //     let path_cow = entry.path()?.to_owned();
    //     let path_str = path_cow.to_str().unwrap();
    //     let path = Path::new(path_str.clone());
    //     let stripped_path = path.strip_prefix(path.components().nth(0).unwrap())?;
    //     println!("{:?}", stripped_path);
    //     entry.unpack(&stripped_path)?;
    // }
    // archive.unpack(zome_path)?;

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
