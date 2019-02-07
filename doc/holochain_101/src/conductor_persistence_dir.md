# Persistence Directory

This is a simple key/value pair specifying a directory on the device to persist the config file, DNAs, and UI bundles, if changes are made dynamically over the JSON-RPC admin API. This is only relevant if you are running one of the [interfaces](./conductor_interfaces.md) with `admin = true`. The default value is in a subdirectory of the $HOME directory, `$HOME/.holochain/conductor`.

**Optional**

If you start a Conductor that has this value set, but then make no changes via the JSON-RPC admin interface, the persistence directory will not be utilized and the Conductor config file you started with will not be moved into that directory. On the other hand, if you do make any changes to the configuration by calling one of the [dynamic admin functions](./conductor_admin.md) then whatever the value of the `persistence_dir` is for that Conductor config, it will create that directory, and then persist the modified Conductor configuration file there. It would then be wise to utilize **that** Conductor config in the future, instead of the original.

Within this `persistence_dir` that is now on the device, there are a number of possible files and folders.

`conductor-config.toml` is the new configuration file, which will be repeatedly written to with any further dynamic updates. This is useful so that when the Conductor is stopped, or if it dies for some reason, when you restart it will behave the same as before.

`storage` is a directory used for persisting the data for [instances](./conductor_instances.md), in particular when new instances are added via the `admin/instance/add` admin function.

`dna` is a directory used for copying [DNA](./conductor_dnas.md) package files into if the `admin/dna/install_from_file` admin function is called.

`static` is a directory used for copying [UI Bundle](./conductor_ui_bundles.md) files into if the `admin/ui/install` admin function is called.

### Example
```toml
persistence_dir = "/home/user/my_holochain"
```


