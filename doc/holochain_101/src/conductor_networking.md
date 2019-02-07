# Networking

The Conductor currently uses mock networking by default. To use real networking you have to install the [n3h networking component](https://github.com/holochain/n3h) and add a configuration block into the config file to tell the Conductor where it can find n3h.


### Example
```toml
[network]
n3h_path = "/home/eric/holochain/n3h"
n3h_persistence_path = "/tmp"
bootstrap_nodes = []
```