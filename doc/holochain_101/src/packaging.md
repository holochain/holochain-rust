# Building Holochain Apps: Packaging

The `hc package` command will automate the process of compiling your Zome code, encoding it, and inserting into the `.dna.json` file. In order to get these benefits, you just need to make sure that you have the right compilation tools installed on the machine you are using the command line tools from, and that you have the proper configuration files in your Zome folders.

`hc package` works with two special files called [`.hcignore` files](./hcignore_files.md) and [`.build` files](./build_files.md).

The `hc package` tool will automate the process of compiling your Zome code, encoding it, and inserting into the `.dna.json` file. In order to get these benefits, you just need to make sure that you have the right compilation tools installed on the machine you are using the command line tools from, and that you have the proper configuration files in your Zome folders.

`hc package` works with two special files called [`.hcignore` files](./hcignore_files.md) and [`.build` files](./build_files.md).

## Building in Rust: Rust -> WASM compilation tools
If we take Zome code in Rust as an example, you will need Rust and Cargo set up appropriately to build WASM from Rust code. WASM compilation is available on the `nightly` Rust toolchain. To enable it, run the following:

```shell
$ rustup target add wasm32-unknown-unknown # adds WASM as a compilation target
```

Once that's done, you should be able to run commands like `cargo build --target=wasm32-unknown-unknown` and have it work.