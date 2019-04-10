# Intro to Language HDKs

Within any Zome, there are a number of conventions that must be followed by the WASM code, in order to work with Holochain core. For example parameters are passed using a specific memory allocation scheme that needs to be followed to access the parameters that a Zome function receives.  Although it would be possible for Zome authors to code directly to the Holochain core API, it makes more sense to provide a software library for each language to make it as easy as possible to use those standard functions and behaviours. We call such a library a "Holochain Development Kit", or HDK.

So in order to get familiar with coding for Holochain, it will involve familiarity with an HDK library.

An HDK performs many important functions for developers.
- It aids with the memory management of the WASM code that your Zomes compile into. WASM has some strict memory limitations, like 64 KiB paged memory.
- It creates helper functions that hide tedious semantics that seem to plague compilation of languages to WASM.
- It implements a complete type system that's compatible with Holochain.
- It addresses hidden functions that Holochain needs from Zome code, but that would be redundant and confusing to make Zome developers write again and again.

The HDK for a given language, if it has deep integration into the build tools, such as the Zome generator, should include this helper library by default, which is the case of the Rust HDK.

There is an in-depth article on [writing an HDK](../writing_development_kit.md) if this sounds interesting to you.

### The Rust HDK

The HDK with priority development and support is implemented in Rust, and is included right in the [core repository](https://github.com/holochain/holochain-rust) along with the Holochain core framework. Other HDKs may be implemented in different languages, and exist in separate repositories. This HDK implements all of the above features for developers, so just know that as you develop your Zome, a lot is going on behind the scenes in the HDK to make it all work.

The Rust HDK has documentation for each released version, available at [developer.holochain.org/api](https://developer.holochain.org/api/). This documentation will be invaluable during your use of the HDK, because it can show you how the definitions of the various custom Holochain types, and give examples and details on the use of the API functions exposed to the Zomes.

Notice that in the `Cargo.toml` file of a new Zome, the HDK is included. For example,

```toml
...
[dependencies]
...
hdk = { git = "https://github.com/holochain/holochain-rust", branch = "master" }
...
```

#### Setting the version of the HDK

Once Holochain stabilizes beyond the 0.0.x version numbers, it will be published to the Rust package manager, [crates.io](https://crates.io) and versioning will be simplified. For now, Cargo installs the HDK specified as a GIT dependency, fetching it from the [specified commit, branch, or tag of the repository](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories).

If you wanted to lock the HDK at a specific version, you could adjust the HDK dependency like this:
```toml
hdk = { git = "https://github.com/holochain/holochain-rust", tag = "v0.0.7-alpha" }
```

#### Use of the HDK in Rust code

Notice now that within the `src/lib.rs` file of a new Zome, the HDK is already imported here too:

```rust
#[macro_use]
extern crate hdk;
...
```

The `#[macro_use]` statement on the first line is very important, since it allows the usage of Rust macros (discussed in the [Define Zome article](./define_zome.md)) defined in the HDK to be used in your code, and the macros will be needed. Now, within the Rust code files, exposed constants, functions, and even special macros from the HDK can be used.

The very first thing to familiarize with is the `define_zome!` macro. Read on to learn about it.
