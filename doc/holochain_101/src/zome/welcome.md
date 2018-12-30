# Building Holochain Apps: Zome Code

Recall that for the DNA of a hApp, there can be many Zomes, and each one will have their own source code. They are the submodules of a hApp. Zomes can call one another's functionality, using an API exposed by Holochain for doing so. Though Rust is the only supported language for writing Zomes for the short term, note that these Zomes could be written in different languages (any language that compiles to WebAssembly) from one another in the future, and still access one another's functionality.

While writing the source code for a hApp, it is extremely important to verify, before putting it into people's hands, that the code works as expected. For this reason, there are tools for testing included by default in newly generated hApps. While there are technically a variety of ways that testing could be accomplished, and you could build your own, the most accessible of those is included by default, which is a JavaScript/nodejs Holochain Container. What this means is that the full scope of writing hApps, as of this writing, is likely for most people to include source code in two languages:
- Rust
- JavaScript

In the near future, this is likely to expand in diversity on both sides, Zome code and testing code.

Throughout this chapter, there will be plenty of examples given as to writing Zome code in Rust, and test code in JavaScript. Before that though, one must know how to generate a Zome with the command line tools.
