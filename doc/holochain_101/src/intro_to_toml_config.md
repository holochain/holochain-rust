# Intro to TOML Config Files

To configure the `holochain` Conductor, a configuration file format called TOML is used. It stands for "Tom's Obvious Minimal Language" and was created by Tom Preston-Werner, one of the original founders of GitHub. The [documentation on GitHub](https://github.com/toml-lang/toml) for it is very good.

For the configuration of `holochain`, what is required are the use [tables](https://github.com/toml-lang/toml#table) and [arrays of tables](https://github.com/toml-lang/toml#array-of-tables).

A table is actually a collection of key/value pairs, and it looks like this:
```toml
[table-1]
key1 = "some string"
key2 = 123
```

An array of tables looks like this:
```toml
[[products]]
name = "Hammer"
sku = 738594937

[[products]]
name = "Nail"
sku = 284758393
color = "gray"
```
This represents two "product" items in an array.

In the following articles, how to configure the various properties of the `holochain` Conductor using these will be expanded on. First, knowing how to reference the configuration file for use by `holochain` will be covered below.

## `holochain` Config Files

When executing `holochain` in a terminal, a path to a configuration file can be given. This can be done with the following option:
```
--config
```
or for short
```
-c
```

This could look like:
```shell
holochain -c ./conductor-config.toml
```

`holochain` does require a configuration file to run. One should be given as an explicit argument, or exist in the default location. `holochain` will return an error if neither is given. The default location for the configuration file is in a subdirectory of the HOME directory on a device, at the path:
 ```toml
# Unix (Mac & Linux)
$HOME/.holochain/conductor/conductor-config.toml

# Windows
%HOME%\.holochain\conductor\conductor-config.toml
 ```

To jump ahead into what these configuration files can look like, you can check out [this folder on GitHub](https://github.com/holochain/holochain-rust/tree/develop/conductor/example-config) which has a number of examples. Otherwise, read on to understand each part.

> The [holochain-nodejs Conductor](./configuration_alternatives.md) also accepts the same TOML based configuration.
