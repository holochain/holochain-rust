# Instances

`instances` is an array of configurations of DNA instances, each of which is a running copy of a [DNA](./conductor_dnas.md), by a particular [agent](./conductor_agents.md). Based on these configurations, the Conductor will attempt to start up these instances, initializing (or resuming) a local source chain and DHT. It is possible to use the same DNA with multiple different [agents](./conductor_agents.md), but it is **not** recommended to run two instances with the same DNA and same agent. An instance has a configurable `storage` property, which can be set to save to disk, or just store temporarily in memory, which is useful for testing purposes.

**Optional**

### Properties

#### `id`: `string`
Give an ID of your choice to this instance

#### `agent`: `string`
A reference to the given ID of a defined [agent](./conductor_agents.md)

#### `dna`: `string`
A reference to the given ID of a defined [DNA](./conductor_dnas.md)

#### `storage`: `StorageConfiguration`
A table for configuring the approach to storage of the local source chain and DHT for this instance

#### `StorageConfiguration.type`: `enum`
Select between different storage implementations. There are two so far:
- `memory`: Persist actions taken in this instance only to memory. Everything will disappear when the Conductor process stops.
- `file`: Persist actions taken in this instance to the disk of the device the Conductor is running on. If the Conductor process stops and then restarts, the actions taken will resume at the place in the local source chain they last were at.
- `pickledb` : Persists to a fast rust implementation. This persists every 5 minutes and on a drop trait, actions will continue from where they were last were at if application is every restarted. Takes in path string which is directory where pickledb instance will be persisting to

#### `StorageConfiguration.path`: `string`
Path to the folder in which to store the data for this instance.

### Example
```toml
[[instances]]
id = "app spec instance 1"
agent = "test agent 1"
dna = "app spec rust"

[instances.storage]
type = "file"
path = "example-config/tmp-storage"
```
