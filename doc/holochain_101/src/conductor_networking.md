# Networking

`network` is a table for the configuration of how networking should behave in the Conductor. The Conductor currently uses mock networking by default. To network with other nodes you have to install the [n3h networking component](https://github.com/holochain/n3h) and add a configuration block into the config file to tell the Conductor where it can find n3h.

**Optional**

### Properties

#### `bootstrap_nodes`: `array of string` Optional
List of URIs that point to other nodes to bootstrap p2p connections.

#### `n3h_log_level`: `char`
Set the logging level used globally by N3H. Must be one of the following: 't', 'd', 'i', 'w', 'e'

#### `n3h_path`: `string`
Absolute path to the local installation/repository of n3h. Default is to a subdirectory of the $HOME directory on the device: `$HOME/.hc/net/n3h`

#### `n3h_persistence_path`: `string` Optional
Absolute path to the directory that n3h uses to store persisted data. The default is that a temporary self-removing directory for this transient data will be used.

#### `n3h_ipc_uri`: `string` Optional
URI pointing to an n3h process that is already running and not managed by this
Conductor. If this is set the Conductor does not spawn n3h itself and ignores the path configs above. Default is this value is empty.

### Example
```toml
[network]
n3h_path = "/home/eric/holochain/n3h"
n3h_persistence_path = "/tmp"
bootstrap_nodes = []
```
