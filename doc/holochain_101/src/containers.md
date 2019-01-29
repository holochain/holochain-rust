# Containers

It is useful to zoom out for a moment to the level of how Holochain runs on devices. Because there was an intention to make Holochain highly platform and system compatible, the core logic was written in such a way that it could be included into many different codebases. Think MacOSX, Linux, Windows, Android, iOS, and more. Thus Holochain core is actually simply a library that needs to be included in another project which mounts, executes and manages it. Because filling this new need is becoming such a foundational aspect of Holochain, it has its' own name: *Container*.

Containers install and uninstall, start and stop instances of DNA on devices. They also create a channel to securely make function calls into the Zome functions of DNA instances.

Containers can implement whatever interfaces to perform these function calls they wish to, opening a wealth of opportunity. With the Rust built binary Container, interfaces for making function calls already includes HTTP and WebSockets.

Holochain provides two reference Containers, one for [Nodejs](https://www.npmjs.com/package/@holochain/holochain-nodejs), and the other a [Rust built binary executable](https://github.com/holochain/holochain-rust/tree/develop/container).

holochain_container

holochain-nodejs

containers wrap container_api

dynamic configuration of the container via admin level RPC