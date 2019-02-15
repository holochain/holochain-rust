# .build Files

In the process of building a `.dna.json` file during packaging, here is what Holochain does:
- It iterates Zome by Zome adding them to the JSON
- For each Zome, it looks for any folders containing a `.build` file
- For any folder with a `.build` file, it __executes one or more commands from the `.build` file to create a WASM file__
- It takes that built WASM file and Base64 encodes it, then stores a key/value pair for the Zome with the key as the folder name and the encoded WASM as the value

When using [`hc generate` to scaffold a Zome](./zome/adding_a_zome.md), you will have a `.build` file automatically. If you create your Zome manually however, you will need to create the file yourself. Here's the structure of a `.build` file, using a Rust Zome which builds using Cargo as an example:
```json
{
  "steps": {
    "cargo": [
      "build",
      "--release",
      "--target=wasm32-unknown-unknown"
    ]
  },
  "artifact": "target/wasm32-unknown-unknown/release/code.wasm"
}
```

The two top level properties are `steps` and `artifact`.

`steps` is a list of commands which will be sequentially executed to build a WASM file.

`artifact` is the expected path to the built WASM file.

Under `steps`, each key refers to the bin(ary) of the command that will be executed, such as `cargo`. The value of `cargo`, the command, is an array of arguments: `build`, and the two `--` flags. In order to determine what should go here, just try running the commands yourself from a terminal, while in the directory of the Zome code.

That would look, for example, like running:
```shell
cargo build --release --target=wasm32-unknown-unknown
```

## Building in Rust: Rust -> WASM compilation tools
If we take Zome code in Rust as an example, you will need Rust and Cargo set up appropriately to build WASM from Rust code. To enable it, run the following:

```shell
$ rustup target add wasm32-unknown-unknown
```

This adds WASM as a compilation target for Rust, so that you can run the previously mentioned command with `--target=wasm32-unknown-unknown`.
