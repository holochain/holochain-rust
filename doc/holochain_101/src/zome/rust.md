# Writing in Rust

It has always been in the designs for Holochain to support programming in multiple languages. In the prototype of Holochain, Zomes could be written in variants of Javascript (ES5) and Lisp (Zygomys). In the new version of Holochain the primary "Ribosome", where there was a JS one and Lisp one before, interprets WebAssembly code.

While we will provide a small introduction to WebAssembly shortly, we should also briefly introduce Rust, since it is the first language that has a first class Holochain app development experience, with its' WebAssembly compilation. This is accomplished via a Holochain Development Kit (HDK) library which has been written for Rust.

For the time being, writing Holochain apps requires writing code in Rust. This will not always be the case. [Assemblyscript](https://github.com/AssemblyScript/assemblyscript), a language based off Typescript, is the next likely language in which there will be an HDK library. If it happens to interest you, there is an article here about [writing an HDK](../writing_development_kit.md), since that is something we also invite and encourage the community to do.

From Wikipedia: 
"Rust is a systems programming language with a focus on safety, especially safe concurrency, supporting both functional and imperative paradigms."

Rust is a strongly typed language, which is desirable for the development of secure P2P applications, and compilation from Rust to WebAssembly is extremely easy.

If Rust is new to you, don't worry. With lots of Holochain app development happening in an open source way, and through learning resources like this guidebook, and the "Rust book" you will have lots to reference to get started.

While there are lots of other materials available for learning Rust, the base materials for the language are always a good resource to go back to: [Rust Docs](https://doc.rust-lang.org/).

In many articles in the following chapters, you will find that there is a dedicated section at the bottom of the article called "Building in Rust". This typically follows after a general discussion of a concept, and is done this way because implementation details may differ based on the language developers are writing Zomes in.



