# Networking

`network` is a table for the configuration of how networking should behave in the Conductor. The Conductor currently uses mock networking by default. To network with other nodes Holochain will automatically setup the [n3h networking component](https://github.com/holochain/n3h). How `n3h` behaves can be configured with the following properties in a Conductor configuration file.

**Optional**

### Properties

#### `n3h_persistence_path`: `string`
Absolute path to the directory that n3h uses to store persisted data. Each Conductor should have a separate folder that `n3h_persistence_path` should be set to, because each should be assigned a custom network ID which will be persisted within that folder, thus they need to be distinct.

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
type = "n3h"
n3h_persistence_path = "./c1_network_files"
bootstrap_nodes = []
```


