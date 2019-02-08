# Production Conductor

In addition to the zero config development Conductor using `hc run`, there is a highly configurable sophisticated Conductor for running DNA instances long term.

This Conductor will play an important role in making the use of Holochain truly easy for end users, because it supports all the functionality that those users are likely to want, in terms of managing their Holochain apps, or hApps, just on a low level. On that note, a graphical user interface that exposes all the functionality of this Conductor to users is under development.

For now, use of this Conductor must happen mostly manually, and by tech-savvy users or developers.

This Conductor is simply a command line tool called `holochain`. Its only function is to boot a Conductor based on a configuration file, and optionally, the ability to write changes back to that file. Within that Conductor many DNA instances can be run for one or more agents, multiple types of interfaces to the APIs can be exposed, UI file bundles can be served, and logs from all of that can be accessed.

The first step to using `holochain` is of course installing it. Instructions for installation can be found in its [README](https://github.com/holochain/holochain-rust/tree/develop/conductor#install). If you wish to attempt any of the things you read in this chapter while going through it, you will need to have installed the executable.

Like Holochain core, this particular Conductor is written in Rust. View it on GitHub [here](https://github.com/holochain/holochain-rust/tree/develop/conductor).

To understand how to configure the `holochain` Conductor, check out the [next article](./intro_to_toml_config.md).
