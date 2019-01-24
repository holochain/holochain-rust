# Containers



## Usage

The container requires a configuration file to run.

You can put your configuration file in `~/.holochain/container_config.toml` or run `holochain_container` explicitly with the `-c` to specify where to find it.

## Configuration File Spec

```toml
bridges = []

[[agents]]
id = "test agent 1"
key_file = "holo_tester.key"
name = "Holo Tester 1"
public_address = "HoloTester1-----------------------------------------------------------------------AAACZp4xHB"

[[agents]]
id = "test agent 2"
key_file = "holo_tester.key"
name = "Holo Tester 2"
public_address = "HoloTester2-----------------------------------------------------------------------AAAGy4WW9e"

[[dnas]]
file = "example-config/app_spec.hcpkg"
hash = "Qm328wyq38924y"
id = "app spec rust"

[[instances]]
agent = "test agent 1"
dna = "app spec rust"
id = "app spec instance 1"

[instances.storage]
path = "example-config/tmp-storage"
type = "file"

[[instances]]
agent = "test agent 2"
dna = "app spec rust"
id = "app spec instance 2"

[instances.storage]
path = "example-config/tmp-storage"
type = "file"

[[interfaces]]
admin = true
id = "websocket interface"

[[interfaces.instances]]
id = "app spec instance 1"

[[interfaces.instances]]
id = "app spec instance 2"

[interfaces.driver]
port = 3000
type = "websocket"

[[interfaces]]
admin = true
id = "http interface"

[[interfaces.instances]]
id = "app spec instance 1"

[[interfaces.instances]]
id = "app spec instance 2"

[interfaces.driver]
port = 4000
type = "http"

[logger]
type = "debug"
[[logger.rules.rules]]
color = "red"
exclude = false
pattern = "^err/"

[[logger.rules.rules]]
color = "white"
exclude = false
pattern = "^debug/dna"

[[logger.rules.rules]]
exclude = false
pattern = ".*"
```

### Using real networking
The container currently uses mock networking by default. To use real networking you have to install the [n3h networking component](https://github.com/holochain/n3h) and add a configuration block into the config file to tell the container where it can find n3h.  It should look something like this:

```toml
[network]
n3h_path = "/home/eric/holochain/n3h"
n3h_persistence_path = "/tmp"
bootstrap_nodes = []
```

## Testing HTTP interface using cURL

Currently the container supports the `websocket` and `http` interfaces.
Assuming the container http interface is running on port 4000 it can be tested by running:
`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":"0","method":"info/instances"}' http://localhost:4000`
