# UI Interfaces
`ui_interfaces` is an array of configurations for "UI Interfaces", meaning there can be multiple within one Conductor. UI Interfaces serve [UI Bundles](./conductor_ui_bundles.md) over HTTP.

**Optional**

### Properties

#### `id`: `string`
Give an ID of your choice to this UI Interface

#### `bundle`: `string`
A reference to the given ID of a defined [ui_bundle](./conductor_ui_bundles.md) to serve over this interface

#### `port`: `u16`
An integer value representing the port on the device to run this interface over. Must not conflict with any of the [interface](./conductor_interfaces.md) ports, nor another UI Interface port.

#### `dna_interface`: `string` Optional
A reference to the given ID of a defined [interface](./conductor_interfaces.md) this UI is allowed to make calls to. This is used to set the CORS headers and also to provide an extra virtual file endpoint at /_dna_config/ that allows [hc-web-client](https://github.com/holochain/hc-web-client) or another solution to redirect Holochain calls to the correct ip/port/protocol

### Example
```toml
[[ui_interfaces]]
id = "ui-interface-1"
bundle = "bundle1"
port = 3000
dna_interface = "websocket_interface"
```