# Building Holochain Apps: Zome Code

Recall that for the DNA of a hApp, there can be many Zomes, and each one will have their own source code. Think of Zomes as the fundamental unit of composability for DNA.  As a DNA developer you can think of Zomes as modules. We expect developers to reuse Zomes written by others, and thus Zomes can call one another's functionality, using the `call` API function. Though currently Rust is the only available language for writing Zomes, note that these Zomes could be written in different languages (any language that compiles to WebAssembly) from one another in the future, and still access one another's functionality.

While writing the source code for DNA, it is extremely important to verify, before putting it into people's hands, that the code works as expected. For this reason, there are tools for testing included by default in newly generated projects. While there are technically a variety of ways that testing could be accomplished, and you could build your own, the most accessible of those is included by default, which is a JavaScript/nodejs Holochain Conductor. What this means is that the full scope of writing DNA, as of this writing, is likely for most people to include source code in two languages:
- Rust
- JavaScript

In the near future, this is likely to expand in diversity on both sides, Zome code and testing code.

Throughout this chapter, there will be plenty of examples given as to writing Zome code in Rust.

This chapter starts with a step by step tutorial, and goes on with a detailed explanation of the affordances of Holochain, and how to use its core functions.
