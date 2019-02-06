# Running Holochain Apps: Conductors

To introduce Conductors, it is useful to zoom out for a moment to the level of how Holochain runs on devices.

Because Holochain is intended to be highly platform and system compatible, the core logic is written in such a way that it can be included into many different codebases. Think of MacOSX, Linux, Windows, Android, iOS, and more. Thus, the core of Holochain was made to be simply a library that gets included in another wrapper which mounts, executes and manages it. Because filling this new need is such a foundational aspect of Holochain, it has its' own name: *Conductor*.

Conductors play quite a number of important roles:
- installing, uninstalling, configuring, starting and stopping instances of DNA
- exposing APIs to securely make function calls into the Zome functions of DNA instances
- accepting information concerning the cryptographic keys and agent info to be used for identity and signing, and passing it into Holochain
- establishing "bridging" between DNA instances
- serving files for web based user interfaces

Those are the basic functions of a Conductor, but in addition to that, a Conductor also allows for the configuration of the networking module for Holochain, enables logging, and if you choose to, exposes APIs at a special 'admin' level that allows for the dynamic configuration of the Conductor while it runs. By default, configuration of the Conductor is done via a static configuration file, written in [TOML](https://github.com/toml-lang/toml).


In regards to the Zome functions APIs, Conductors can implement a diversity of interfaces to perform these function calls, creating an abundance of opportunity. Another way to build Holochain into an application is to use language bindings from the Rust built version of the Conductor, to another language, that then allows for the direct use of Holochain in that language.

There are currently three Conductor implementations:
- [Nodejs](https://www.npmjs.com/package/@holochain/holochain-nodejs)
    - this is built using the language bindings approach, using [neon]()
- [hc run](./development_conductor.md)
    - this is a zero config quick Conductor for development
- [`holochain` executable](./production_conductor.md)
    - this is a highly configurable sophisticated Conductor for running DNA instances long term

The articles that follow discuss these different Conductors in greater detail.