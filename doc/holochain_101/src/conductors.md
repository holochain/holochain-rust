# Running Holochain Apps: Conductors

To introduce Conductors, it is useful to zoom out for a moment to the level of how Holochain runs on devices.

Because Holochain is intended to be highly platform and system compatible, the core logic is written in such a way that it can be included into many different codebases. Think of MacOSX, Linux, Windows, Android, iOS, and more. Thus, the core of Holochain was made to be simply a library that gets included in another wrapper which mounts, executes and manages it. Because filling this new need is such a foundational aspect of Holochain, it has its' own name: *Conductor*.

Conductors play quite a number of important roles:
- installing, uninstalling, starting and stopping instances of DNA
- exposing APIs to securely make function calls into the Zome functions of DNA instances
- accepting information concerning the cryptographic keys and agent info to be used for identity and signing, and passing it into Holochain
- 


Conductors can implement a diversity of interfaces to perform these function calls, which opens an abundance of opportunity. Another way to build Holochain into an application is to use language bindings from the Rust built version of the Conductor, to another language, that then allows for the direct use of Holochain in that language.

Holochain provides two reference Conductors, one for [Nodejs](https://www.npmjs.com/package/@holochain/holochain-nodejs) (which is built using the language bindings approach), and the other a [binary executable written in Rust](https://github.com/holochain/holochain-rust/tree/develop/conductor).

## Conductor Configuration

There are lots of different ways to configure a Conductor, and many different parts that can be configured. Broadly speaking, here are the parts that can be configured for a Conductor:

1. 

Initial TOML based configuration

Configuration of...
- [[bridges]]
- [[agents]]
- [[dnas]]
- [[instances]]
- [instances.storage]
- [[interfaces]] (admin)
- [[interfaces.instances]]
- [[ui_bundles]]
- [[ui_interfaces]]
- [interfaces.driver]
- [logger]
- [network]
- [persistence dir]

dynamic configuration of the conductor via admin level RPC

Saving of that new configuration to your system

