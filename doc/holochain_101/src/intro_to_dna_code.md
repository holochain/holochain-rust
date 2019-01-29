# Intro to DNA: Code

The functionality of Holochain applications is written as a collection of logical modules called "Zomes".

Zomes are created inside a folder called `zomes`, and each Zome should have its own sub-folder within that, in which the configuration and code for that particular Zome should be placed.

These Zomes can call and access the functionality of the others, but they are written independently. 

When the DNA file is being packaged, the code for these Zomes is encoded using Base64 encoding and combined with the configuration file associated with the Zome.

The configuration file should be a JSON file, stored in the Zome folder. The file can be named anything, but the default is `zome.json`.

This Zome file is extremely simplistic at this point, and contains only a `description` property, which is a human readable property that describes what the Zome is for.

The only coding language that Holochain knows how to execute is WebAssembly. However, it is unlikely that you'll want to write WebAssembly code by hand. Instead, most people will write their Zomes' code in a language that can compile to WebAssembly, such as Rust or Assemblyscript, and then define a build step in which it is compiled to WebAssembly. There is already a large, and growing, number of languages that compile to WebAssembly.

If this is sounding complex, don't worry. There are tools supplied to make this easy, and you'll be writing in a language that's familiar, or easy to learn.

With this overview in mind, the details of app development can be explored.
