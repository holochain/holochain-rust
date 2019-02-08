# Running Holochain Apps: Conductors

To introduce Conductors, it is useful to zoom out for a moment to the level of how Holochain runs on devices.

Holochain was designed to be highly platform and system compatible.  The core logic that runs a DNA instance was written in such a way that it could be included into many different codebases as a library, thus making it easier to build different implementations on the same platform as well as across platforms (MacOSX, Linux, Windows, Android, iOS, and more). Architecturally, Holochain DNAs are intended to be small composable units that provide bits of distributed data integrity functionality.  Thus most Holochain based applications will actually be assemblages of many "bridged" DNA instances.  For this to work we needed a distinct layer that orchestrates the data flow (i.e. zome function call requests and responses), between the transport layer (i.e. HTTP, Websockets, Unix domain sockets, etc) and the DNA instances.  We call the layer that performs these two crucial functions, the *Conductor*, and we have written a `conductor_api` library to make it easy to build actual Conductor implementations.

Conductors play quite a number of important roles:
- installing, uninstalling, configuring, starting and stopping instances of DNA
- exposing APIs to securely make function calls into the Zome functions of DNA instances
- accepting information concerning the cryptographic keys and agent info to be used for identity and signing, and passing it into Holochain
- establishing "bridging" between DNA instances
- serving files for web based user interfaces that connect to these DNA instances over the interfaces

Those are the basic functions of a Conductor, but in addition to that, a Conductor also allows for the configuration of the networking module for Holochain, enables logging, and if you choose to, exposes APIs at a special 'admin' level that allows for the dynamic configuration of the Conductor while it runs. By default, configuration of the Conductor is done via a static configuration file, written in [TOML](https://github.com/toml-lang/toml).


In regards to the Zome functions APIs, Conductors can implement a diversity of interfaces to perform these function calls, creating an abundance of opportunity. Another way to build Holochain into an application is to use language bindings from the Rust built version of the Conductor, to another language, that then allows for the direct use of Holochain in that language.

There are currently three Conductor implementations:
- [Nodejs](https://www.npmjs.com/package/@holochain/holochain-nodejs)
    - this is built using the language bindings approach, using [neon](https://github.com/neon-bindings/neon)
- [hc run](./development_conductor.md)
    - this is a zero config quick Conductor for development
- [`holochain` executable](./production_conductor.md)
    - this is a highly configurable sophisticated Conductor for running DNA instances long term

The articles that follow discuss these different Conductors in greater detail.

> What is now known as a "Conductor" used to be called a "Container", so if you see the language of Container from other versions know that these refer to the same thing. Fun fact: because this component has such a variety of functions, there was some difficulty in naming it. The word "Conductor" was finally chosen because it actually implies multiple metaphors, each of which resonates with an aspect of what the Conductor does. Like an orchestra conductor, it helps several parts work together as a whole. Like a train conductor, it oversees and instructs how the engine runs. Like an electricity conductor, it allows information to pass through it.
