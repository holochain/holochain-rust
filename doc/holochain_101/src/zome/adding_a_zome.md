# Adding a Zome

After [creating a new project](../new_project.md), in the resulting folder there is an empty folder called 'zomes'. The name of this folder is important and should not be changed. It will serve as the root folder for the one or more Zomes in a project.

Every Zome should have its own folder within the 'zomes' root folder, and the name of those folders is also important, it should be the name of the Zome. It shouldn't have any spaces, and it should be a valid folder name.

While you could go about creating a new Zome manually through your file system, it will be far faster to use the Holochain command line tools to generate one, with all the basic files you need included.

To do this, navigate in a command line to the root directory of your hApp project. In the command line, run
```shell
hc generate zomes/your_zome_name
```

`hc` specifies that you wish to use the Holochain command line tools. `generate` specifies to use the command for initializing a new Zome. `zomes/your_zome_name` is an argument you supply as the path to, and the name of the Zome to generate.

The output should be as follows
```shell
cargo init --lib --vcs none
Created library package
Generated new rust Zome at "zomes/your_zome_name"
```

Note that in the case of a Rust Zome, which is the only language for a Zome we can generate at the moment, it will rely internally on Rust related commands (`cargo init`), meaning that Rust (and its package manager, cargo) must already ALSO be installed for this command to work successfully.

This has created a new folder (`zomes/your_zome_name`) in which you have the beginnings of a Zome.

## What's in a Zome?

A Rust based Zome folder looks something like this:
- code
    - src
        - lib.rs
    - .hcbuild
    - Cargo.toml
- zome.json

`code` is a folder that should always exist in a Zome, and should contain either pre-compiled WASM, or the source code and instructions to generate WASM. Everything within `code` is contextual to the language the Zome is written in, in the case above, a Rust "crate". Files within `code` will be explained in detail below.

`zome.json` is the top level configuration of your Zome.

### Rust crate Zomes
As mentioned above, the files within `code` are contextual to the language the Zome is written in, and in this case, that's Rust.

As developers tend to do, Rust developers gave their own unique name to Rust projects: "crates". There are two types of Rust crates: `library` and `binary`. Since Zome code is getting compiled to WebAssembly, not standard binary executables, Zome crates use the `library` style, which is why we see under `code/src` a `lib.rs` file.

The most minimalistic library crate would look like this:

- src
    - lib.rs
- Cargo.toml

Notice that the Zome we generated has one extra file, `.hcbuild`. This is the only Holochain specific file in the `code` folder. The rest is standard Rust. The [`.hcbuild` file is discussed](../build_files.md) in another chapter.

In general, the generated files have been modified from their defaults to offer basic boilerplate needed to get started writing Zome code.

`src/lib.rs` is the default entry point to the code of a library crate. It can be the one and only Rust file of a Zome, or it can use standard Rust imports from other Rust files and their exports, taking full advantage of the Rust module system natively.

`Cargo.toml` is Rust's equivalent to nodejs' `package.json` or Ruby's `Gemfile`: a configuration and dependency specification at the same time.

Note that with the Cargo dependency system, Zome developers can take advantage of pre-existing Rust crates in their code, with one condition: that those dependencies are compatible when compiling to WebAssembly. This will be gone into in more detail elsewhere.
