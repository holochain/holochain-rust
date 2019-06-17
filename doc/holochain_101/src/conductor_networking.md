<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Contents**

- [Networking](#networking)
    - [Properties](#properties)
      - [`n3h_persistence_path`: `string`](#n3h_persistence_path-string)
      - [`bootstrap_nodes`: `array of string` Optional](#bootstrap_nodes-array-of-string-optional)
      - [`n3h_log_level`: `char`](#n3h_log_level-char)
      - [`n3h_ipc_uri`: `string` Optional](#n3h_ipc_uri-string-optional)
    - [Example](#example)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# Networking

`network` is a table for the configuration of how networking should behave in the Conductor. The Conductor currently uses mock networking by default. To network with other nodes Holochain will automatically setup the [n3h networking component](https://github.com/holochain/n3h). How `n3h` behaves can be configured with the following properties in a Conductor configuration file.

**Optional**

### Properties

#### `n3h_persistence_path`: `string`
Absolute path to the directory that n3h uses to store persisted data. The default is that a temporary self-removing directory for this transient data will be used.

#### `bootstrap_nodes`: `array of string` Optional
List of URIs that point to other nodes to bootstrap p2p connections.

#### `n3h_log_level`: `char`
Set the logging level used globally by N3H. Must be one of the following: 't', 'd', 'i', 'w', 'e'
Each value corresponding to the industry standard log level: Trace, Debug, Info, Warning, Error.

#### `n3h_ipc_uri`: `string` Optional
URI pointing to an n3h process that is already running and not managed by this
Conductor. If this is set the Conductor does not spawn n3h itself and ignores the path configs above. Default is this value is empty.

### Example
```toml
[network]
n3h_persistence_path = "/tmp"
bootstrap_nodes = []
```
