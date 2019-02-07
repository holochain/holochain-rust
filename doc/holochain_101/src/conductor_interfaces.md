# Interfaces
`interfaces` is an array of configurations for "interfaces", meaning there can be multiple within one Conductor. Interfaces are user facing and make Zome functions, info, and optionally admin functions available to GUIs, browser based web UIs, local native UIs, and other local applications and scripts.
The following implementations are already developed:
* WebSockets
* HTTP

The instances (referenced by ID) that are to be made available via that interface should be listed.
An admin flag can enable special Conductor functions for programatically changing the configuration
(e.g. installing apps), which even persists back to the configuration file.

**Optional**

### Properties

#### `id`: `string`
Give an ID of your choice to this interface

#### `driver`: `InterfaceDriver`
A table which should provide info regarding the protocol and port over which this interface should run

#### `InterfaceDriver.type`: `enum`
Select between different protocols for serving the API. There are two so far:
- `websocket`: serve the API as JSON-RPC via [WebSockets](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API)
- `http`: serve the API as JSON-RPC via HTTP

These are discussed in great detail in [Intro to JSON-RPC Interfaces](./json_rpc_interfaces.md), and the following articles.

#### `InterfaceDriver.port`: `u16`
An integer value representing the port on the device to run this interface over

#### `admin`: `bool` Optional
Whether to expose [admin level functions](./conductor_admin.md) for dynamically administering the Conductor via this JSON-RPC interface. Defaults to false.
    
#### `instances`: `array of InstanceReferenceConfiguration`
An array of tables which should provide the IDs of [instances](./conductor_instances.md) to serve over this interface. Only the ones which are listed here will be served.

#### `InstanceReferenceConfiguration.id`: `string`
A reference to the given ID of a defined [instance](./conductor_instances.md)

### Example Without Admin
```toml
[[interfaces]]
id = "websocket interface"

[[interfaces.instances]]
id = "app spec instance 1"

[interfaces.driver]
type = "websocket"
port = 4000
```

### Example With Admin
```toml
[[interfaces]]
id = "http interface"
admin = true

[[interfaces.instances]]
id = "app spec instance 1"

[interfaces.driver]
type = "http"
port = 4000
```