# Intro to DNA: Code

Obviously, the logic and functionality of your application will be written in code. Here is a quick overview of how that works.

Holochain allows you to break down the functionality of your application into smaller, logical modules, called "Zomes". 

There should be a folder called `zomes` and each Zome should have its own sub-folder within that, in which the configuration and code for that particular Zome should be placed.

These Zomes can call and access the functionality of the others, but they are written independently. 

When the DNA file is being packaged, the code for these Zomes is encoded using Base64 encoding and combined with the configuration file associated with the Zome.

The configuration file should be a JSON file, stored in the Zome folder. The file can be named anything, but the default is `zome.json`.

This zome file is extremely simplistic at this point, and contains only a `description` property, which is a human readable property that describes what the Zome is for.

The only coding language that Holochain knows how to execute is WebAssembly. However, it is unlikely to write WebAssembly code by hand, more likely is to write code in a language that can compile to WebAssembly, such as Rust, or Assemblyscript. There is already a large, and growing, number of languages that compile to WebAssembly.

So it is likely within each Zome to store code written in another language, and to define a build step in which it is compiled to WebAssembly, and then included in your DNA.

If this is sounding complex, don't worry, you won't have to confront a lot of this complexity yourself, there are tools supplied to make this easy, and you'll be writing in a language that's familiar, or easy to learn.

With this overview in mind, the details of app development can be explored.